use crate::state_types::*;
use crate::types::*;
use enclose::*;
use futures::{future, Future};
use lazy_static::*;
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
    key: String,
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

    fn exec_load_fut(&self, fut: EnvFuture<()>, emit: Rc<DispatcherFn>) {
        T::exec(Box::new(fut.or_else(move |e| {
            emit(&Action::UserMiddlewareFatal(MiddlewareError::Env(
                e.to_string(),
            )));
            future::err(())
        })));
    }

    fn handle_action_user(&self, action_user: &ActionUser, emit: Rc<DispatcherFn>) {
        // @TODO actions that do not require auth Login, Register, Logout; those can emit
        // UserOpError and UserChanged
        // @TODO actions that do require auth PullAddons (persist if same auth key), PushAddons
        // Login/Register follow the same code (except the method); and perhaps Register will take
        // extra (gdpr, etc.)
        // Logout is a separate operation, it clears the UD and sets it to default; it
        // should first clear the UD, THEN clear the session, to make it logout even w/o a conn
        // PushAddons/PullAddon just pushes and sends a UserOpWarning if it fails
        // PullAddons will set the storage (if the authkey has not changed), and emit AddonsChanged
        //  it also needs to determine whether the remote dataset is newer or not
        match action_user {
            ActionUser::Login { .. } | ActionUser::Register { .. } => {}
            ActionUser::Logout => {}
            ActionUser::PullAddons => {
                // @TODO if we have auth_key
                // @TODO check if auth_key has changed, before persisting
                let req = Request::post(format!("{}/api/addonCollection", &self.api_url))
                    .body(CollectionRequest {})
                    .expect("failed to build API request");
                let state = self.state.clone();
                let fut = T::fetch_serde::<_, APIResult<CollectionResponse>>(req)
                    .and_then(enclose!((action_user, emit) move |result| {
                        // @TODO fetch_api which returns MiddlewareError?
                        match *result {
                            APIResult::Ok {
                                result: CollectionResponse { addons },
                            } => {
                                // @TODO preserve auth
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
            ActionUser::PushAddons => {}
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
            self.exec_load_fut(Box::new(fut), emit.clone());
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
            self.exec_load_fut(Box::new(fut), emit.clone());
        }

        if let Action::UserOp(action_user) = action {
            self.handle_action_user(action_user, emit.clone());
        }
    }
}
