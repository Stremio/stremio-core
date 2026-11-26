use crate::{
    runtime::TryEnvFuture,
    types::addon::{Manifest, ResourcePath, ResourceResponse},
};

/// Fetches and decodes data from a single addon.
///
/// Both methods return a future that resolves to a decoded value or an
/// [`EnvError`](crate::runtime::EnvError).
pub trait AddonTransport {
    /// Fetches the resource identified by its name, content type, ID and optional extras.
    fn resource(&self, path: &ResourcePath) -> TryEnvFuture<ResourceResponse>;

    /// Fetches the manifest describing the resources and catalogs the addon offers.
    fn manifest(&self) -> TryEnvFuture<Manifest>;
}
