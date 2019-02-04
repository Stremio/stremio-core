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
    fn for_request(&self, res_req: &ResourceRequest, emit: Rc<DispatcherFn>) {
        // @TODO use transport
        // @TODO: better identifier?
        let url = res_req.transport_url.replace(
            "/manifest.json",
            &format!("/catalog/{}/{}.json", res_req.resource_ref.type_name, res_req.resource_ref.id),
        );
        let res_req = res_req.to_owned();
        let req = Request::get(&url).body(()).unwrap();
        let fut = T::fetch_serde::<_, CatalogResponse>(req).then(move |res| {
            emit(&match res {
                Ok(resp) => Action::AddonResponse(res_req, Ok(*resp)),
                Err(e) => Action::AddonResponse(res_req, Err(e.description().to_owned())),
            });
            future::ok(())
        });
        T::exec(Box::new(fut));
    }
}
impl<T: Environment> Handler for AddonsMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // @TODO can we avoid the identation
        if let Action::LoadWithAddons(addons, action_load) = action {
            if let Some(aggr_req) = action_load.addon_aggr_req() {
                for resource_req in aggr_req.plan(&addons).iter() {
                    self.for_request(&resource_req, emit.clone())
                }
            }
        }
    }
}
