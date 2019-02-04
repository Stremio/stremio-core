use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use serde_derive::*;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Default)]
pub struct UserMiddleware<T: Environment> {
    //id: usize,
    pub user: Option<String>,
    pub env: PhantomData<T>,
}
impl<T: Environment> UserMiddleware<T> {
    pub fn new() -> Self {
        UserMiddleware {
            user: None,
            env: PhantomData,
        }
    }
}
impl<T: Environment> Handler for UserMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // only handle the Init
        // @TODO handle LoadCatalogs
        match action {
            Action::Load(_) => {}
            _ => return,
        }
        let action = action.to_owned();
        // @TODO find a way to implicitly unwrap the .result field
        // @TODO APIWrapper, err handling
        #[derive(Serialize, Clone)]
        struct APICollectionRequest {};
        #[derive(Serialize, Deserialize)]
        struct APIRes<T> {
            pub result: T,
        };
        #[derive(Serialize, Deserialize)]
        struct APICollectionRes {
            pub addons: Vec<Descriptor>,
        };
        // @TODO get rid of this hardcode
        let req = Request::post("https://api.strem.io/api/addonCollectionGet")
            .body(APICollectionRequest {})
            .unwrap();
        let fut = T::fetch_serde::<_, APIRes<APICollectionRes>>(req)
            .and_then(move |r| {
                let addons = &r.result.addons;
                emit(&Action::WithAddons(addons.to_vec(), action.into()));
                future::ok(())
            })
            // @TODO handle the error
            .or_else(|_| future::err(()));
        T::exec(Box::new(fut));
    }
}
