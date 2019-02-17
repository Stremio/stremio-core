use crate::state_types::*;
use crate::types::*;
use enclose::*;
use futures::{future, Future};
use lazy_static::*;
use serde::de::DeserializeOwned;
use serde_derive::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

const USER_DATA_KEY: &str = "userData";
const DEFAULT_API_URL: &str = "https://api.strem.io";
lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(include_bytes!("../../stremio-official-addons/index.json"))
            .expect("official addons JSON parse");
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Auth {
    key: AuthKey,
    user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserStorage {
    auth: Option<Auth>,
    addons: Vec<Descriptor>,
}
impl Default for UserStorage {
    fn default() -> Self {
        UserStorage {
            auth: None,
            addons: DEFAULT_ADDONS.to_owned(),
        }
    }
}
impl UserStorage {
    fn get_current_key(&self) -> Option<&AuthKey> {
        Some(&self.auth.as_ref()?.key)
    }
    fn action_to_request(&self, action: &ActionUser) -> Option<APIRequest> {
        let key = self.get_current_key().map(|k| k.to_owned());
        Some(match action.to_owned() {
            ActionUser::Login { email, password } => APIRequest::Login { email, password },
            ActionUser::Register { email, password } => APIRequest::Register { email, password },
            ActionUser::Logout => APIRequest::Logout { auth_key: key? },
            ActionUser::PullAddons => APIRequest::AddonCollectionGet { auth_key: key? },
            ActionUser::PushAddons => APIRequest::AddonCollectionSet {
                auth_key: key?,
                addons: self.addons.to_owned(),
            },
        })
    }
}

type MiddlewareFuture<T> = Box<Future<Item = T, Error = MiddlewareError>>;
type UserStorageHolder = Rc<RefCell<Option<UserStorage>>>;
#[derive(Default)]
pub struct UserMiddleware<T: Environment> {
    //id: usize,
    state: UserStorageHolder,
    api_url: String,
    env: PhantomData<T>,
}
impl<T: Environment> UserMiddleware<T> {
    pub fn new() -> Self {
        UserMiddleware {
            state: Default::default(),
            api_url: DEFAULT_API_URL.to_owned(),
            env: PhantomData,
        }
    }

    fn load(&self) -> MiddlewareFuture<UserStorage> {
        let current_state = self.state.borrow().to_owned();
        if let Some(ud) = current_state {
            return Box::new(future::ok(ud));
        }

        let state = self.state.clone();
        let fut = T::get_storage(USER_DATA_KEY)
            .and_then(move |result: Option<Box<UserStorage>>| {
                let ud: UserStorage = *result.unwrap_or_default();
                let _ = state.replace(Some(ud.to_owned()));
                future::ok(ud)
            })
            .map_err(|e| e.into());
        Box::new(fut)
    }

    fn save(state: UserStorageHolder, new_user_data: UserStorage) -> MiddlewareFuture<()> {
        let fut = T::set_storage(USER_DATA_KEY, Some(&new_user_data)).map_err(|e| e.into());
        state.replace(Some(new_user_data));
        Box::new(fut)
    }

    fn api_fetch<OUT: 'static>(base_url: &str, api_req: APIRequest) -> MiddlewareFuture<OUT>
    where
        OUT: DeserializeOwned,
    {
        let url = format!("{}/api/{}", base_url, api_req.method_name());
        let req = match Request::post(url).body(api_req) {
            Ok(req) => req,
            Err(e) => return Box::new(future::err(MiddlewareError::Env(e.to_string()))),
        };

        let fut = T::fetch_serde::<_, APIResult<OUT>>(req)
            .map_err(|e| e.into())
            .and_then(|result| match *result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(error.into()),
            });
        Box::new(fut)
    }

    fn exec_or_error(fut: MiddlewareFuture<()>, action_user: ActionUser, emit: Rc<DispatcherFn>) {
        T::exec(Box::new(fut.or_else(move |e| {
            emit(&Action::UserOpError(action_user, e));
            future::err(())
        })));
    }

    fn exec_or_fatal(fut: MiddlewareFuture<()>, emit: Rc<DispatcherFn>) {
        T::exec(Box::new(fut.or_else(move |e| {
            emit(&Action::UserMiddlewareFatal(e));
            future::err(())
        })));
    }

    fn handle_action_user(&self, action_user: &ActionUser, emit: Rc<DispatcherFn>) {
        let base_url = self.api_url.to_owned();
        // @TODO call .load first
        let us = self.state.borrow().to_owned().expect("TODO");
        let api_req = match us.action_to_request(&action_user) {
            Some(r) => r,
            None => {
                emit(&Action::UserOpError(
                    action_user.to_owned(),
                    MiddlewareError::AuthRequired,
                ));
                return;
            }
        };

        let state = self.state.clone();
        let fut: MiddlewareFuture<()> = match action_user {
            ActionUser::Login { .. } | ActionUser::Register { .. } => {
                let fut = Self::api_fetch::<AuthResponse>(&base_url, api_req).and_then(
                    move |AuthResponse { key, user }| {
                        // @TODO: Emit UserChanged, Pull Addons
                        let new_user_storage = UserStorage {
                            auth: Some(Auth { key, user }),
                            addons: DEFAULT_ADDONS.to_owned(),
                        };
                        Self::save(state, new_user_storage)
                    },
                );
                Box::new(fut)
            }
            ActionUser::Logout => {
                let fut = Self::save(state.clone(), Default::default()).and_then(|_| {
                    // @TODO emit UserChanged ASAP here
                    // @TODO destroy session
                    future::ok(())
                });
                Box::new(fut)
            }
            ActionUser::PullAddons => {
                let fut = Self::api_fetch::<CollectionResponse>(&base_url, api_req).and_then(
                    |CollectionResponse { addons }| {
                        // @TODO: keep auth
                        Self::save(state, UserStorage { auth: None, addons })
                    },
                );
                Box::new(fut)
            }
            ActionUser::PushAddons => {
                let fut = future::ok(());
                // @TODO
                Box::new(fut)
            }
        };
        Self::exec_or_error(fut, action_user.to_owned(), emit.clone());
    }
}

impl<T: Environment> Handler for UserMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        if let Action::Load(action_load) = action {
            let fut = self
                .load()
                .and_then(enclose!((emit, action_load) move |UserStorage{ auth, addons }| {
                    emit(&Action::LoadWithUser(auth.map(|a| a.user), addons.to_owned(), action_load));
                    future::ok(())
                }));
            Self::exec_or_fatal(Box::new(fut), emit.clone());
        }

        if let Action::AddonOp(action_addon) = action {
            let state = self.state.clone();
            let fut = self
                .load()
                .and_then(enclose!((emit, action_addon) move |ud| {
                    let addons = match action_addon {
                        ActionAddon::Remove{ transport_url } => {
                            ud.addons.iter()
                                .filter(|a| a.transport_url != transport_url)
                                .cloned()
                                .collect()
                        },
                        ActionAddon::Install(descriptor) => {
                            let mut addons = ud.addons.to_owned();
                            addons.push(*descriptor);
                            addons
                        },
                    };
                    emit(&Action::AddonsChanged(addons.to_owned()));
                    let new_user_data = UserStorage{
                        addons,
                        ..ud
                    };
                    Self::save(state, new_user_data)
                }));
            Self::exec_or_fatal(Box::new(fut), emit.clone());
        }

        if let Action::UserOp(action_user) = action {
            self.handle_action_user(action_user, emit.clone());
        }
    }
}
