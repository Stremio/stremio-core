use crate::addon_transport::AddonTransport;
use crate::runtime::{EnvError, TryEnvFuture};
use crate::types::addon::{Manifest, ResourcePath, ResourceResponse};
use futures::{future, FutureExt};
use url::Url;

pub struct UnsupportedTransport {
    transport_url: Url,
}

impl UnsupportedTransport {
    pub fn new(transport_url: Url) -> Self {
        UnsupportedTransport { transport_url }
    }
    fn result<T: 'static>(&self) -> TryEnvFuture<T> {
        future::err(EnvError::AddonTransport(format!(
            "Unsupported addon transport: {}",
            self.transport_url.scheme()
        )))
        .boxed_local()
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
