use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use serde_derive::*;
use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::RefCell;
use lazy_static::*;

const USER_DATA_KEY: &str = "userData";

lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> = serde_json::from_slice(include_bytes!("../../stremio-official-addons/index.json")).expect("official addons JSON parse");
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
    pub env: PhantomData<T>,
}
impl<T: Environment> UserMiddleware<T> {
    pub fn new() -> Self {
        UserMiddleware {
            state: Rc::new(RefCell::new(None)),
            env: PhantomData,
        }
    }
    // @TODO is there a better way to do this?
    fn load(&self) -> EnvFuture<UserData> {
        let current_state = self.state.borrow().to_owned();
        if let Some(ud) = current_state {
            return Box::new(future::ok(ud))
        }

        let state = self.state.clone();
        let fut = T::get_storage(USER_DATA_KEY)
           .and_then(move |result: Option<Box<UserData>>| {
                let ud: UserData = *result.unwrap_or_default();
                let _ = state.replace(Some(ud.to_owned()));
                future::ok(ud)
            });
        Box::new(fut)
    }
}
impl<T: Environment> Handler for UserMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // @TODO Action::SyncAddons, Action::TryLogin
        if let Action::Load(action_load) = action {
            let action_load = action_load.to_owned();
            /*
            let req = Request::post("https://api.strem.io/api/addonCollectionGet")
                .body(CollectionRequest {})
                .unwrap();
            let fut = T::fetch_serde::<_, APIResult<CollectionResponse>>(req)
                .and_then(move |result| {
                    // @TODO err handling
                    match *result {
                        APIResult::Ok {
                            result: CollectionResponse { addons },
                        } => {
                            emit(&Action::LoadWithAddons(addons.to_vec(), action_load));
                        }
                        APIResult::Err { error } => {
                            // @TODO err handling
                            dbg!(error);
                        }
                    }
                    future::ok(())
                })
                .or_else(|_e| {
                    // @TODO err handling
                    future::err(())
                });
            */
            let fut = self.load()
                .and_then(move |ud| {
                    emit(&Action::LoadWithAddons(ud.addons.to_vec(), action_load));
                    future::ok(())
                })
                .or_else(|e| {
                    // @TODO proper handling, must emit some sort of a UserMiddlewareError
                    dbg!(e);
                    future::err(())
                });
            T::exec(Box::new(fut));
        }
    }
}
