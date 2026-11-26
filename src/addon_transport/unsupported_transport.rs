use futures::future;
use url::Url;

use crate::{
    addon_transport::AddonTransport,
    runtime::{ConditionalSend, EnvError, EnvFutureExt, TryEnvFuture},
    types::addon::{Manifest, ResourcePath, ResourceResponse},
};

/// Reports an unsupported URL scheme for both manifest and resource requests.
///
/// Both requests resolve to [`EnvError::AddonTransport`] without fetching data.
pub struct UnsupportedTransport {
    transport_url: Url,
}

impl UnsupportedTransport {
    /// Creates an error transport for the given URL.
    pub fn new(transport_url: Url) -> Self {
        UnsupportedTransport { transport_url }
    }
    fn result<T: Sized + ConditionalSend + 'static>(&self) -> TryEnvFuture<T> {
        future::err(EnvError::AddonTransport(format!(
            "Unsupported addon transport: {}",
            self.transport_url.scheme()
        )))
        .boxed_env()
    }
}

impl AddonTransport for UnsupportedTransport {
    fn resource(&self, _path: &ResourcePath) -> TryEnvFuture<ResourceResponse> {
        self.result::<ResourceResponse>()
    }
    fn manifest(&self) -> TryEnvFuture<Manifest> {
        self.result::<Manifest>()
    }
}
