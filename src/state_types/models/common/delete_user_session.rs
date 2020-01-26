use super::{fetch_api, ModelError};
use crate::state_types::Environment;
use crate::types::api::{APIRequest, SuccessResponse};
use futures::Future;

pub fn delete_user_session<Env: Environment + 'static>(
    auth_key: &str,
) -> impl Future<Item = (), Error = ModelError> {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::Logout {
        auth_key: auth_key.to_owned(),
    })
    .map(|_| ())
}
