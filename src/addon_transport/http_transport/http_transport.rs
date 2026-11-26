use std::marker::PhantomData;

use futures::future;
use http::Request;
use once_cell::sync::Lazy;
use percent_encoding::utf8_percent_encode;
use url::Url;

use crate::{
    addon_transport::{http_transport::legacy::AddonLegacyTransport, AddonTransport},
    constants::{ADDON_LEGACY_PATH, ADDON_MANIFEST_PATH, CINEMETA_URL, URI_COMPONENT_ENCODE_SET},
    runtime::{Env, EnvError, EnvFutureExt, TryEnvFuture},
    types::{
        addon::{Manifest, ResourcePath, ResourceResponse},
        query_params_encode,
    },
};

/// Fetches addon data over HTTP or HTTPS using [`Env::fetch`].
///
/// For modern addons, the manifest is fetched from the supplied URL. Resource
/// requests replace the `/manifest.json` suffix of the URL path with
/// `/{resource}/{type}/{id}.json`. With extras, the final segment holds the
/// encoded pairs, as in `/catalog/movie/top/genre=Science%20Fiction.json`.
/// The resource name, type, ID, and extra names and values are percent-encoded.
///
/// A URL path ending in `/stremio/v1` selects the legacy JSON-RPC adapter for
/// both manifest and resource requests. The adapter converts successful
/// responses to the same Rust types and reports JSON-RPC errors as
/// [`EnvError::AddonTransport`].
pub struct AddonHTTPTransport<E: Env> {
    transport_url: Url,
    env: PhantomData<E>,
}

impl<E: Env> AddonHTTPTransport<E> {
    /// Creates a transport from the addon's manifest URL or legacy endpoint URL.
    pub fn new(transport_url: Url) -> Self {
        AddonHTTPTransport {
            transport_url,
            env: PhantomData,
        }
    }
}

impl<E: Env> AddonTransport for AddonHTTPTransport<E> {
    /// Fetches an addon resource.
    ///
    /// # Errors
    ///
    /// For modern resource requests, a URL path without the `/manifest.json`
    /// suffix yields [`EnvError::AddonTransport`]. Fetch and decoding errors
    /// from the environment are propagated.
    fn resource(&self, path: &ResourcePath) -> TryEnvFuture<ResourceResponse> {
        if self.transport_url.path().ends_with(ADDON_LEGACY_PATH) {
            return AddonLegacyTransport::<E>::new(&self.transport_url).resource(path);
        }
        if !self.transport_url.path().ends_with(ADDON_MANIFEST_PATH) {
            return future::err(EnvError::AddonTransport(format!(
                "addon http transport url must ends with {ADDON_MANIFEST_PATH}"
            )))
            .boxed_env();
        }
        let path = if path.extra.is_empty() {
            format!(
                "/{}/{}/{}.json",
                utf8_percent_encode(&path.resource, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&path.id, URI_COMPONENT_ENCODE_SET),
            )
        } else {
            format!(
                "/{}/{}/{}/{}.json",
                utf8_percent_encode(&path.resource, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&path.id, URI_COMPONENT_ENCODE_SET),
                query_params_encode(path.extra.iter().map(|ev| (&ev.name, &ev.value)))
            )
        };

        let mut url = self
            .transport_url
            .as_str()
            .replace(ADDON_MANIFEST_PATH, &path);

        static CINEMETA_ADDONS_CATALOG_URL: Lazy<String> = Lazy::new(|| {
            CINEMETA_URL
                .as_str()
                .replace(ADDON_MANIFEST_PATH, "/addon_catalog/all/community.json")
        });

        match (
            url.clone(),
            std::env::var("CINEMETA_ADDONS_CATALOG_URL")
                .ok()
                .or(option_env!("CINEMETA_ADDONS_CATALOG_URL").map(|slice| slice.to_string()))
                .filter(|env| !env.is_empty()),
        ) {
            (current_url, Some(replace_url)) if url.contains(&*CINEMETA_ADDONS_CATALOG_URL) => {
                let new_url = current_url.replace(&*CINEMETA_ADDONS_CATALOG_URL, &replace_url);
                url = new_url.clone();
                tracing::warn!(
                    current_url = current_url,
                    replace_url = replace_url,
                    new_url = new_url,
                    "Custom cinemeta addons catalog url will be used",
                );
            }
            _ => {}
        }

        let request = Request::get(url).body(()).expect("request builder failed");
        E::fetch(request)
    }
    /// Fetches the manifest from the supplied URL or via the legacy adapter.
    fn manifest(&self) -> TryEnvFuture<Manifest> {
        if self.transport_url.path().ends_with(ADDON_LEGACY_PATH) {
            return AddonLegacyTransport::<E>::new(&self.transport_url).manifest();
        }

        let request = Request::get(self.transport_url.as_str())
            .body(())
            .expect("request builder failed");
        E::fetch(request)
    }
}
