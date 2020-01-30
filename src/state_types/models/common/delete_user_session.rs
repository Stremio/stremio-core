use super::fetch_api;
use crate::state_types::models::ctx::error::CtxError;
use crate::state_types::Environment;
use crate::types::api::{APIRequest, SuccessResponse};
use futures::Future;

pub fn delete_user_session<Env: Environment + 'static>(
    auth_key: &str,
) -> impl Future<Item = (), Error = CtxError> {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::Logout {
        auth_key: auth_key.to_owned(),
    })
    .map(|_| ())
}
