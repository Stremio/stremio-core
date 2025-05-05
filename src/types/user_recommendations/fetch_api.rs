use crate::{
    runtime::{ConditionalSend, Env, EnvError, EnvFutureExt, TryEnvFuture},
    types::api::APIResult,
};

use http::Request;
use serde::{Deserialize, Serialize};

use super::{APIRequest, RequestParameters};

pub trait FetchApi: RequestParameters<Option<serde_json::Value>> + Clone + 'static {
    fn fetch_api<E: Env, RESP: for<'de> Deserialize<'de> + ConditionalSend + 'static>(
        &self,
    ) -> TryEnvFuture<RESP> {
        let self_clone = self.clone();
        async move {
            let request = self_clone
                .build()
                .map_err(|err| EnvError::Other(err.to_string()))?;

            let response = E::fetch::<_, RESP>(request).await?;

            Ok(response)
        }
        .boxed_env()
    }
}

impl FetchApi for APIRequest {}

// pub fn fetch_api<
//     E: Env,
//     // BODY: Serialize + ConditionalSend + 'static,
//     REQ: RequestParameters<BODY> + Clone + Serialize,
//     RESP: for<'de> Deserialize<'de> + ConditionalSend + 'static,
// >(
//     api_request: &REQ,
// ) -> TryEnvFuture<serde_json::Result<RESP>> {
//     async {
//         let request = api_request.build()?;
//         // let request = Request::builder()
//         // .method(api_request.method())
//         // .uri(url.as_str())
//         // .body(api_request.to_owned().body())
//         // .expect("request builder failed");
//         E::fetch::<_, RESP>(request)
//     }
//     .boxed_env()
// }
