use crate::state_types::Internal::*;
use crate::state_types::*;
use crate::types::addons::*;
use futures::{future, Future};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::addon_transport::*;

#[derive(Default)]
pub struct AddonsMiddleware<T: Environment> {
    env: PhantomData<T>,
    extra_addons: RefCell<HashMap<String, Rc<dyn AddonInterface>>>,
}

impl<T: Environment> Handler for AddonsMiddleware<T> {
    fn handle(&self, msg: &Msg, emit: Rc<DispatcherFn>) {
        match msg {
            Msg::Internal(LoadWithCtx(Context { addons, .. }, action_load)) => {
                if let Some(aggr_req) = action_load.addon_aggr_req() {
                    for resource_req in aggr_req.plan(&addons) {
                        self.for_request(resource_req, emit.clone())
                    }
                }
            }
            Msg::Internal(SetInternalAddon(url, iface)) => {
                self.extra_addons
                    .borrow_mut()
                    .insert(url.to_owned(), iface.clone());
            }
            _ => (),
        }
    }
}
impl<T: Environment> AddonsMiddleware<T> {
    // @TODO loading URLs, collections, etc.
    pub fn new() -> Self {
        AddonsMiddleware {
            env: PhantomData,
            extra_addons: Default::default(),
        }
    }
    fn for_request(&self, req: ResourceRequest, emit: Rc<DispatcherFn>) {
        let url = &req.transport_url;
        let req_fut = match self.extra_addons.borrow().get(url) {
            Some(addon) => addon.get(&req.resource_ref),
            None => AddonHTTPTransport::<T>::from_url(url).get(&req.resource_ref),
        };
        let fut = req_fut.then(move |res| {
            emit(&match res {
                Ok(resp) => Internal::AddonResponse(req, Ok(resp)).into(),
                Err(e) => Internal::AddonResponse(req, Err(e.to_string())).into(),
            });
            future::ok(())
        });
        T::exec(Box::new(fut));
    }
}
