use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use serde_derive::*;
use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::RefCell;

const USER_DATA_KEY: &str = "userData";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct UserData {
    // @TODO: one option which is either None or Some(authKey + user)
    auth_key: Option<String>,
    user: Option<User>,
    addons: Vec<Descriptor>,
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
    fn load(&mut self) -> EnvFuture<UserData> {
        let current_state = self.state.borrow().to_owned();
        if let Some(ud) = current_state {
            return Box::new(future::ok(ud))
        }

        let state = self.state.clone();
        let fut = T::get_storage(USER_DATA_KEY)
           .and_then(move |result: Option<Box<UserData>>| {
                let ud: UserData = *result.unwrap_or_default();
                *state.borrow_mut() = Some(ud.to_owned());
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
            T::exec(Box::new(fut));
        }
    }
}

// @TODO move those to types/api or something
#[derive(Deserialize, Clone, Debug)]
struct APIErr {
    message: String,
}
// @TODO
#[derive(Serialize, Clone)]
struct CollectionRequest {}
#[derive(Serialize, Deserialize)]
struct CollectionResponse {
    pub addons: Vec<Descriptor>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum APIResult<T> {
    Ok { result: T },
    Err { error: APIErr },
}
