use crate::state_types::Output::*;
use crate::state_types::*;
use crate::types::addons::*;
use crate::types::api::*;
use enclose::*;
use futures::{future, Future};
use lazy_static::*;
use serde::de::DeserializeOwned;
use serde_derive::*;
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::convert::Into;
use std::marker::PhantomData;
use std::rc::Rc;

const USER_DATA_KEY: &str = "userData";
const DEFAULT_API_URL: &str = "https://api.strem.io";
lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(include_bytes!("../../stremio-official-addons/index.json"))
            .expect("official addons JSON parse");
}

// These will be stored, so they need to implement both Serialize and Deserilaize
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    fn get_auth_key(&self) -> Option<&AuthKey> {
        Some(&self.auth.as_ref()?.key)
    }
    fn action_to_request(&self, action: &ActionUser) -> Option<APIRequest> {
        let key = self.get_auth_key().map(ToOwned::to_owned);
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
    fn are_addons_same(&self, addons: &[Descriptor]) -> bool {
        let urls = self.addons.iter().map(|a| &a.transport_url);
        let new_urls = addons.iter().map(|a| &a.transport_url);
        urls.eq(new_urls)
    }
}

type MiddlewareFuture<T> = Box<Future<Item = T, Error = MiddlewareError>>;
type UserStorageHolder = Rc<RefCell<Option<UserStorage>>>;
#[derive(Default)]
pub struct ContextMiddleware<T: Environment> {
    //id: usize,
    state: UserStorageHolder,
    api_url: String,
    env: PhantomData<T>,
}
impl<T: Environment> ContextMiddleware<T> {
    pub fn new() -> Self {
        ContextMiddleware {
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
            .map_err(Into::into);
        Box::new(fut)
    }

    fn save(state: UserStorageHolder, new_user_data: UserStorage) -> MiddlewareFuture<()> {
        let fut = T::set_storage(USER_DATA_KEY, Some(&new_user_data)).map_err(Into::into);
        state.replace(Some(new_user_data));
        Box::new(fut)
    }

    // save_and_emit will only emit AuthChanged/AddonsChangedfromPull if saving to storage is successful
    fn save_and_emit(
        state: UserStorageHolder,
        new_us: UserStorage,
        emit: Rc<DispatcherFn>,
    ) -> MiddlewareFuture<()> {
        let addons_changed = state
            .borrow()
            .as_ref()
            .map_or(false, |us| !us.are_addons_same(&new_us.addons));
        let auth_changed = state
            .borrow()
            .as_ref()
            .map_or(false, |us| us.get_auth_key() != new_us.get_auth_key());
        let user = new_us.auth.as_ref().map(|a| a.user.to_owned());
        let fut = Self::save(state, new_us).and_then(move |_| {
            if auth_changed {
                emit(&AuthChanged(user).into());
            }
            if addons_changed {
                emit(&AddonsChangedFromPull.into());
            }
            future::ok(())
        });
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
            .map_err(Into::into)
            .and_then(|result| match *result {
                APIResult::Err { error } => future::err(error.into()),
                APIResult::Ok { result } => future::ok(result),
            });
        Box::new(fut)
    }

    fn exec_or_error(fut: MiddlewareFuture<()>, action_user: ActionUser, emit: Rc<DispatcherFn>) {
        T::exec(Box::new(fut.or_else(move |e| {
            emit(&UserOpError(action_user, e).into());
            future::err(())
        })));
    }

    fn exec_or_fatal(fut: MiddlewareFuture<()>, emit: Rc<DispatcherFn>) {
        T::exec(Box::new(fut.or_else(move |e| {
            emit(&ContextMiddlewareFatal(e).into());
            future::err(())
        })));
    }

    fn handle_user_op(
        state: UserStorageHolder,
        api_url: String,
        current_storage: UserStorage,
        action_user: &ActionUser,
        emit: Rc<DispatcherFn>,
    ) {
        let api_req = match current_storage.action_to_request(action_user) {
            Some(r) => r,
            None => {
                emit(&UserOpError(action_user.to_owned(), MiddlewareError::AuthRequired).into());
                return;
            }
        };

        let fut: MiddlewareFuture<()> = match action_user {
            ActionUser::Login { .. } | ActionUser::Register { .. } => {
                // Sends the initial request (authentication), and pulls addons collection after
                let fut = Self::api_fetch::<AuthResponse>(&api_url, api_req)
                    .and_then(move |AuthResponse { key, user }| {
                        let pull_req = APIRequest::AddonCollectionGet {
                            auth_key: key.to_owned(),
                        };
                        Self::api_fetch::<CollectionResponse>(&api_url, pull_req).and_then(
                            move |CollectionResponse { addons, .. }| {
                                future::ok(UserStorage {
                                    auth: Some(Auth { key, user }),
                                    addons,
                                })
                            },
                        )
                    })
                    .and_then(enclose!((emit) | us | Self::save_and_emit(state, us, emit)));
                Box::new(fut)
            }
            ActionUser::Logout => {
                // Reset the storage, and then issue the API request to clean the session
                // that way, you can logout even w/o a connection
                let fut = Self::save_and_emit(state.clone(), Default::default(), emit.clone())
                    .and_then(move |_| Self::api_fetch::<SuccessResponse>(&api_url, api_req))
                    .and_then(|_| future::ok(()));
                Box::new(fut)
            }
            ActionUser::PullAddons => {
                let auth = current_storage.auth.to_owned();
                let fut = Self::api_fetch::<CollectionResponse>(&api_url, api_req).and_then(
                    enclose!((emit) move |CollectionResponse { addons, .. }| -> MiddlewareFuture<()> {
                        // @TODO handle last_modified here
                        // The authentication has changed in between - abort
                        match *state.borrow() {
                            Some(ref s) if s.get_auth_key() == current_storage.get_auth_key() => {},
                            _ => { return Box::new(future::err(MiddlewareError::AuthRace)) },
                        }
                        Self::save_and_emit(state, UserStorage { auth, addons }, emit.clone())
                    }),
                );
                Box::new(fut)
            }
            ActionUser::PushAddons => {
                let fut = Self::api_fetch::<SuccessResponse>(&api_url, api_req)
                    .and_then(|_| future::ok(()));
                Box::new(fut)
            }
        };
        Self::exec_or_error(fut, action_user.to_owned(), emit.clone());
    }
}

impl<T: Environment> Handler for ContextMiddleware<T> {
    fn handle(&self, msg: &Msg, emit: Rc<DispatcherFn>) {
        if let Msg::Action(Action::Load(action_load)) = msg {
            let fut = self.load().and_then(
                enclose!((emit, action_load) move |UserStorage{ auth, addons }| {
                    // @TODO this is the place to inject built-in addons, such as the
                    // Library/Notifications addon
                    let ctx = Context { user:auth.map(|a| a.user), addons: addons.to_owned() };
                    emit(&Internal::LoadWithCtx(ctx, action_load).into());
                    future::ok(())
                }),
            );
            Self::exec_or_fatal(Box::new(fut), emit.clone());
        }

        if let Msg::Action(Action::AddonOp(action_addon)) = msg {
            let state = self.state.clone();
            let fut = self
                .load()
                .and_then(enclose!((emit, action_addon) move |us| {
                    let addons = match action_addon {
                        ActionAddon::Remove{ transport_url } => {
                            us.addons.iter()
                                .filter(|a| a.transport_url != transport_url)
                                .cloned()
                                .collect()
                        },
                        ActionAddon::Install(descriptor) => {
                            let mut addons = us.addons.to_owned();
                            addons.push(*descriptor);
                            addons
                        },
                    };
                    let new_user_storage = UserStorage{
                        addons,
                        ..us
                    };
                    Self::save(state, new_user_storage)
                        .and_then(move |_| {
                            emit(&AddonsChanged.into());
                            future::ok(())
                        })
                }));
            Self::exec_or_fatal(Box::new(fut), emit.clone());
        }

        if let Msg::Action(Action::UserOp(action_user)) = msg {
            let state = self.state.clone();
            let api_url = self.api_url.to_owned();
            let fut = self
                .load()
                .and_then(enclose!((emit, action_user) move |ud| {
                    // handle_user_op will spawn it's own tasks,
                    // since they are handled with exec_or_error
                    Self::handle_user_op(state, api_url, ud, &action_user, emit.clone());
                    future::ok(())
                }));
            Self::exec_or_fatal(Box::new(fut), emit.clone());
        }
    }
}
