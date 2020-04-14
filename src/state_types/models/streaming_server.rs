use crate::state_types::models::common::Loadable;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionStreamingServer, Internal, Msg};
use crate::state_types::{Effects, EnvError, Environment, UpdateWithCtx};
use crate::types::api::SuccessResponse;
use futures::Future;
use http::request::Request;
use serde::{Deserialize, Serialize};
use url::Url;

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

pub type Selected = Url;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct StreamingServer {
    pub selected: Option<Selected>,
    pub settings: Option<Loadable<Settings, String>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for StreamingServer {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::StreamingServer(ActionStreamingServer::Reload)) => {
                let url = ctx.profile.settings.streaming_server_url.to_owned();
                let next_selected = Some(url.to_owned());
                let next_settings = Some(Loadable::Loading);
                if self.selected != next_selected || self.settings != next_settings {
                    self.selected = next_selected;
                    self.settings = next_settings;
                    Effects::one(Box::new(get_settings::<Env>(&url).then(move |result| {
                        Ok(Msg::Internal(Internal::StreamingServerSettingsResult(
                            url, result,
                        )))
                    })))
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::StreamingServer(ActionStreamingServer::UpdateSettings(
                settings,
            ))) => match (&self.selected, &mut self.settings) {
                (Some(url), Some(Loadable::Ready(ready_settings)))
                    if ready_settings != settings =>
                {
                    *ready_settings = settings.to_owned();
                    let url = url.to_owned();
                    Effects::one(Box::new(update_settings::<Env>(&url, settings).then(
                        move |result| {
                            Ok(Msg::Internal(
                                Internal::StreamingServerUpdateSettingsResult(url, result),
                            ))
                        },
                    )))
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ProfileChanged(_))
                if self.selected.as_ref() != Some(&ctx.profile.settings.streaming_server_url) =>
            {
                let url = ctx.profile.settings.streaming_server_url.to_owned();
                self.selected = Some(url.to_owned());
                self.settings = Some(Loadable::Loading);
                Effects::one(Box::new(get_settings::<Env>(&url).then(move |result| {
                    Ok(Msg::Internal(Internal::StreamingServerSettingsResult(
                        url, result,
                    )))
                })))
            }
            Msg::Internal(Internal::StreamingServerSettingsResult(url, result)) => {
                match (&self.selected, &self.settings) {
                    (Some(loading_url), Some(Loadable::Loading)) if loading_url == url => {
                        self.settings = match result {
                            Ok(settings) => Some(Loadable::Ready(settings.to_owned())),
                            Err(error) => Some(Loadable::Err(error.to_string())),
                        };
                        Effects::none()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::StreamingServerUpdateSettingsResult(url, result)) => {
                match &self.selected {
                    Some(server_url) if server_url == url => match result {
                        Ok(_) => Effects::none().unchanged(),
                        Err(error) => {
                            self.settings = Some(Loadable::Err(error.to_string()));
                            Effects::none()
                        }
                    },
                    _ => Effects::none().unchanged(),
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}

pub fn get_settings<Env: Environment + 'static>(
    url: &Url,
) -> impl Future<Item = Settings, Error = EnvError> {
    #[derive(Deserialize)]
    struct Resp {
        pub values: Settings,
    }
    let endpoint = url.join("settings").unwrap();
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder cannot fail");
    Env::fetch_serde::<_, Resp>(request).map(|resp| resp.values)
}

pub fn update_settings<Env: Environment + 'static>(
    url: &Url,
    settings: &Settings,
) -> impl Future<Item = (), Error = EnvError> {
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
    let endpoint = url.join("settings").unwrap();
    let request = Request::post(endpoint.as_str())
        .header("content-type", "application/json")
        .body(body)
        .expect("request builder cannot fail");
    Env::fetch_serde::<_, SuccessResponse>(request).map(|_| ())
}
