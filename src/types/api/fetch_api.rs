use crate::constants::API_URL;
use crate::runtime::{Env, TryEnvFuture};
use crate::types::api::{APIMethodName, APIResult};
use http::Request;
use serde::{Deserialize, Serialize};

pub fn fetch_api<
    E: Env,
    REQ: APIMethodName + Clone + Serialize,
    #[cfg(target_arch = "wasm32")] RESP: for<'de> Deserialize<'de> + 'static,
    #[cfg(not(target_arch = "wasm32"))] RESP: for<'de> Deserialize<'de> + Send + 'static,
>(
    api_request: &REQ,
) -> TryEnvFuture<APIResult<RESP>> {
    let url = API_URL
        .join("api/")
        .expect("url builder failed")
        .join(api_request.method_name())
        .expect("url builder failed");
    let request = Request::post(url.as_str())
        .body(api_request.to_owned())
        .expect("request builder failed");
    E::fetch::<_, _>(request)
}
