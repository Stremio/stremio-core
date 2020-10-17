use crate::runtime::EnvFuture;
use crate::types::addon::{Manifest, ResourcePath, ResourceResponse};

pub trait AddonTransport {
    fn resource(&self, path: &ResourcePath) -> EnvFuture<ResourceResponse>;
    fn manifest(&self) -> EnvFuture<Manifest>;
}
