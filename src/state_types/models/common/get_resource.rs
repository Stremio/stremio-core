use super::ModelError;
use crate::state_types::Environment;
use crate::types::addons::{ResourceRequest, ResourceResponse};
use futures::Future;

pub fn get_resource<Env: Environment + 'static>(
    request: &ResourceRequest,
) -> impl Future<Item = ResourceResponse, Error = ModelError> {
    Env::addon_transport(&request.base)
        .get(&request.path)
        .map_err(ModelError::from)
}
