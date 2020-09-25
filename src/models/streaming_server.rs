use crate::models::common::Loadable;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionStreamingServer, Internal, Msg};
use crate::runtime::{Effects, Env, EnvError, UpdateWithCtx};
use crate::types::api::SuccessResponse;
use enclose::enclose;
use futures::{Future, FutureExt, TryFutureExt};
use http::request::Request;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Serialize)]
pub struct StreamingServer {
    pub selected: Option<Selected>,
    pub settings: Option<Loadable<Settings, EnvError>>,
    pub base_url: Option<Loadable<Url, EnvError>>,
}

impl<E: Env + 'static> UpdateWithCtx<Ctx<E>> for StreamingServer {
    fn update(&mut self, ctx: &Ctx<E>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::StreamingServer(ActionStreamingServer::Reload)) => {
                let url = ctx.profile.settings.streaming_server_url.to_owned();
                let next_selected = Some(url.to_owned());
                let next_settings = Some(Loadable::Loading);
                let next_base_url = Some(Loadable::Loading);
                if self.selected != next_selected
                    || self.settings != next_settings
                    || self.base_url != next_base_url
                {
                    self.selected = next_selected;
                    self.settings = next_settings;
                    self.base_url = next_base_url;
                    Effects::futures(vec![
                        get_settings::<E>(&url)
                            .map(enclose!((url) move |result| {
                                Msg::Internal(Internal::StreamingServerSettingsResult(
                                    url, result,
                                ))
                            }))
                            .boxed_local(),
                        get_base_url::<E>(&url)
                            .map(move |result| {
                                Msg::Internal(Internal::StreamingServerBaseURLResult(url, result))
                            })
                            .boxed_local(),
                    ])
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
                    Effects::future(
                        set_settings::<E>(&url, settings)
                            .map(enclose!((url) move |result| {
                                Msg::Internal(Internal::StreamingServerUpdateSettingsResult(
                                    url, result,
                                ))
                            }))
                            .boxed_local(),
                    )
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ProfileChanged(_))
                if self.selected.as_ref() != Some(&ctx.profile.settings.streaming_server_url) =>
            {
                let url = ctx.profile.settings.streaming_server_url.to_owned();
                self.selected = Some(url.to_owned());
                self.settings = Some(Loadable::Loading);
                self.base_url = Some(Loadable::Loading);
                Effects::futures(vec![
                    get_settings::<E>(&url)
                        .map(enclose!((url) move |result| {
                            Msg::Internal(Internal::StreamingServerSettingsResult(
                                url, result,
                            ))
                        }))
                        .boxed_local(),
                    get_base_url::<E>(&url)
                        .map(move |result| {
                            Msg::Internal(Internal::StreamingServerBaseURLResult(url, result))
                        })
                        .boxed_local(),
                ])
            }
            Msg::Internal(Internal::StreamingServerSettingsResult(url, result)) => {
                match (&self.selected, &self.settings) {
                    (Some(loading_url), Some(Loadable::Loading)) if loading_url == url => {
                        self.settings = match result {
                            Ok(settings) => Some(Loadable::Ready(settings.to_owned())),
                            Err(error) => Some(Loadable::Err(error.to_owned())),
                        };
                        Effects::none()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::StreamingServerBaseURLResult(url, result)) => {
                match (&self.selected, &self.base_url) {
                    (Some(loading_url), Some(Loadable::Loading)) if loading_url == url => {
                        self.base_url = match result {
                            Ok(base_url) => Some(Loadable::Ready(base_url.to_owned())),
                            Err(error) => Some(Loadable::Err(error.to_owned())),
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
                            self.settings = Some(Loadable::Err(error.to_owned()));
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

fn get_settings<E: Env + 'static>(url: &Url) -> impl Future<Output = Result<Settings, EnvError>> {
    #[derive(Deserialize)]
    struct Resp {
        values: Settings,
    }
    let endpoint = url.join("settings").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    E::fetch::<_, Resp>(request).map_ok(|resp| resp.values)
}

fn get_base_url<E: Env + 'static>(url: &Url) -> impl Future<Output = Result<Url, EnvError>> {
    #[derive(Deserialize)]
    struct Resp {
        #[serde(rename = "baseUrl")]
        base_url: Url,
    }
    let endpoint = url.join("settings").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    E::fetch::<_, Resp>(request).map_ok(|resp| resp.base_url)
}

fn set_settings<E: Env + 'static>(
    url: &Url,
    settings: &Settings,
) -> impl Future<Output = Result<(), EnvError>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body {
        cache_size: Option<f64>,
        bt_max_connections: u64,
        bt_handshake_timeout: u64,
        bt_request_timeout: u64,
        bt_download_speed_soft_limit: f64,
        bt_download_speed_hard_limit: f64,
        bt_min_peers_for_stable: u64,
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
    let endpoint = url.join("settings").expect("url builder failed");
    let request = Request::post(endpoint.as_str())
        .header("content-type", "application/json")
        .body(body)
        .expect("request builder failed");
    E::fetch::<_, SuccessResponse>(request).map_ok(|_| ())
}
