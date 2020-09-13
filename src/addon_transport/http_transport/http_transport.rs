use crate::addon_transport::http_transport::legacy::AddonLegacyTransport;
use crate::addon_transport::AddonTransport;
use crate::constants::{ADDON_LEGACY_PATH, ADDON_MANIFEST_PATH};
use crate::runtime::{EnvError, EnvFuture, Environment};
use crate::types::addon::{Manifest, ResourceRef, ResourceResponse};
use futures::{future, FutureExt};
use http::Request;
use std::marker::PhantomData;
use url::Url;

pub struct AddonHTTPTransport<Env: Environment> {
    transport_url: Url,
    env: PhantomData<Env>,
}

impl<Env: Environment> AddonHTTPTransport<Env> {
    pub fn new(transport_url: Url) -> Self {
        AddonHTTPTransport {
            transport_url,
            env: PhantomData,
        }
    }
}

impl<Env: Environment> AddonTransport for AddonHTTPTransport<Env> {
    fn resource(&self, path: &ResourceRef) -> EnvFuture<ResourceResponse> {
        if self.transport_url.path().ends_with(ADDON_LEGACY_PATH) {
            return AddonLegacyTransport::<Env>::new(&self.transport_url).resource(&path);
        }

        if !self.transport_url.path().ends_with(ADDON_MANIFEST_PATH) {
            return future::err(EnvError::AddonTransport(format!(
                "addon http transport url must ends with {}",
                ADDON_MANIFEST_PATH
            )))
            .boxed_local();
        }

        let url = self
            .transport_url
            .as_str()
            .replace(ADDON_MANIFEST_PATH, &path.to_string());
        let request = Request::get(&url).body(()).expect("request builder failed");
        Env::fetch(request)
    }
    fn manifest(&self) -> EnvFuture<Manifest> {
        if self.transport_url.path().ends_with(ADDON_LEGACY_PATH) {
            return AddonLegacyTransport::<Env>::new(&self.transport_url).manifest();
        }

        let request = Request::get(self.transport_url.as_str())
            .body(())
            .expect("request builder failed");
        Env::fetch(request)
    }
}
