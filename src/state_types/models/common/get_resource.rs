use crate::state_types::msg::MsgError;
use crate::state_types::Environment;
use crate::types::addons::{ResourceRequest, ResourceResponse};
use futures::Future;

pub fn get_resource<Env: Environment + 'static>(
    request: &ResourceRequest,
) -> impl Future<Item = ResourceResponse, Error = MsgError> {
    Env::addon_transport(&request.base)
        .get(&request.path)
        .map_err(MsgError::from)
}
