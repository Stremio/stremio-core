use crate::state_types::messages::MsgError;
use crate::state_types::{Environment, Request};
use crate::types::api::{APIMethodName, APIResult};
use futures::{future, Future};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn api_fetch<Env, OUT, REQ>(req: REQ) -> impl Future<Item = OUT, Error = MsgError>
where
    Env: Environment,
    OUT: DeserializeOwned + 'static,
    REQ: APIMethodName + Serialize + 'static,
{
    let url = format!("{}/api/{}", Env::api_url(), req.method_name());
    let req = Request::post(url).body(req).expect("builder cannot fail");
    Env::fetch_serde::<_, APIResult<OUT>>(req)
        .map_err(Into::into)
        .and_then(|res| match res {
            APIResult::Err { error } => future::err(error.into()),
            APIResult::Ok { result } => future::ok(result),
        })
}
