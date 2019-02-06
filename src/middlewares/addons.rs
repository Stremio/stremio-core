use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Default)]
pub struct AddonsMiddleware<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonsMiddleware<T> {
    pub fn new() -> Self {
        AddonsMiddleware { env: PhantomData }
    }
    fn for_request(&self, resource_req: ResourceRequest, emit: Rc<DispatcherFn>) {
        // @TODO use transport
        // @TODO: better identifier?
        let url = resource_req.transport_url.replace(
            "/manifest.json",
            &format!(
                "/catalog/{}/{}.json",
                &resource_req.resource_ref.type_name, &resource_req.resource_ref.id
            ),
        );
        let req = Request::get(&url).body(()).unwrap();
        let fut = T::fetch_serde::<_, CatalogResponse>(req).then(move |res| {
            emit(&match res {
                Ok(resp) => Action::AddonResponse(resource_req, Ok(*resp)),
                Err(e) => Action::AddonResponse(resource_req, Err(e.to_string())),
            });
            future::ok(())
        });
        T::exec(Box::new(fut));
    }
}
impl<T: Environment> Handler for AddonsMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        if let Action::LoadWithAddons(addons, action_load) = action {
            if let Some(aggr_req) = action_load.addon_aggr_req() {
                for resource_req in aggr_req.plan(&addons) {
                    self.for_request(resource_req, emit.clone())
                }
            }
        }
    }
}
