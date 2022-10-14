use crate::models::common::{eq_update, Loadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionStreamingServer, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvError, EnvFutureExt, UpdateWithCtx};
use crate::types::api::SuccessResponse;
use crate::types::profile::Profile;
use crate::types::resource::Torrent;
use enclose::enclose;
use futures::{FutureExt, TryFutureExt};
use http::request::Request;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::iter;
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
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

#[derive(Serialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub transport_url: Url,
}

#[derive(Serialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct StreamingServer {
    pub selected: Selected,
    pub settings: Loadable<Settings, EnvError>,
    pub base_url: Loadable<Url, EnvError>,
    pub torrent: Option<(Torrent, Loadable<(), EnvError>)>,
}

impl StreamingServer {
    pub fn new<E: Env + 'static>(profile: &Profile) -> (Self, Effects) {
        let effects = Effects::many(vec![
            get_settings::<E>(&profile.settings.streaming_server_url),
            get_base_url::<E>(&profile.settings.streaming_server_url),
        ]);
        (
            Self {
                selected: Selected {
                    transport_url: profile.settings.streaming_server_url.to_owned(),
                },
                settings: Loadable::Loading,
                base_url: Loadable::Loading,
                torrent: None,
            },
            effects.unchanged(),
        )
    }
}

impl<E: Env + 'static> UpdateWithCtx<E> for StreamingServer {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::StreamingServer(ActionStreamingServer::Reload)) => {
                let settings_effects = eq_update(&mut self.settings, Loadable::Loading);
                let base_url_effects = eq_update(&mut self.base_url, Loadable::Loading);
                Effects::many(vec![
                    get_settings::<E>(&self.selected.transport_url),
                    get_base_url::<E>(&self.selected.transport_url),
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
                Effects::one(set_settings::<E>(&self.selected.transport_url, settings))
                    .unchanged()
                    .join(settings_effects)
            }
            Msg::Action(Action::StreamingServer(ActionStreamingServer::CreateTorrent(torrent))) => {
                let torrent_effects = eq_update(
                    &mut self.torrent,
                    Some((torrent.to_owned(), Loadable::Loading)),
                );
                Effects::one(create_torrent::<E>(&self.selected.transport_url, torrent))
                    .unchanged()
                    .join(torrent_effects)
            }
            Msg::Internal(Internal::ProfileChanged)
                if self.selected.transport_url != ctx.profile.settings.streaming_server_url =>
            {
                self.selected = Selected {
                    transport_url: ctx.profile.settings.streaming_server_url.to_owned(),
                };
                self.settings = Loadable::Loading;
                self.base_url = Loadable::Loading;
                Effects::many(vec![
                    get_settings::<E>(&self.selected.transport_url),
                    get_base_url::<E>(&self.selected.transport_url),
                ])
            }
            Msg::Internal(Internal::StreamingServerSettingsResult(url, result))
                if self.selected.transport_url == *url && self.settings.is_loading() =>
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
                if self.selected.transport_url == *url && self.base_url.is_loading() =>
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
                if self.selected.transport_url == *url =>
            {
                match result {
                    Ok(_) => Effects::none().unchanged(),
                    Err(error) => {
                        self.settings = Loadable::Err(error.to_owned());
                        Effects::none()
                    }
                }
            }
            Msg::Internal(Internal::StreamingServerCreateTorrentResult(
                loading_torrent,
                result,
            )) => match &mut self.torrent {
                Some((torrent, loadable))
                    if torrent == loading_torrent && loadable.is_loading() =>
                {
                    *loadable = match result {
                        Ok(_) => Loadable::Ready(()),
                        Err(error) => Loadable::Err(error.to_owned()),
                    };
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            },
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
    EffectFuture::Concurrent(
        E::fetch::<_, Resp>(request)
            .map_ok(|resp| resp.values)
            .map(enclose!((url) move |result| {
                Msg::Internal(Internal::StreamingServerSettingsResult(
                    url, result,
                ))
            }))
            .boxed_env(),
    )
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
    EffectFuture::Concurrent(
        E::fetch::<_, Resp>(request)
            .map_ok(|resp| resp.base_url)
            .map(enclose!((url) move |result|
                Msg::Internal(Internal::StreamingServerBaseURLResult(url, result))
            ))
            .boxed_env(),
    )
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
    EffectFuture::Concurrent(
        E::fetch::<_, SuccessResponse>(request)
            .map_ok(|_| ())
            .map(enclose!((url) move |result| {
                Msg::Internal(Internal::StreamingServerUpdateSettingsResult(
                    url, result,
                ))
            }))
            .boxed_env(),
    )
    .into()
}

fn create_torrent<E: Env + 'static>(url: &Url, torrent: &Torrent) -> Effect {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PeerSearch {
        sources: Vec<String>,
        min: u32,
        max: u32,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body {
        torrent: Torrent,
        peer_search: Option<PeerSearch>,
        guess_file_idx: bool,
    }
    let encoded_info_hash = hex::encode(&torrent.info_hash);
    let endpoint = url
        .join(&format!("{}/", encoded_info_hash))
        .expect("url builder failed")
        .join("create")
        .expect("url builder failed");
    let body = Body {
        torrent: torrent.to_owned(),
        peer_search: if !torrent.announce.is_empty() {
            Some(PeerSearch {
                sources: iter::once(&format!("dht:{}", encoded_info_hash))
                    .chain(torrent.announce.iter())
                    .cloned()
                    .unique()
                    .collect(),
                min: 40,
                max: 200,
            })
        } else {
            None
        },
        guess_file_idx: false,
    };
    let request = Request::post(endpoint.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, ()>(request)
            .map(enclose!((torrent) move |result| {
                Msg::Internal(Internal::StreamingServerCreateTorrentResult(
                    torrent, result,
                ))
            }))
            .boxed_env(),
    )
    .into()
}
