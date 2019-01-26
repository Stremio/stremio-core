use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
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
            Action::Init => {}
            _ => return,
        }
        let action_owned = action.to_owned();
        // @TODO get rid of this hardcode
        let url = "https://api.strem.io/addonsofficialcollection.json";
        let fut = T::fetch_serde::<(), Vec<AddonDescriptor>>(
            &Request::builder().uri(url).body(()).unwrap(),
        )
        .and_then(move |addons| {
            // @TODO Should we have an Into Box on action, so we can write this
            // as .clone().into() ?
            emit(&Action::WithAddons(addons.to_vec(), Box::new(action_owned)));
            future::ok(())
        })
        // @TODO handle the error
        .or_else(|_| future::err(()));
        T::exec(Box::new(fut));
    }
}
