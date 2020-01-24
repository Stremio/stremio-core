use crate::state_types::msg::MsgError;
use crate::state_types::Environment;
use crate::types::addons::Manifest;
use futures::Future;

pub fn get_manifest<Env: Environment + 'static>(
    transport_url: &str,
) -> impl Future<Item = Manifest, Error = MsgError> {
    Env::addon_transport(transport_url)
        .manifest()
        .map_err(MsgError::from)
}
