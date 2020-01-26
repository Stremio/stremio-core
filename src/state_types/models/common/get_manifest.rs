use super::ModelError;
use crate::state_types::Environment;
use crate::types::addons::Manifest;
use futures::Future;

pub fn get_manifest<Env: Environment + 'static>(
    transport_url: &str,
) -> impl Future<Item = Manifest, Error = ModelError> {
    Env::addon_transport(transport_url)
        .manifest()
        .map_err(ModelError::from)
}
