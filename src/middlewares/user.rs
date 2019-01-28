use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use std::marker::PhantomData;
use std::rc::Rc;
use serde_derive::*;

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
            Action::Init => {}
            _ => return,
        }
        let action = action.to_owned();
        // @TODO find a way to implicitly unwrap the .result field
        // @TODO APIWrapper, err handling
        #[derive(Serialize,Clone)]
        struct APICollectionRequest {};
        #[derive(Serialize,Deserialize)]
        struct APIRes<T> { pub result: T };
        #[derive(Serialize,Deserialize)]
        struct APICollectionRes { pub addons: Vec<AddonDescriptor> };
        // @TODO get rid of this hardcode
        let req = Request::post("https://api.strem.io/api/addonCollectionGet")
            .body(APICollectionRequest{})
            .unwrap();
        let fut = T::fetch_serde::<_, APIRes<APICollectionRes>>(req)
            .and_then(move |r| {
                let addons = &r.result.addons;
                // @TODO Should we have an Into Box on action, so we can write this
                // as .clone().into() ?
                emit(&Action::WithAddons(addons.to_vec(), Box::new(action)));
                future::ok(())
            })
            // @TODO handle the error
            .or_else(|_| future::err(()));
        T::exec(Box::new(fut));
    }
}
