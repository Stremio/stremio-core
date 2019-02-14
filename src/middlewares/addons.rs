use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use std::marker::PhantomData;
use std::rc::Rc;
use url::form_urlencoded;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub trait AddonImpl {
    fn get(resource_req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>>;
}

#[derive(Default)]
pub struct AddonHTTPTransport<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonImpl for AddonHTTPTransport<T> {
    fn get(
        ResourceRequest {
            resource_ref,
            transport_url,
        }: &ResourceRequest,
    ) -> EnvFuture<Box<ResourceResponse>> {
        let url_pathname = if !resource_ref.extra.is_empty() {
            let mut extra_encoded = form_urlencoded::Serializer::new(String::new());
            for (k, v) in resource_ref.extra.iter() {
                extra_encoded.append_pair(&k, &v);
            }
            format!(
                "/{}/{}/{}/{}.json",
                &utf8_percent_encode(&resource_ref.resource, DEFAULT_ENCODE_SET),
                &utf8_percent_encode(&resource_ref.type_name, DEFAULT_ENCODE_SET),
                &utf8_percent_encode(&resource_ref.id, DEFAULT_ENCODE_SET),
                &extra_encoded.finish()
            )
        } else {
            format!(
                "/{}/{}/{}.json",
                &utf8_percent_encode(&resource_ref.resource, DEFAULT_ENCODE_SET),
                &utf8_percent_encode(&resource_ref.type_name, DEFAULT_ENCODE_SET),
                &utf8_percent_encode(&resource_ref.id, DEFAULT_ENCODE_SET)
            )
        };
        let url = transport_url.replace("/manifest.json", &url_pathname);
        // Building a request might fail, if the addon URL is malformed
        match Request::get(&url).body(()) {
            Ok(req) => T::fetch_serde::<_, ResourceResponse>(req),
            Err(e) => Box::new(future::err(e.into())),
        }
    }
}

#[derive(Default)]
pub struct AddonsMiddleware<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonsMiddleware<T> {
    // @TODO loading URLs, collections, etc.
    pub fn new() -> Self {
        AddonsMiddleware { env: PhantomData }
    }
    fn for_request(&self, resource_req: ResourceRequest, emit: Rc<DispatcherFn>) {
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
