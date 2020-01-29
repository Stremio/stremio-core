use super::{fetch_api, ModelError};
use crate::state_types::Environment;
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, SuccessResponse};
use futures::Future;

pub fn push_user_addons<Env: Environment + 'static>(
    auth_key: &str,
    addons: &[Descriptor],
) -> impl Future<Item = (), Error = ModelError> {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::AddonCollectionSet {
        auth_key: auth_key.to_owned(),
        addons: addons.to_owned(),
    })
    .map(|_| ())
}
