use crate::state_types::*;
use crate::types::*;
use enclose::*;
use futures::{future, Future};
use lazy_static::*;
use serde_derive::*;
use serde::Serialize;
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
struct UserData {
    auth: Option<Auth>,
    addons: Vec<Descriptor>,
}
impl Default for UserData {
    fn default() -> Self {
        UserData {
            auth: None,
            addons: DEFAULT_ADDONS.to_owned(),
        }
    }
}

type UserDataHolder = Rc<RefCell<Option<UserData>>>;
#[derive(Default)]
pub struct UserMiddleware<T: Environment> {
    //id: usize,
    state: UserDataHolder,
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

    fn load(&self) -> EnvFuture<UserData> {
        let current_state = self.state.borrow().to_owned();
        if let Some(ud) = current_state {
            return Box::new(future::ok(ud));
        }

        let state = self.state.clone();
        let fut = T::get_storage(USER_DATA_KEY).and_then(move |result: Option<Box<UserData>>| {
            let ud: UserData = *result.unwrap_or_default();
            let _ = state.replace(Some(ud.to_owned()));
            future::ok(ud)
        });
        Box::new(fut)
    }

    fn save(state: UserDataHolder, new_user_data: UserData) -> EnvFuture<()> {
        let fut = T::set_storage(USER_DATA_KEY, Some(&new_user_data));
        state.replace(Some(new_user_data));
        fut
    }

    fn get_current_key(state: &UserDataHolder) -> Option<AuthKey> {
        match *state.borrow() {
            Some(UserData {
                auth: Some(Auth { ref key, .. }),
                ..
            }) => Some(key.to_owned()),
            _ => None,
        }
    }

    //fn exec_or_warning(fut: EnvFuture<()>, emit: Rc<DispatcherFn>) {
    //}

    fn exec_or_fatal(fut: EnvFuture<()>, emit: Rc<DispatcherFn>) {
        T::exec(Box::new(fut.or_else(move |e| {
            emit(&Action::UserMiddlewareFatal(MiddlewareError::Env(
                e.to_string(),
            )));
            future::err(())
        })));
    }

    fn handle_action_user(&self, action_user: &ActionUser, emit: Rc<DispatcherFn>) {
        // Login/Register follow the same code (except the method); and perhaps Register will take
        // extra (gdpr, etc.)
        // @TODO emit UserChanged
        // @TODO share more code, a lot of those clone the state, all of these handle API errors
        let api_request_body: APIRequestBody = action_user.into();
        let req = Request::post(format!("{}/api/{}", &self.api_url, api_request_body.method_name()))
            .body(APIRequest{ key: Self::get_current_key(&self.state), body: api_request_body })
            .expect("failed to build API request");

        match action_user {
            // These DO NOT require authentication
            ActionUser::Login { .. } | ActionUser::Register { .. } => {
                // @TODO register
                let state = self.state.clone();
                let fut = T::fetch_serde::<_, APIResult<AuthResponse>>(req)
                    .and_then(enclose!((action_user, emit) move |result| {
                        match *result {
                            APIResult::Ok {
                                result: AuthResponse { key, user },
                            } => {
                                // @TODO pull addons
                                Self::save(state, UserData { auth: Some(Auth{ key, user }), addons: DEFAULT_ADDONS.to_owned() })
                            }
                            APIResult::Err { error } => {
                                emit(&Action::UserOpError(action_user, MiddlewareError::API(error)));
                                Box::new(future::ok(()))
                            }
                        }
                    }))
                    .or_else(enclose!((action_user, emit) move |e| {
                        emit(&Action::UserOpError(action_user, MiddlewareError::Env(e.to_string())));
                        future::err(())
                    }));
                T::exec(Box::new(fut));
            }
            // The following actions require authentication
            ActionUser::Logout => {
                // NOTE: This will clean up the storage first, and do the API call later
                // if the API call fails, it will be a UserOpError but the user would still be
                // logged out
                let fut = Self::save(self.state.clone(), Default::default())
                    .and_then(|_| {
                        // @TODO emit UserChanged ASAP here
                        // @TODO destroy session
                        future::ok(())
                    })
                    // @TODO remove duplication here
                    .or_else(enclose!((action_user, emit) move |e| {
                        emit(&Action::UserOpError(action_user, MiddlewareError::Env(e.to_string())));
                        future::err(())
                    }));
                T::exec(Box::new(fut));
            }
            ActionUser::PullAddons => {
                let state = self.state.clone();
                let fut = T::fetch_serde::<_, APIResult<CollectionResponse>>(req)
                    .and_then(enclose!((action_user, emit) move |result| {
                        // @TODO fetch_api which returns MiddlewareError?
                        match *result {
                            APIResult::Ok {
                                result: CollectionResponse { addons },
                            } => {
                                // @TODO preserve auth
                                // @TODO this  needs to determine whether the remote dataset is newer or not
                                // @TODO emit AddonsChanged
                                Self::save(state, UserData { auth: None, addons })
                            }
                            APIResult::Err { error } => {
                                emit(&Action::UserOpError(action_user, MiddlewareError::API(error)));
                                Box::new(future::ok(()))
                            }
                        }
                    }))
                    .or_else(enclose!((action_user, emit) move |e| {
                        emit(&Action::UserOpError(action_user, MiddlewareError::Env(e.to_string())));
                        future::err(())
                    }));
                T::exec(Box::new(fut));
            }
            ActionUser::PushAddons => {
                // @TODO
            }
        }
    }
}

impl<T: Environment> Handler for UserMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        if let Action::Load(action_load) = action {
            let fut = self
                .load()
                .and_then(enclose!((emit, action_load) move |ud| {
                    emit(&Action::LoadWithUser(ud.auth.map(|a| a.user), ud.addons.to_owned(), action_load));
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
                    let new_user_data = UserData{
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
