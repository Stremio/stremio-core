use super::fetch_api;
use crate::state_types::messages::MsgError;
use crate::state_types::Environment;
use crate::types::api::{APIRequest, Auth, AuthResponse};
use futures::Future;

pub fn authenticate<Env: Environment + 'static>(
    auth_request: &APIRequest,
) -> impl Future<Item = Auth, Error = MsgError> {
    fetch_api::<Env, _, _>(auth_request).map(|AuthResponse { key, user }| Auth { key, user })
}
