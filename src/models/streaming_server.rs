use crate::models::common::{eq_update, Loadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionStreamingServer, Internal, Msg};
use crate::runtime::{Effect, Effects, Env, EnvError, UpdateWithCtx};
use crate::types::api::SuccessResponse;
use crate::types::profile::Profile;
use enclose::enclose;
use futures::{FutureExt, TryFutureExt};
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

#[derive(Serialize)]
pub struct StreamingServer {
    pub selected: Selected,
    pub settings: Loadable<Settings, EnvError>,
    pub base_url: Loadable<Url, EnvError>,
}

impl StreamingServer {
    pub fn new<E: Env + 'static>(profile: &Profile) -> (Self, Effects) {
        let selected = profile.settings.streaming_server_url.to_owned();
        let effects = Effects::many(vec![
            get_settings::<E>(&selected),
            get_base_url::<E>(&selected),
        ]);
        (
            StreamingServer {
                selected,
                settings: Loadable::Loading,
                base_url: Loadable::Loading,
            },
            effects.unchanged(),
        )
    }
}

impl<E: Env + 'static> UpdateWithCtx<Ctx<E>> for StreamingServer {
    fn update(&mut self, msg: &Msg, ctx: &Ctx<E>) -> Effects {
        match msg {
            Msg::Action(Action::StreamingServer(ActionStreamingServer::Reload)) => {
                let settings_effects = eq_update(&mut self.settings, Loadable::Loading);
                let base_url_effects = eq_update(&mut self.base_url, Loadable::Loading);
                Effects::many(vec![
                    get_settings::<E>(&self.selected),
                    get_base_url::<E>(&self.selected),
                ])
                .unchanged()
                .join(settings_effects)
                .join(base_url_effects)
            }
            Msg::Action(Action::StreamingServer(ActionStreamingServer::UpdateSettings(
                settings,
            ))) if self.settings.is_ready() => {
                let settings_effects =
                    eq_update(&mut self.settings, Loadable::Ready(settings.to_owned()));
                Effects::one(set_settings::<E>(&self.selected, settings))
                    .unchanged()
                    .join(settings_effects)
            }
            Msg::Internal(Internal::ProfileChanged)
                if self.selected != ctx.profile.settings.streaming_server_url =>
            {
                self.selected = ctx.profile.settings.streaming_server_url.to_owned();
                self.settings = Loadable::Loading;
                self.base_url = Loadable::Loading;
                Effects::many(vec![
                    get_settings::<E>(&self.selected),
                    get_base_url::<E>(&self.selected),
                ])
            }
            Msg::Internal(Internal::StreamingServerSettingsResult(url, result))
                if self.selected == *url && self.settings.is_loading() =>
            {
                eq_update(
                    &mut self.settings,
                    match result {
                        Ok(settings) => Loadable::Ready(settings.to_owned()),
                        Err(error) => Loadable::Err(error.to_owned()),
                    },
                )
            }
            Msg::Internal(Internal::StreamingServerBaseURLResult(url, result))
                if self.selected == *url && self.base_url.is_loading() =>
            {
                eq_update(
                    &mut self.base_url,
                    match result {
                        Ok(base_url) => Loadable::Ready(base_url.to_owned()),
                        Err(error) => Loadable::Err(error.to_owned()),
                    },
                )
            }
            Msg::Internal(Internal::StreamingServerUpdateSettingsResult(url, result))
                if self.selected == *url =>
            {
                match result {
                    Ok(_) => Effects::none().unchanged(),
                    Err(error) => {
                        self.settings = Loadable::Err(error.to_owned());
                        Effects::none()
                    }
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn get_settings<E: Env + 'static>(url: &Url) -> Effect {
    #[derive(Deserialize)]
    struct Resp {
        values: Settings,
    }
    let endpoint = url.join("settings").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    E::fetch::<_, Resp>(request)
        .map_ok(|resp| resp.values)
        .map(enclose!((url) move |result| {
            Msg::Internal(Internal::StreamingServerSettingsResult(
                url, result,
            ))
        }))
        .boxed_local()
        .into()
}

fn get_base_url<E: Env + 'static>(url: &Url) -> Effect {
    #[derive(Deserialize)]
    struct Resp {
        #[serde(rename = "baseUrl")]
        base_url: Url,
    }
    let endpoint = url.join("settings").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    E::fetch::<_, Resp>(request)
        .map_ok(|resp| resp.base_url)
        .map(enclose!((url) move |result|
            Msg::Internal(Internal::StreamingServerBaseURLResult(url, result))
        ))
        .boxed_local()
        .into()
}

fn set_settings<E: Env + 'static>(url: &Url, settings: &Settings) -> Effect {
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
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .expect("request builder failed");
    E::fetch::<_, SuccessResponse>(request)
        .map_ok(|_| ())
        .map(enclose!((url) move |result| {
            Msg::Internal(Internal::StreamingServerUpdateSettingsResult(
                url, result,
            ))
        }))
        .boxed_local()
        .into()
}
