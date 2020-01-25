use crate::state_types::msg::MsgError;
use crate::state_types::{Environment, Request};
use crate::types::api::{APIMethodName, APIResult};
use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn fetch_api<Env, REQ, RESP>(api_request: &REQ) -> impl Future<Item = RESP, Error = MsgError>
where
    Env: Environment + 'static,
    REQ: APIMethodName + Clone + Serialize + 'static,
    RESP: DeserializeOwned + 'static,
{
    let url = format!("{}/api/{}", Env::api_url(), api_request.method_name());
    let request = Request::post(url)
        .body(api_request.to_owned())
        .expect("builder cannot fail");
    Env::fetch_serde::<_, _>(request)
        .map_err(MsgError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => Ok(result),
            APIResult::Err { error } => Err(MsgError::from(error)),
        })
}
