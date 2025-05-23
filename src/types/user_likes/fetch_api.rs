use crate::runtime::{ConditionalSend, Env, EnvError, EnvFutureExt, TryEnvFuture};

use serde::Deserialize;

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
