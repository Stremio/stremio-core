use crate::state_types::messages::MsgError;
use crate::state_types::{Environment, Request};
use crate::types::api::{APIMethodName, APIResult};
use futures::{future, Future};
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
        .map_err(|error| MsgError::from(error))
        .and_then(|result| match result {
            APIResult::Ok { result } => future::ok(result),
            APIResult::Err { error } => future::err(MsgError::from(error)),
        })
}
