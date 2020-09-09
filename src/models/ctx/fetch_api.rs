use crate::constants::API_URL;
use crate::models::ctx::CtxError;
use crate::runtime::Environment;
use crate::types::api::{APIMethodName, APIResult};
use futures::{future, Future, TryFutureExt};
use http::Request;
use serde::{Deserialize, Serialize};

pub fn fetch_api<Env, REQ, RESP>(api_request: &REQ) -> impl Future<Output = Result<RESP, CtxError>>
where
    Env: Environment + 'static,
    REQ: APIMethodName + Clone + Serialize + 'static,
    for<'de> RESP: Deserialize<'de> + 'static,
{
    let url = API_URL
        .join("api/")
        .expect("url builder failed")
        .join(api_request.method_name())
        .expect("url builder failed");
    let request = Request::post(url.as_str())
        .body(api_request.to_owned())
        .expect("request builder failed");
    Env::fetch::<_, _>(request)
        .map_err(CtxError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => future::ready(Ok(result)),
            APIResult::Err { error } => future::ready(Err(CtxError::from(error))),
        })
}
