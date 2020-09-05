use crate::state_types::EnvFuture;
use crate::types::addon::{Manifest, ResourceRef, ResourceResponse};

pub trait AddonTransport {
    fn resource(&self, path: &ResourceRef) -> EnvFuture<ResourceResponse>;
    fn manifest(&self) -> EnvFuture<Manifest>;
}
