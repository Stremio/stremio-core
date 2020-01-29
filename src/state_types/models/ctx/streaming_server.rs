use crate::state_types::models::common::ModelError;
use crate::state_types::models::ctx::user::UserLoadable;
use crate::state_types::msg::{Action, ActionCtx, ActionStreamingServer, Event, Internal, Msg};
use crate::state_types::{Effects, Environment};
use crate::types::api::SuccessResponse;
use derivative::Derivative;
use futures::future::Either;
use futures::{future, Future};
use http::request::Request;
use http::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
use url_serde;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub app_path: String,
    pub cache_root: String,
    pub cache_size: f64,
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: u64,
    pub server_version: String,
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
    },
    Ready {
        settings: Settings,
        #[serde(with = "url_serde")]
        base_url: Url,
        #[serde(with = "url_serde")]
        url: Url,
    },
}

// move to models
impl StreamingServerLoadable {
    pub fn update<Env: Environment + 'static>(
        &mut self,
        user: &UserLoadable,
        msg: &Msg,
    ) -> Effects {
        let streaming_server_effects = match msg {
            Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(_)))
            | Msg::Action(Action::Ctx(ActionCtx::Logout))
            | Msg::Internal(Internal::UserStorageResult(_))
            | Msg::Internal(Internal::UserAuthenticateResult(_, _))
                if Some(&user.settings().streaming_server_url).ne(&self.url()) =>
            {
                let url = user.settings().streaming_server_url.to_owned();
                *self = StreamingServerLoadable::Loading {
                    url: url.to_owned(),
                };
                Effects::one(Box::new(load::<Env>(&url).then(move |result| {
                    Ok(Msg::Internal(Internal::StreamingServerReloadResult(
                        url, result,
                    )))
                })))
            }
            Msg::Action(Action::Ctx(ActionCtx::StreamingServer(ActionStreamingServer::Reload))) => {
                match &self {
                    StreamingServerLoadable::Error { url }
                    | StreamingServerLoadable::Loading { url }
                    | StreamingServerLoadable::Ready { url, .. } => {
                        let url = url.to_owned();
                        *self = StreamingServerLoadable::Loading {
                            url: url.to_owned(),
                        };
                        Effects::one(Box::new(load::<Env>(&url).then(move |result| {
                            Ok(Msg::Internal(Internal::StreamingServerReloadResult(
                                url, result,
                            )))
                        })))
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::StreamingServer(
                ActionStreamingServer::UpdateSettings(settings),
            ))) => match self {
                StreamingServerLoadable::Ready {
                    url,
                    settings: ready_settings,
                    ..
                } => {
                    *ready_settings = settings.to_owned();
                    let url = url.to_owned();
                    let result_url = url.to_owned();
                    Effects::one(Box::new(
                        update_settings::<Env>(&url, settings)
                            .and_then(move |_| load::<Env>(&url))
                            .then(move |result| {
                                Ok(Msg::Internal(Internal::StreamingServerReloadResult(
                                    result_url, result,
                                )))
                            }),
                    ))
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::StreamingServerReloadResult(url, result)) => match &self {
                StreamingServerLoadable::Loading { url: loading_url } if loading_url.eq(url) => {
                    match result {
                        Ok((base_url, settings)) => {
                            *self = StreamingServerLoadable::Ready {
                                url: url.to_owned(),
                                base_url: base_url.to_owned(),
                                settings: settings.to_owned(),
                            };
                            Effects::msg(Msg::Event(Event::StreamingServerLoaded))
                        }
                        Err(error) => {
                            *self = StreamingServerLoadable::Error {
                                url: url.to_owned(),
                            };
                            Effects::msg(Msg::Event(Event::Error(error.to_owned())))
                        }
                    }
                }
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        };
        if streaming_server_effects.has_changed {
            Effects::msg(Msg::Internal(Internal::StreamingServerChanged))
                .join(streaming_server_effects)
        } else {
            streaming_server_effects
        }
    }
    pub fn url(&self) -> Option<&Url> {
        match &self {
            StreamingServerLoadable::NotLoaded => None,
            StreamingServerLoadable::Error { url }
            | StreamingServerLoadable::Loading { url }
            | StreamingServerLoadable::Ready { url, .. } => Some(url),
        }
    }
}

fn update_settings<Env: Environment + 'static>(
    url: &Url,
    settings: &Settings,
) -> impl Future<Item = (), Error = ModelError> {
    request::<Env, _, SuccessResponse>(
        &url,
        "settings",
        Method::POST,
        [("content-type", "application/json")]
            .iter()
            .cloned()
            .collect(),
        settings.to_owned(),
    )
    .then(|result| result.map(|_| ()))
}

fn load<Env: Environment + 'static>(
    url: &Url,
) -> impl Future<Item = (Url, Settings), Error = ModelError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Resp {
        pub values: Settings,
        #[serde(with = "url_serde")]
        pub base_url: Url,
    }
    request::<Env, _, Resp>(&url, "settings", Method::GET, HashMap::new(), ())
        .then(|result| result.map(|resp| (resp.base_url, resp.values)))
}

fn request<Env, Body, Resp>(
    url: &Url,
    route: &str,
    method: Method,
    headers: HashMap<&str, &str>,
    body: Body,
) -> impl Future<Item = Resp, Error = ModelError>
where
    Env: Environment + 'static,
    Body: Serialize + 'static,
    Resp: DeserializeOwned + 'static,
{
    match url.join(route) {
        Ok(endpoint) => {
            // TODO refactor this code! it looks ugly because of http::request::Builder
            let mut request_builder = Request::builder();
            let request_builder = request_builder.method(method);
            let mut request_builder = request_builder.uri(endpoint.as_str());
            for (name, value) in headers.into_iter() {
                request_builder = request_builder.header(name, value);
            }
            let request = request_builder.body(body);
            match request {
                Ok(request) => {
                    Either::A(Env::fetch_serde::<_, _>(request).map_err(ModelError::from))
                }
                Err(error) => Either::B(future::err(ModelError::from(error))),
            }
        }
        Err(error) => Either::B(future::err(ModelError::from(error))),
    }
}
