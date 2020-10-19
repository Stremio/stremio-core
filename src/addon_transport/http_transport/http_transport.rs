use crate::addon_transport::http_transport::legacy::AddonLegacyTransport;
use crate::addon_transport::AddonTransport;
use crate::constants::{ADDON_LEGACY_PATH, ADDON_MANIFEST_PATH};
use crate::runtime::{Env, EnvError, EnvFuture};
use crate::types::addon::{Manifest, ResourcePath, ResourceResponse};
use futures::{future, FutureExt};
use http::Request;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::marker::PhantomData;
use url::{form_urlencoded, Url};

pub struct AddonHTTPTransport<E: Env> {
    transport_url: Url,
    env: PhantomData<E>,
}

impl<E: Env> AddonHTTPTransport<E> {
    pub fn new(transport_url: Url) -> Self {
        AddonHTTPTransport {
            transport_url,
            env: PhantomData,
        }
    }
}

impl<E: Env> AddonTransport for AddonHTTPTransport<E> {
    fn resource(&self, path: &ResourcePath) -> EnvFuture<ResourceResponse> {
        if self.transport_url.path().ends_with(ADDON_LEGACY_PATH) {
            return AddonLegacyTransport::<E>::new(&self.transport_url).resource(&path);
        }
        if !self.transport_url.path().ends_with(ADDON_MANIFEST_PATH) {
            return future::err(EnvError::AddonTransport(format!(
                "addon http transport url must ends with {}",
                ADDON_MANIFEST_PATH
            )))
            .boxed_local();
        }
        let path = if path.extra.is_empty() {
            format!(
                "/{}/{}/{}.json",
                utf8_percent_encode(&path.resource, NON_ALPHANUMERIC),
                utf8_percent_encode(&path.r#type, NON_ALPHANUMERIC),
                utf8_percent_encode(&path.id, NON_ALPHANUMERIC),
            )
        } else {
            format!(
                "/{}/{}/{}/{}.json",
                utf8_percent_encode(&path.resource, NON_ALPHANUMERIC),
                utf8_percent_encode(&path.r#type, NON_ALPHANUMERIC),
                utf8_percent_encode(&path.id, NON_ALPHANUMERIC),
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(path.extra.iter().map(|ev| (&ev.name, &ev.value)))
                    .finish()
            )
        };
        let url = self
            .transport_url
            .as_str()
            .replace(ADDON_MANIFEST_PATH, &path);
        let request = Request::get(&url).body(()).expect("request builder failed");
        E::fetch(request)
    }
    fn manifest(&self) -> EnvFuture<Manifest> {
        if self.transport_url.path().ends_with(ADDON_LEGACY_PATH) {
            return AddonLegacyTransport::<E>::new(&self.transport_url).manifest();
        }

        let request = Request::get(self.transport_url.as_str())
            .body(())
            .expect("request builder failed");
        E::fetch(request)
    }
}
