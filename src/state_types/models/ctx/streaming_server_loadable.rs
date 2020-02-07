use super::error::CtxError;
use crate::state_types::{EnvError, Environment};
use crate::types::api::SuccessResponse;
use derivative::Derivative;
use futures::Future;
use http::request::Request;
use http::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
use url_serde;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub app_path: String,
    pub cache_root: String,
    pub server_version: String,
    pub cache_size: Option<f64>,
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: u64,
}

#[derive(Derivative, Debug, Clone, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type")]
pub enum StreamingServerLoadable {
    #[derivative(Default)]
    NotLoaded,
    Loading {
        #[serde(with = "url_serde")]
        url: Url,
    },
    Error {
        #[serde(with = "url_serde")]
        url: Url,
        error: String,
    },
    Ready {
        settings: Settings,
        #[serde(with = "url_serde")]
        base_url: Url,
        #[serde(with = "url_serde")]
        url: Url,
    },
}

impl StreamingServerLoadable {
    pub fn url(&self) -> Option<&Url> {
        match &self {
            StreamingServerLoadable::NotLoaded => None,
            StreamingServerLoadable::Loading { url }
            | StreamingServerLoadable::Error { url, .. }
            | StreamingServerLoadable::Ready { url, .. } => Some(url),
        }
    }
    pub fn update_settings<Env: Environment + 'static>(
        url: &Url,
        settings: &Settings,
    ) -> impl Future<Item = (), Error = CtxError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            pub cache_size: Option<f64>,
            pub bt_max_connections: u64,
            pub bt_handshake_timeout: u64,
            pub bt_request_timeout: u64,
            pub bt_download_speed_soft_limit: f64,
            pub bt_download_speed_hard_limit: f64,
            pub bt_min_peers_for_stable: u64,
        }
        let body = Body {
            cache_size: settings.cache_size.to_owned(),
            bt_max_connections: settings.bt_max_connections.to_owned(),
            bt_handshake_timeout: settings.bt_handshake_timeout.to_owned(),
            bt_request_timeout: settings.bt_request_timeout.to_owned(),
            bt_download_speed_soft_limit: settings.bt_download_speed_soft_limit.to_owned(),
            bt_download_speed_hard_limit: settings.bt_download_speed_hard_limit.to_owned(),
            bt_min_peers_for_stable: settings.bt_min_peers_for_stable.to_owned(),
        };
        request::<Env, _, SuccessResponse>(
            &url,
            "settings",
            Method::POST,
            [("content-type", "application/json")]
                .iter()
                .cloned()
                .collect(),
            body,
        )
        .then(|result| result.map(|_| ()))
        .map_err(CtxError::from)
    }
    pub fn load<Env: Environment + 'static>(
        url: &Url,
    ) -> impl Future<Item = (Url, Settings), Error = CtxError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Resp {
            pub values: Settings,
            #[serde(with = "url_serde")]
            pub base_url: Url,
        }
        request::<Env, _, Resp>(&url, "settings", Method::GET, HashMap::new(), ())
            .then(|result| result.map(|resp| (resp.base_url, resp.values)))
            .map_err(CtxError::from)
    }
}

fn request<Env, Body, Resp>(
    url: &Url,
    route: &str,
    method: Method,
    headers: HashMap<&str, &str>,
    body: Body,
) -> impl Future<Item = Resp, Error = EnvError>
where
    Env: Environment + 'static,
    Body: Serialize + 'static,
    Resp: DeserializeOwned + 'static,
{
    // TODO refactor this code! it looks ugly because of http::request::Builder
    let endpoint = url.join(route).unwrap();
    let mut request_builder = Request::builder();
    let request_builder = request_builder.method(method);
    let mut request_builder = request_builder.uri(endpoint.as_str());
    for (name, value) in headers.into_iter() {
        request_builder = request_builder.header(name, value);
    }
    let request = request_builder.body(body).unwrap();
    Env::fetch_serde::<_, _>(request)
}
