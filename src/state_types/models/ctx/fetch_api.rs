use crate::state_types::models::ctx::CtxError;
use crate::state_types::{Environment, Request};
use crate::types::api::{APIMethodName, APIResult};
use futures::Future;
use serde::{Deserialize, Serialize};

pub fn fetch_api<Env, REQ, RESP>(api_request: &REQ) -> impl Future<Item = RESP, Error = CtxError>
where
    Env: Environment + 'static,
    REQ: APIMethodName + Clone + Serialize + 'static,
    for<'de> RESP: Deserialize<'de> + 'static,
{
    let url = format!("{}/api/{}", Env::api_url(), api_request.method_name());
    let request = Request::post(url)
        .body(api_request.to_owned())
        .expect("fetch_api request builder cannot fail");
    Env::fetch::<_, _>(request)
        .map_err(CtxError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => Ok(result),
            APIResult::Err { error } => Err(CtxError::from(error)),
        })
}
