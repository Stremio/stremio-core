use crate::state_types::{EnvFuture, Environment, Request};
use crate::types::addons::{ResourceRequest, ResourceResponse};
use futures::future;
use std::marker::PhantomData;

mod legacy;
use self::legacy::AddonLegacyTransport;

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
    fn get(req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        // This method detects the used transport type, and invokes it
        if req.transport_url.ends_with(MANIFEST_PATH) {
            Self::get_http(req)
        } else if req.transport_url.ends_with(LEGACY_PATH) {
            AddonLegacyTransport::<T>::get(req)
        } else {
            Box::new(future::err("invalid transport_url".into()))
        }
    }
}

impl<T: Environment> AddonHTTPTransport<T> {
    fn get_http(req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        let url = req
            .transport_url
            .replace(MANIFEST_PATH, &req.resource_ref.to_string());
        match Request::get(&url).body(()) {
            Ok(r) => T::fetch_serde::<_, ResourceResponse>(r),
            Err(e) => Box::new(future::err(e.into())),
        }
    }
}
