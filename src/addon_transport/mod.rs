use crate::state_types::{EnvFuture, Environment, Request};
use crate::types::addons::{ResourceRef, ResourceRequest, ResourceResponse};
use futures::future;
use std::convert::Into;
use std::error::Error;
use std::marker::PhantomData;

// @TODO facilitate detect from URL (manifest)
pub trait AddonTransport {
    fn get(resource_req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>>;
}

const MANIFEST_PATH: &str = "/manifest.json";
const LEGACY_PATH: &str = "/stremio/v1";
#[derive(Default)]
pub struct AddonHTTPTransport<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonTransport for AddonHTTPTransport<T> {
    fn get(resource_req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        // @TODO depending on whether it's legacy or not, the response will be handled differently
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
    Request::get(&url).body(()).map_err(Into::into)
}
fn build_legacy_url_path(resource_ref: &ResourceRef) -> Result<String, Box<dyn Error>> {
    match &resource_ref.resource as &str {
        "catalog" => {}
        "meta" => {}
        "streams" => {}
        // @TODO better error
        _ => return Err("legacy transport: unsupported resource".into()),
    }
    Ok("".to_owned())
}
