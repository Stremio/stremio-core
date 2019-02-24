use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use std::error::Error;
use std::marker::PhantomData;
use std::rc::Rc;

// @TODO facilitate detect from URL (manifest)
pub trait AddonTransport {
    fn get(resource_req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>>;
}

// @TODO move the transport out of here
// @TODO multi-transports, detection
const MANIFEST_PATH: &str = "/manifest.json";
const LEGACY_PATH: &str = "/stremio/v1";
#[derive(Default)]
pub struct AddonHTTPTransport<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonTransport for AddonHTTPTransport<T> {
    fn get(resource_req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        match try_build_request(resource_req) {
            Ok(req) => T::fetch_serde::<_, ResourceResponse>(req),
            Err(e) => Box::new(future::err(e)),
        }
    }
}
fn try_build_request(
    ResourceRequest {
        resource_ref,
        transport_url,
    }: &ResourceRequest,
) -> Result<Request<()>, Box<dyn Error>> {
    let url = if transport_url.ends_with(MANIFEST_PATH) {
        transport_url.replace(MANIFEST_PATH, &resource_ref.to_string())
    } else if transport_url.ends_with(LEGACY_PATH) {
        format!(
            "{}/{}",
            &transport_url,
            build_legacy_url_path(&resource_ref)?
        )
    } else {
        // @TODO better errors
        return Err("invalid transport_url".into());
    };
    // Building a request might fail, if the addon URL is malformed
    Request::get(&url).body(()).map_err(|e| e.into())
}
fn build_legacy_url_path(resource_ref: &ResourceRef) -> Result<String, Box<dyn Error>> {
    match &resource_ref.resource as &str {
        "catalog" => {},
        "meta" => {},
        "streams" => {},
        // @TODO better error
        _ => return Err("legacy transport: unsupported resource".into())
    }
    Ok("".to_owned())
}

#[derive(Default)]
pub struct AddonsMiddleware<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> Handler for AddonsMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        if let Action::LoadWithUser(_, addons, action_load) = action {
            if let Some(aggr_req) = action_load.addon_aggr_req() {
                for resource_req in aggr_req.plan(&addons) {
                    self.for_request(resource_req, emit.clone())
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
    fn for_request(&self, resource_req: ResourceRequest, emit: Rc<DispatcherFn>) {
        // @TODO a bunch of transports, detect them here
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

