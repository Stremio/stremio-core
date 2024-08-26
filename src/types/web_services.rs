use crate::{
    runtime::{ConditionalSend, Env, TryEnvFuture},
    types::api::{APIResult, FetchRequestParams},
};


pub mod response;

use http::Request;
use serde::{Deserialize, Serialize};

pub fn fetch<
    E: Env,
    BODY: Serialize + ConditionalSend + 'static,
    REQ: FetchRequestParams<BODY> + Clone + Serialize,
    RESP: for<'de> Deserialize<'de> + ConditionalSend + 'static,
>(
    api_request: &REQ,
) -> TryEnvFuture<APIResult<RESP>> {
    todo!()
}
