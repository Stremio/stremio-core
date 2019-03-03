use crate::state_types::*;
use crate::types::addons::*;
use futures::{future, Future};
use std::marker::PhantomData;
use std::rc::Rc;

// AddonHTTPTransport supports the v3 addon protocol and has a legacy adapter
use crate::addon_transport::{AddonHTTPTransport, AddonTransport};

#[derive(Default)]
pub struct AddonsMiddleware<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> Handler for AddonsMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        if let Action::LoadWithUser(_, addons, action_load) = action {
            if let Some(aggr_req) = action_load.addon_aggr_req() {
                for resource_req in aggr_req.plan(&addons) {
                    Self::for_request(resource_req, emit.clone())
                }
            }
        }
    }
}
impl<T: Environment> AddonsMiddleware<T> {
    // @TODO loading URLs, collections, etc.
    pub fn new() -> Self {
        AddonsMiddleware { env: PhantomData }
    }
    fn for_request(resource_req: ResourceRequest, emit: Rc<DispatcherFn>) {
        let fut = AddonHTTPTransport::<T>::get(&resource_req).then(move |res| {
            emit(&match res {
                Ok(resp) => Action::AddonResponse(resource_req, Ok(*resp)),
                Err(e) => Action::AddonResponse(resource_req, Err(e.to_string())),
            });
            future::ok(())
        });
        T::exec(Box::new(fut));
    }
}
