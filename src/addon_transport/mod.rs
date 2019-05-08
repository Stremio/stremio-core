use crate::state_types::{EnvFuture, Environment, Request};
use crate::types::addons::*;
use futures::future;
use std::marker::PhantomData;
//use std::collections::HashMap;

mod legacy;
use self::legacy::AddonLegacyTransport;

pub trait AddonInterface {
    fn get(&self, resource_ref: &ResourceRef) -> EnvFuture<ResourceResponse>;
    fn manifest(&self) -> EnvFuture<Manifest>;
}

pub trait AddonTransport {
    fn get(resource_req: &ResourceRequest) -> EnvFuture<ResourceResponse>;
    fn manifest(url: &str) -> EnvFuture<Manifest>;
}

const MANIFEST_PATH: &str = "/manifest.json";
const LEGACY_PATH: &str = "/stremio/v1";

#[derive(Default)]
pub struct AddonHTTPTransport<T: Environment> {
    pub env: PhantomData<T>,
    //pub extra_addons: HashMap<String, Box<dyn AddonInterface>>,
}
impl<T: Environment> AddonTransport for AddonHTTPTransport<T> {
    fn get(req: &ResourceRequest) -> EnvFuture<ResourceResponse> {
        // This method detects the used transport type, and invokes it
        if req.transport_url.ends_with(MANIFEST_PATH) {
            Self::get_http(req)
        } else if req.transport_url.ends_with(LEGACY_PATH) {
            AddonLegacyTransport::<T>::get(req)
        } else {
            Box::new(future::err("invalid transport_url".into()))
        }
    }
    fn manifest(url: &str) -> EnvFuture<Manifest> {
        // This method detects the used transport type, and invokes it
        if url.ends_with(MANIFEST_PATH) {
            match Request::get(url).body(()) {
                Ok(r) => T::fetch_serde::<_, Manifest>(r),
                Err(e) => Box::new(future::err(e.into())),
            }
        } else if url.ends_with(LEGACY_PATH) {
            AddonLegacyTransport::<T>::manifest(url)
        } else {
            Box::new(future::err("invalid url".into()))
        }
    }
}

impl<T: Environment> AddonHTTPTransport<T> {
    fn get_http(req: &ResourceRequest) -> EnvFuture<ResourceResponse> {
        let url = req
            .transport_url
            .replace(MANIFEST_PATH, &req.resource_ref.to_string());
        match Request::get(&url).body(()) {
            Ok(r) => T::fetch_serde::<_, ResourceResponse>(r),
            Err(e) => Box::new(future::err(e.into())),
        }
    }
}
