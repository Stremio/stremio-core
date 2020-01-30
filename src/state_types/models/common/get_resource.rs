use crate::state_types::{EnvError, Environment};
use crate::types::addons::{ResourceRequest, ResourceResponse};
use futures::Future;

pub fn get_resource<Env: Environment + 'static>(
    request: &ResourceRequest,
) -> impl Future<Item = ResourceResponse, Error = EnvError> {
    Env::addon_transport(&request.base).get(&request.path)
}
