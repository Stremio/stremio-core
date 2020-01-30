use crate::state_types::{EnvError, Environment};
use crate::types::addons::Manifest;
use futures::Future;

pub fn get_manifest<Env: Environment + 'static>(
    transport_url: &str,
) -> impl Future<Item = Manifest, Error = EnvError> {
    Env::addon_transport(transport_url).manifest()
}
