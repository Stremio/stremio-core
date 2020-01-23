use super::fetch_api;
use crate::state_types::messages::MsgError;
use crate::state_types::Environment;
use crate::types::api::{APIRequest, AuthKey, SuccessResponse};
use futures::Future;

pub fn delete_user_session<Env: Environment + 'static>(
    auth_key: &AuthKey,
) -> impl Future<Item = (), Error = MsgError> {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::Logout {
        auth_key: auth_key.to_owned(),
    })
    .map(|_| ())
}
