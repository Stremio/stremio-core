use super::fetch_api;
use crate::state_types::messages::MsgError;
use crate::state_types::Environment;
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, AuthKey, CollectionResponse};
use futures::Future;

pub fn get_addons<Env: Environment + 'static>(
    auth_key: &AuthKey,
) -> impl Future<Item = Vec<Descriptor>, Error = MsgError> {
    fetch_api::<Env, _>(&APIRequest::AddonCollectionGet {
        auth_key: auth_key.to_owned(),
        update: true,
    })
    .map(|CollectionResponse { addons, .. }| addons)
}
