use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use std::marker::PhantomData;
use std::rc::Rc;
use url::form_urlencoded;

#[derive(Default)]
pub struct AddonsMiddleware<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonsMiddleware<T> {
    pub fn new() -> Self {
        AddonsMiddleware { env: PhantomData }
    }
    fn for_request(&self, resource_req: ResourceRequest, emit: Rc<DispatcherFn>) {
        // @TODO use transport abstraction
        let url_pathname = if resource_req.resource_ref.extra.is_empty() {
            let mut extra_encoded = form_urlencoded::Serializer::new(String::new());
            for (k, v) in resource_req.resource_ref.extra.iter() {
                extra_encoded.append_pair(&k, &v);
            }
            format!(
                "/catalog/{}/{}/{}.json",
                &resource_req.resource_ref.type_name,
                &resource_req.resource_ref.id,
                &extra_encoded.finish()
            )
        } else {
            format!(
                "/catalog/{}/{}.json",
                &resource_req.resource_ref.type_name, &resource_req.resource_ref.id
            )
        };
        let url = resource_req
            .transport_url
            .replace("/manifest.json", &url_pathname);
        let req = Request::get(&url).body(()).unwrap();
        let fut = T::fetch_serde::<_, ResourceResponse>(req).then(move |res| {
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
