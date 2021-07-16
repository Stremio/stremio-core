use crate::runtime::TryEnvFuture;
use crate::types::addon::{Manifest, ResourcePath, ResourceResponse};

pub trait AddonTransport {
    fn resource(&self, path: &ResourcePath) -> TryEnvFuture<ResourceResponse>;
    fn manifest(&self) -> TryEnvFuture<Manifest>;
}
