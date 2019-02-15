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

#[derive(Default)]
pub struct UserMiddleware<T: Environment> {
    //id: usize,
    state: Rc<RefCell<Option<UserData>>>,
    api_url: String,
    env: PhantomData<T>,
}
impl<T: Environment> UserMiddleware<T> {
    pub fn new() -> Self {
        UserMiddleware {
            state: Rc::new(RefCell::new(None)),
            api_url: "https://api.strem.io".to_owned(),
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

    fn handle_user_op(&self, user_op: &ActionUser, emit: Rc<DispatcherFn>) {
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
        match user_op {
            ActionUser::PullAddons => {
                // @TODO if we have auth_key
                // @TODO check if auth_key has changed, before persisting
                let req = Request::post(format!("{}/api/addonCollection", &self.api_url))
                    .body(CollectionRequest {})
                    .expect("failed to build API request");
                let state = self.state.clone();
                let fut = T::fetch_serde::<_, APIResult<CollectionResponse>>(req)
                    .and_then(move |result| {
                        match *result {
                            APIResult::Ok {
                                result: CollectionResponse { addons },
                            } => {
                                let new_user_data = UserData { auth: None, addons };
                                state.replace(Some(new_user_data.to_owned()));
                                T::set_storage(USER_DATA_KEY, Some(&new_user_data))
                            }
                            // @TODO should this error be handled better?
                            APIResult::Err { error } => Box::new(future::err(error.message.into())),
                        }
                    })
                    .or_else(enclose!((emit) move |e| {
                        // @TODO better err handling?
                        // there are a few types of errors here: network errors, deserialization
                        // errors, API errors
                        emit(&Action::UserMError(e.to_string()));
                        future::err(())
                    }));
                T::exec(Box::new(fut));
            }
            _ => {
                // @TODO
            }
        }
    }
}

impl<T: Environment> Handler for UserMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        if let Action::Load(action_load) = action {
            let action_load = action_load.to_owned();
            let fut = self
                .load()
                .and_then(enclose!((emit) move |ud| {
                    emit(&Action::LoadWithUser(ud.auth.map(|a| a.user), ud.addons.to_vec(), action_load));
                    future::ok(())
                }))
                .or_else(enclose!((emit) move |e| {
                    emit(&Action::UserMFatal(e.to_string()));
                    future::err(())
                }));
            T::exec(Box::new(fut));
        }

        if let Action::AddonRemove(descriptor) | Action::AddonInstall(descriptor) = action {
            let fut = self.load()
                .and_then(enclose!((emit) move |ud| {
                    //self.save(ud)
                    future::ok(())
                }))
                .or_else(enclose!((emit) move |e| {
                    emit(&Action::UserMFatal(e.to_string()));
                    future::err(())
                }));
            T::exec(Box::new(fut));
        }

        if let Action::UserOp(user_op) = action {
            self.handle_user_op(user_op, emit);
        }
    }
}
