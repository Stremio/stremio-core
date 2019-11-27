use super::{APIMethodName, APIResult};
use crate::state_types::messages::CtxError;
use crate::state_types::{Environment, Request};
use futures::{future, Future};

pub fn api_fetch<Env, OUT, REQ>(req: REQ) -> impl Future<Item = OUT, Error = CtxError>
where
    Env: Environment,
    OUT: serde::de::DeserializeOwned + 'static,
    REQ: APIMethodName + serde::Serialize + 'static,
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
