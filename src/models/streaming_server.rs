use crate::constants::META_RESOURCE_NAME;
use crate::models::common::{eq_update, Loadable};
use crate::models::ctx::{Ctx, CtxError};
use crate::runtime::msg::{
    Action, ActionStreamingServer, CreateTorrentArgs, Event, Internal, Msg, PlayOnDeviceArgs,
};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvError, EnvFutureExt, UpdateWithCtx};
use crate::types::addon::ResourcePath;
use crate::types::api::SuccessResponse;
use crate::types::profile::Profile;
use crate::types::streaming_server::Statistics;
use enclose::enclose;
use futures::{FutureExt, TryFutureExt};
use http::request::Request;
use magnet_url::{Magnet, MagnetError};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::iter;
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackDevice {
    pub id: String,
    pub name: String,
    pub r#type: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsRequest {
    pub info_hash: String,
    pub file_idx: u16,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub transport_url: Url,
    pub statistics: Option<StatisticsRequest>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamingServer {
    pub selected: Selected,
    pub settings: Loadable<Settings, EnvError>,
    pub base_url: Loadable<Url, EnvError>,
    pub playback_devices: Loadable<Vec<PlaybackDevice>, EnvError>,
    pub torrent: Option<(String, Loadable<ResourcePath, EnvError>)>,
    pub statistics: Option<Loadable<Statistics, EnvError>>,
}

impl StreamingServer {
    pub fn new<E: Env + 'static>(profile: &Profile) -> (Self, Effects) {
        let effects = Effects::many(vec![
            get_settings::<E>(&profile.settings.streaming_server_url),
            get_base_url::<E>(&profile.settings.streaming_server_url),
            get_playback_devices::<E>(&profile.settings.streaming_server_url),
        ]);
        (
            Self {
                selected: Selected {
                    transport_url: profile.settings.streaming_server_url.to_owned(),
                    statistics: None,
                },
                settings: Loadable::Loading,
                base_url: Loadable::Loading,
                playback_devices: Loadable::Loading,
                torrent: None,
                statistics: None,
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
                    get_playback_devices::<E>(&self.selected.transport_url),
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
            Msg::Action(Action::StreamingServer(ActionStreamingServer::CreateTorrent(
                CreateTorrentArgs::Magnet(magnet),
            ))) => match parse_magnet(magnet) {
                Ok((info_hash, announce)) => {
                    let torrent_effects = eq_update(
                        &mut self.torrent,
                        Some((info_hash.to_owned(), Loadable::Loading)),
                    );
                    Effects::many(vec![
                        create_magnet::<E>(&self.selected.transport_url, &info_hash, &announce),
                        Effect::Msg(Box::new(Msg::Event(Event::MagnetParsed {
                            magnet: magnet.to_owned(),
                        }))),
                    ])
                    .unchanged()
                    .join(torrent_effects)
                }
                Err(_) => {
                    let torrent_effects = eq_update(&mut self.torrent, None);
                    Effects::msg(Msg::Event(Event::Error {
                        error: CtxError::Env(EnvError::Other(
                            "Failed to parse magnet url".to_owned(),
                        )),
                        source: Box::new(Event::MagnetParsed {
                            magnet: magnet.to_owned(),
                        }),
                    }))
                    .unchanged()
                    .join(torrent_effects)
                }
            },
            Msg::Action(Action::StreamingServer(ActionStreamingServer::CreateTorrent(
                CreateTorrentArgs::File(torrent),
            ))) => match parse_torrent(torrent) {
                Ok((info_hash, _)) => {
                    let torrent_effects = eq_update(
                        &mut self.torrent,
                        Some((info_hash.to_owned(), Loadable::Loading)),
                    );
                    Effects::many(vec![
                        create_torrent::<E>(&self.selected.transport_url, &info_hash, torrent),
                        Effect::Msg(Box::new(Msg::Event(Event::TorrentParsed {
                            torrent: torrent.to_owned(),
                        }))),
                    ])
                    .unchanged()
                    .join(torrent_effects)
                }
                Err(error) => {
                    let torrent_effects = eq_update(&mut self.torrent, None);
                    Effects::msg(Msg::Event(Event::Error {
                        error: CtxError::Env(EnvError::Other(format!(
                            "Failed to parse torrent file: {error}"
                        ))),
                        source: Box::new(Event::TorrentParsed {
                            torrent: torrent.to_owned(),
                        }),
                    }))
                    .unchanged()
                    .join(torrent_effects)
                }
            },
            Msg::Action(Action::StreamingServer(ActionStreamingServer::GetStatistics(request))) => {
                let selected_effects =
                    eq_update(&mut self.selected.statistics, Some(request.to_owned()));
                let statistics_effects = eq_update(&mut self.statistics, Some(Loadable::Loading));
                Effects::one(get_torrent_statistics::<E>(
                    &self.selected.transport_url,
                    request,
                ))
                .unchanged()
                .join(selected_effects)
                .join(statistics_effects)
            }
            Msg::Action(Action::StreamingServer(ActionStreamingServer::PlayOnDevice(args))) => {
                match Url::parse(&args.source).is_ok() {
                    true => match &mut self.playback_devices {
                        Loadable::Ready(playback_devices) => {
                            let device_exists = playback_devices
                                .iter()
                                .any(|device| device.id == args.device);
                            match device_exists {
                                true => Effects::one(play_on_device::<E>(
                                    &self.selected.transport_url,
                                    args,
                                ))
                                .unchanged(),
                                _ => Effects::none().unchanged(),
                            }
                        }
                        _ => Effects::none().unchanged(),
                    },
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::ProfileChanged)
                if self.selected.transport_url != ctx.profile.settings.streaming_server_url =>
            {
                self.selected = Selected {
                    transport_url: ctx.profile.settings.streaming_server_url.to_owned(),
                    statistics: None,
                };
                self.settings = Loadable::Loading;
                self.base_url = Loadable::Loading;
                self.torrent = None;
                self.statistics = None;
                Effects::many(vec![
                    get_settings::<E>(&self.selected.transport_url),
                    get_base_url::<E>(&self.selected.transport_url),
                    get_playback_devices::<E>(&self.selected.transport_url),
                ])
            }
            Msg::Internal(Internal::StreamingServerSettingsResult(url, result))
                if self.selected.transport_url == *url && self.settings.is_loading() =>
            {
                match result {
                    Ok(settings) => {
                        eq_update(&mut self.settings, Loadable::Ready(settings.to_owned()))
                    }
                    Err(error) => {
                        let base_url_effects =
                            eq_update(&mut self.base_url, Loadable::Err(error.to_owned()));
                        let playback_devices_effects =
                            eq_update(&mut self.playback_devices, Loadable::Err(error.to_owned()));
                        let settings_effects =
                            eq_update(&mut self.settings, Loadable::Err(error.to_owned()));
                        let torrent_effects = eq_update(&mut self.torrent, None);
                        base_url_effects
                            .join(playback_devices_effects)
                            .join(settings_effects)
                            .join(torrent_effects)
                    }
                }
            }
            Msg::Internal(Internal::StreamingServerBaseURLResult(url, result))
                if self.selected.transport_url == *url && self.base_url.is_loading() =>
            {
                match result {
                    Ok(base_url) => {
                        eq_update(&mut self.base_url, Loadable::Ready(base_url.to_owned()))
                    }
                    Err(error) => {
                        let base_url_effects =
                            eq_update(&mut self.base_url, Loadable::Err(error.to_owned()));
                        let playback_devices_effects =
                            eq_update(&mut self.playback_devices, Loadable::Err(error.to_owned()));
                        let settings_effects =
                            eq_update(&mut self.settings, Loadable::Err(error.to_owned()));
                        let torrent_effects = eq_update(&mut self.torrent, None);
                        base_url_effects
                            .join(playback_devices_effects)
                            .join(settings_effects)
                            .join(torrent_effects)
                    }
                }
            }
            Msg::Internal(Internal::StreamingServerPlaybackDevicesResult(url, result))
                if self.selected.transport_url == *url && self.playback_devices.is_loading() =>
            {
                match result {
                    Ok(playback_devices) => eq_update(
                        &mut self.playback_devices,
                        Loadable::Ready(playback_devices.to_owned()),
                    ),
                    Err(error) => {
                        let base_url_effects =
                            eq_update(&mut self.base_url, Loadable::Err(error.to_owned()));
                        let playback_devices_effects =
                            eq_update(&mut self.playback_devices, Loadable::Err(error.to_owned()));
                        let settings_effects =
                            eq_update(&mut self.settings, Loadable::Err(error.to_owned()));
                        let torrent_effects = eq_update(&mut self.torrent, None);
                        base_url_effects
                            .join(playback_devices_effects)
                            .join(settings_effects)
                            .join(torrent_effects)
                    }
                }
            }
            Msg::Internal(Internal::StreamingServerUpdateSettingsResult(url, result))
                if self.selected.transport_url == *url =>
            {
                match result {
                    Ok(_) => Effects::none().unchanged(),
                    Err(error) => {
                        let base_url_effects =
                            eq_update(&mut self.base_url, Loadable::Err(error.to_owned()));
                        let settings_effects =
                            eq_update(&mut self.settings, Loadable::Err(error.to_owned()));
                        let torrent_effects = eq_update(&mut self.torrent, None);
                        base_url_effects
                            .join(settings_effects)
                            .join(torrent_effects)
                    }
                }
            }
            Msg::Internal(Internal::StreamingServerStatisticsResult((url, request), result))
                if self.selected.transport_url == *url
                    && self.selected.statistics.as_ref() == Some(request) =>
            {
                let loadable = match result {
                    Ok(statistics) => Loadable::Ready(statistics.to_owned()),
                    Err(error) => Loadable::Err(error.to_owned()),
                };
                eq_update(&mut self.statistics, Some(loadable))
            }
            Msg::Internal(Internal::StreamingServerPlayOnDeviceResult(device, result)) => {
                match result {
                    Ok(_) => {
                        Effects::one(Effect::Msg(Box::new(Msg::Event(Event::PlayingOnDevice {
                            device: device.to_owned(),
                        }))))
                        .unchanged()
                    }
                    Err(_) => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::StreamingServerCreateTorrentResult(
                loading_info_hash,
                result,
            )) => match &mut self.torrent {
                Some((info_hash, loadable))
                    if info_hash == loading_info_hash && loadable.is_loading() =>
                {
                    *loadable = match result {
                        Ok(_) => Loadable::Ready(ResourcePath {
                            resource: META_RESOURCE_NAME.to_owned(),
                            r#type: "other".to_owned(),
                            id: format!("bt:{info_hash}"),
                            extra: vec![],
                        }),
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

fn get_playback_devices<E: Env + 'static>(url: &Url) -> Effect {
    let endpoint = url.join("casting").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, Vec<PlaybackDevice>>(request)
            .map_ok(|resp| resp)
            .map(enclose!((url) move |result|
                Msg::Internal(Internal::StreamingServerPlaybackDevicesResult(url, result))
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

fn create_magnet<E: Env + 'static>(url: &Url, info_hash: &str, announce: &[String]) -> Effect {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PeerSearch {
        sources: Vec<String>,
        min: u32,
        max: u32,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Torrent {
        info_hash: String,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body {
        torrent: Torrent,
        peer_search: Option<PeerSearch>,
    }
    let info_hash = info_hash.to_owned();
    let endpoint = url
        .join(&format!("{info_hash}/"))
        .expect("url builder failed")
        .join("create")
        .expect("url builder failed");
    let body = Body {
        torrent: Torrent {
            info_hash: info_hash.to_owned(),
        },
        peer_search: if !announce.is_empty() {
            Some(PeerSearch {
                sources: iter::once(&format!("dht:{info_hash}"))
                    .chain(announce.iter())
                    .cloned()
                    .collect(),
                min: 40,
                max: 200,
            })
        } else {
            None
        },
    };
    let request = Request::post(endpoint.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, serde_json::Value>(request)
            .map_ok(|_| ())
            .map(enclose!((info_hash) move |result| {
                Msg::Internal(Internal::StreamingServerCreateTorrentResult(
                    info_hash, result,
                ))
            }))
            .boxed_env(),
    )
    .into()
}

fn create_torrent<E: Env + 'static>(url: &Url, info_hash: &str, torrent: &[u8]) -> Effect {
    #[derive(Serialize)]
    struct Body {
        blob: String,
    }
    let info_hash = info_hash.to_owned();
    let endpoint = url.join("/create").expect("url builder failed");
    let request = Request::post(endpoint.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body {
            blob: hex::encode(torrent),
        })
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, serde_json::Value>(request)
            .map_ok(|_| ())
            .map(enclose!((info_hash) move |result| {
                Msg::Internal(Internal::StreamingServerCreateTorrentResult(
                    info_hash, result,
                ))
            }))
            .boxed_env(),
    )
    .into()
}

fn parse_magnet(magnet: &Url) -> Result<(String, Vec<String>), MagnetError> {
    let magnet = Magnet::new(magnet.as_str())?;
    let info_hash = magnet.xt.ok_or(MagnetError::NotAMagnetURL)?;
    let announce = magnet.tr;
    Ok((info_hash, announce))
}

fn parse_torrent(torrent: &[u8]) -> Result<(String, Vec<String>), serde_bencode::Error> {
    #[derive(Deserialize)]
    struct TorrentFile {
        info: serde_bencode::value::Value,
        #[serde(default)]
        announce: Option<String>,
        #[serde(default)]
        #[serde(rename = "announce-list")]
        announce_list: Option<Vec<Vec<String>>>,
    }
    let torrent_file = serde_bencode::from_bytes::<TorrentFile>(torrent)?;
    let info_bytes = serde_bencode::to_bytes(&torrent_file.info)?;
    let mut hasher = Sha1::new();
    hasher.update(info_bytes);
    let info_hash = hex::encode(hasher.finalize());
    let mut announce = vec![];
    if let Some(announce_entry) = torrent_file.announce {
        announce.push(announce_entry);
    };
    if let Some(announce_lists) = torrent_file.announce_list {
        for announce_list in announce_lists {
            announce.extend(announce_list.into_iter());
        }
    };
    announce.dedup();
    Ok((info_hash, announce))
}

fn get_torrent_statistics<E: Env + 'static>(url: &Url, request: &StatisticsRequest) -> Effect {
    let statistics_request = request.clone();
    let endpoint = url
        .join(&format!(
            "/{}/{}/stats.json",
            statistics_request.info_hash.clone(),
            statistics_request.file_idx
        ))
        .expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(())
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, Statistics>(request)
            .map(enclose!((url) move |result|
                Msg::Internal(Internal::StreamingServerStatisticsResult((url, statistics_request), result))
            ))
            .boxed_env(),
    )
    .into()
}

fn play_on_device<E: Env + 'static>(url: &Url, args: &PlayOnDeviceArgs) -> Effect {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body {
        source: String,
        time: u64,
    }
    let device = args.device.clone();
    let endpoint = url
        .join(&format!("casting/{device}/player"))
        .expect("url builder failed");
    let body = Body {
        source: args.source.to_owned(),
        time: args.time.unwrap_or(0),
    };
    let request = Request::post(endpoint.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, serde_json::Value>(request)
            .map_ok(|_| ())
            .map(enclose!(() move |result|
                Msg::Internal(Internal::StreamingServerPlayOnDeviceResult(device, result))
            ))
            .boxed_env(),
    )
    .into()
}
