use enclose::enclose;
use futures::{FutureExt, TryFutureExt};
use http::request::Request;
use magnet_url::{Magnet, MagnetError};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sha1::{Digest, Sha1};
use std::iter;
use url::Url;

use crate::constants::META_RESOURCE_NAME;
use crate::models::common::{eq_update, Loadable};
use crate::models::ctx::{Ctx, CtxError};
use crate::runtime::msg::{
    Action, ActionStreamingServer, CreateTorrentArgs, Event, Internal, Msg, PlayOnDeviceArgs,
};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvError, EnvFutureExt, UpdateWithCtx};
use crate::types::addon::ResourcePath;
use crate::types::api::SuccessResponse;
use crate::types::profile::{AuthKey, Profile};
use crate::types::resource::{ConvertedStreamSource, InfoHash, SeriesInfo, Stream, StreamSource};
use crate::types::streaming_server::{
    ArchiveCreateResponse, ArchiveStreamOptions, ArchiveStreamRequest, CreateTorrentRequest, DeviceInfo, FileNameRequest, GetHTTPSResponse, NetworkInfo, OpensubtitlesParamsRequest, OpensubtitlesParamsResponse, PeerSearch, Settings, SettingsResponse, Statistics
};

use super::player::VideoParams;

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

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub transport_url: Url,
    pub statistics: Option<StatisticsRequest>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamingServer {
    pub selected: Selected,
    pub settings: Loadable<Settings, EnvError>,
    pub base_url: Option<Url>,
    pub remote_url: Option<Url>,
    pub playback_devices: Loadable<Vec<PlaybackDevice>, EnvError>,
    pub network_info: Loadable<NetworkInfo, EnvError>,
    pub device_info: Loadable<DeviceInfo, EnvError>,
    pub torrent: Option<(String, Loadable<ResourcePath, EnvError>)>,
    /// [`Loadable::Loading`] is used only on the first statistics request.
    pub statistics: Option<Loadable<Statistics, EnvError>>,
}

impl StreamingServer {
    pub fn new<E: Env + 'static>(profile: &Profile) -> (Self, Effects) {
        let effects = Effects::many(vec![
            get_settings::<E>(&profile.settings.streaming_server_url),
            get_playback_devices::<E>(&profile.settings.streaming_server_url),
            get_network_info::<E>(&profile.settings.streaming_server_url),
            get_device_info::<E>(&profile.settings.streaming_server_url),
        ]);
        (
            Self {
                selected: Selected {
                    transport_url: profile.settings.streaming_server_url.to_owned(),
                    statistics: None,
                },
                settings: Loadable::Loading,
                base_url: None,
                remote_url: None,
                playback_devices: Loadable::Loading,
                network_info: Loadable::Loading,
                device_info: Loadable::Loading,
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
                let network_info_effects = eq_update(&mut self.network_info, Loadable::Loading);
                let device_info_effects = eq_update(&mut self.device_info, Loadable::Loading);
                let base_url_effects = eq_update(&mut self.base_url, None);
                let remote_url_effects = eq_update(&mut self.remote_url, None);
                Effects::many(vec![
                    get_settings::<E>(&self.selected.transport_url),
                    get_playback_devices::<E>(&self.selected.transport_url),
                    get_network_info::<E>(&self.selected.transport_url),
                    get_device_info::<E>(&self.selected.transport_url),
                ])
                .unchanged()
                .join(settings_effects)
                .join(network_info_effects)
                .join(device_info_effects)
                .join(base_url_effects)
                .join(remote_url_effects)
            }
            Msg::Action(Action::StreamingServer(ActionStreamingServer::UpdateSettings(
                settings,
            ))) if self.settings.is_ready() => {
                let settings_effects =
                    eq_update(&mut self.settings, Loadable::Ready(settings.to_owned()));
                let remote_url_effects =
                    update_remote_url::<E>(&mut self.remote_url, &self.selected, settings, ctx);
                Effects::one(set_settings::<E>(&self.selected.transport_url, settings))
                    .unchanged()
                    .join(settings_effects)
                    .join(remote_url_effects)
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
                        create_torrent_request::<E>(
                            &self.selected.transport_url,
                            &info_hash,
                            torrent,
                        ),
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
                let is_different_request = self.selected.statistics.as_ref() != Some(request);
                //set the Loading state only on the first fetch of the statistics
                let statistics_effects = match (&self.statistics, is_different_request) {
                    (None, _) | (_, true) => {
                        eq_update(&mut self.statistics, Some(Loadable::Loading))
                    }
                    _ => Effects::none().unchanged(),
                };
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
                self.network_info = Loadable::Loading;
                self.device_info = Loadable::Loading;
                self.base_url = None;
                self.remote_url = None;
                self.torrent = None;
                self.statistics = None;
                Effects::many(vec![
                    get_settings::<E>(&self.selected.transport_url),
                    get_playback_devices::<E>(&self.selected.transport_url),
                    get_network_info::<E>(&self.selected.transport_url),
                    get_device_info::<E>(&self.selected.transport_url),
                ])
            }
            Msg::Internal(Internal::StreamingServerSettingsResult(url, result))
                if self.selected.transport_url == *url && self.settings.is_loading() =>
            {
                match result {
                    Ok(settings) => {
                        let settings_effects = eq_update(
                            &mut self.settings,
                            Loadable::Ready(settings.values.to_owned()),
                        );
                        let base_url_effects =
                            eq_update(&mut self.base_url, Some(settings.base_url.to_owned()));
                        let remote_url_effects = update_remote_url::<E>(
                            &mut self.remote_url,
                            &self.selected,
                            &settings.values,
                            ctx,
                        );
                        settings_effects
                            .join(base_url_effects)
                            .join(remote_url_effects)
                    }
                    Err(error) => {
                        let base_url_effects = eq_update(&mut self.base_url, None);
                        let remote_url_effects = eq_update(&mut self.remote_url, None);
                        let playback_devices_effects =
                            eq_update(&mut self.playback_devices, Loadable::Err(error.to_owned()));
                        let network_info_effects =
                            eq_update(&mut self.network_info, Loadable::Err(error.to_owned()));
                        let device_info_effects =
                            eq_update(&mut self.device_info, Loadable::Err(error.to_owned()));
                        let settings_effects =
                            eq_update(&mut self.settings, Loadable::Err(error.to_owned()));
                        let torrent_effects = eq_update(&mut self.torrent, None);
                        base_url_effects
                            .join(remote_url_effects)
                            .join(playback_devices_effects)
                            .join(network_info_effects)
                            .join(device_info_effects)
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
                        eq_update(&mut self.playback_devices, Loadable::Err(error.to_owned()))
                    }
                }
            }
            Msg::Internal(Internal::StreamingServerNetworkInfoResult(url, result))
                if self.selected.transport_url == *url && self.network_info.is_loading() =>
            {
                match result {
                    Ok(network_info) => eq_update(
                        &mut self.network_info,
                        Loadable::Ready(network_info.to_owned()),
                    ),
                    Err(error) => {
                        eq_update(&mut self.network_info, Loadable::Err(error.to_owned()))
                    }
                }
            }
            Msg::Internal(Internal::StreamingServerDeviceInfoResult(url, result))
                if self.selected.transport_url == *url && self.device_info.is_loading() =>
            {
                match result {
                    Ok(device_info) => eq_update(
                        &mut self.device_info,
                        Loadable::Ready(device_info.to_owned()),
                    ),
                    Err(error) => eq_update(&mut self.device_info, Loadable::Err(error.to_owned())),
                }
            }
            Msg::Internal(Internal::StreamingServerUpdateSettingsResult(url, result))
                if self.selected.transport_url == *url =>
            {
                match result {
                    Ok(_) => Effects::none().unchanged(),
                    Err(error) => {
                        let base_url_effects = eq_update(&mut self.base_url, None);
                        let remote_url_effects = eq_update(&mut self.remote_url, None);
                        let playback_devices_effects =
                            eq_update(&mut self.playback_devices, Loadable::Err(error.to_owned()));
                        let network_info_effects =
                            eq_update(&mut self.network_info, Loadable::Err(error.to_owned()));
                        let device_info_effects =
                            eq_update(&mut self.device_info, Loadable::Err(error.to_owned()));
                        let settings_effects =
                            eq_update(&mut self.settings, Loadable::Err(error.to_owned()));
                        let torrent_effects = eq_update(&mut self.torrent, None);
                        base_url_effects
                            .join(remote_url_effects)
                            .join(playback_devices_effects)
                            .join(network_info_effects)
                            .join(device_info_effects)
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
                    Ok(Some(statistics)) => Loadable::Ready(statistics.to_owned()),
                    // we've loaded the whole stream, no need to update the statistics.
                    Ok(None) => return Effects::none().unchanged(),
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
            Msg::Internal(Internal::StreamingServerGetHTTPSResult(url, result))
                if self.selected.transport_url == *url =>
            {
                match result {
                    Ok(GetHTTPSResponse { domain, port, .. }) => {
                        let remote_url = Url::parse(&format!("https://{domain}:{port}")).ok();
                        eq_update(&mut self.remote_url, remote_url)
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
            Msg::Internal(Internal::UpdateStreamSource { stream }) => {
                let effects =
                    update_stream_source_effects::<E>(self.base_url.as_ref(), stream.to_owned());

                effects
            }
            // Msg::Internal(Internal::StreamingServerStreamSourceResult { original, result }) => {}
            _ => Effects::none().unchanged(),
        }
    }
}

fn get_settings<E: Env + 'static>(url: &Url) -> Effect {
    let endpoint = url.join("settings").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, SettingsResponse>(request)
            .map(enclose!((url) move |result| {
                Msg::Internal(Internal::StreamingServerSettingsResult(
                    url, result,
                ))
            }))
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

fn get_network_info<E: Env + 'static>(url: &Url) -> Effect {
    let endpoint = url.join("network-info").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, NetworkInfo>(request)
            .map_ok(|resp| resp)
            .map(enclose!((url) move |result|
                Msg::Internal(Internal::StreamingServerNetworkInfoResult(url, result))
            ))
            .boxed_env(),
    )
    .into()
}

fn get_device_info<E: Env + 'static>(url: &Url) -> Effect {
    let endpoint = url.join("device-info").expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .body(())
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, DeviceInfo>(request)
            .map_ok(|resp| resp)
            .map(enclose!((url) move |result|
                Msg::Internal(Internal::StreamingServerDeviceInfoResult(url, result))
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
        remote_https: Option<String>,
        transcode_profile: Option<String>,
    }
    let body = Body {
        cache_size: settings.cache_size.to_owned(),
        bt_max_connections: settings.bt_max_connections.to_owned(),
        bt_handshake_timeout: settings.bt_handshake_timeout.to_owned(),
        bt_request_timeout: settings.bt_request_timeout.to_owned(),
        bt_download_speed_soft_limit: settings.bt_download_speed_soft_limit.to_owned(),
        bt_download_speed_hard_limit: settings.bt_download_speed_hard_limit.to_owned(),
        bt_min_peers_for_stable: settings.bt_min_peers_for_stable.to_owned(),
        remote_https: settings.remote_https.to_owned(),
        transcode_profile: settings.transcode_profile.to_owned(),
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

pub async fn create_magnet_request<E: Env + 'static>(url: &Url, info_hash: InfoHash, announce: &[String]) -> Result<(), EnvError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Torrent {
        info_hash: InfoHash,
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

pub fn create_torrent_request<E: Env + 'static>(
    url: &Url,
    info_hash: &str,
    torrent: &[u8],
) -> Effect {
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

    // It's happening when the engine is destroyed for inactivity:
    // If it was downloaded to 100% and that the stream is paused, then played,
    // it will create a new engine and return the correct stats
    EffectFuture::Concurrent(
        E::fetch::<_, Option<Statistics>>(request)
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

fn get_https_endpoint<E: Env + 'static>(
    url: &Url,
    auth_key: &AuthKey,
    ip_address: &String,
) -> Effect {
    let endpoint = url
        .join(&format!(
            "/get-https?authKey={}&ipAddress={}",
            auth_key, ip_address,
        ))
        .expect("url builder failed");
    let request = Request::get(endpoint.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(())
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, GetHTTPSResponse>(request)
            .map(enclose!((url) move |result|
                Msg::Internal(Internal::StreamingServerGetHTTPSResult(url, result))
            ))
            .boxed_env(),
    )
    .into()
}

fn update_remote_url<E: Env + 'static>(
    remote_url: &mut Option<Url>,
    selected: &Selected,
    settings: &Settings,
    ctx: &Ctx,
) -> Effects {
    match (settings.remote_https.as_ref(), ctx.profile.auth_key()) {
        (Some(ip_address), Some(auth_key)) if !ip_address.is_empty() => Effects::one(
            get_https_endpoint::<E>(&selected.transport_url, auth_key, ip_address),
        )
        .unchanged(),
        _ => eq_update(remote_url, None),
    }
}

/// TODO: <https://github.com/Stremio/stremio-video/blob/5c50f9caa5bee2a66b5c6d3bbc967536a1305cbb/src/withStreamingServer/convertStream.js>
pub fn convert_stream<E: Env + 'static>(
    transport_url: Option<&Url>,
    stream: Stream,
) -> Result<Option<Stream<ConvertedStreamSource>>, EnvError> {
    // TODO: Convert the StreamSource to ConvertedStreamSource, i.e. resolved and setup stream in the server
    let converted_stream = match &stream.source {
        StreamSource::Url { url } => ConvertedStreamSource::Url(url.clone()),
        StreamSource::Url { url } if url.scheme() == "magnet" => {
            let magnet = stream
                .magnet_url()
                .ok_or(EnvError::Other("Not a valid magent url".to_owned()))?;

            let info_hash = todo!("from magnet");
            // let sources = ;
            // magnet.bti
            let series_info = SeriesInfo {
                season: todo!(),
                episode: todo!(),
            };

            let magnet_url = todo!();
            // let magnet_url = create_torrent_request(url, info_hash, torrent);

            // .and_then(|magnet_url| Url::parse(&magnet_url.to_string())
            // .map_err(|_err| EnvError::Other("Magnets should always be a valid url!".to_owned())))?;
            // TODO: Create torrent
            // Magnet

            //     var parsedMagnetURI;
            //     try {
            //         parsedMagnetURI = magnet.decode(stream.url);
            //         if (!parsedMagnetURI || typeof parsedMagnetURI.infoHash !== 'string') {
            //             throw new Error('Failed to decode magnet url');
            //         }
            //     } catch (error) {
            //         reject(error);
            //         return;
            //     }

            //     var sources = Array.isArray(parsedMagnetURI.announce) ?
            //         parsedMagnetURI.announce.map(function(source) {
            //             return 'tracker:' + source;
            //         })
            //         :
            //         [];
            //     createTorrent(streamingServerURL, parsedMagnetURI.infoHash, null, sources, seriesInfo)
            //         .then(function(torrent) {
            //             resolve({ url: torrent.url, infoHash: torrent.infoHash, fileIdx: torrent.fileIdx });
            //         })
            //         .catch(function(error) {
            //             reject(error);
            //         });
            // } else {
            //     resolve({ url: stream.url });
            // }

            // createTorrent(streamingServerURL, parsedMagnetURI.infoHash, null, sources, seriesInfo)
            // .then(function(torrent) {
            //     resolve({ url: torrent.url, infoHash: torrent.infoHash, fileIdx: torrent.fileIdx });
            // })
            ConvertedStreamSource::Url(todo!())
        }
        StreamSource::Torrent {
            info_hash,
            file_idx,
            announce,
            ..
        } => {
            // Torrent
            // createTorrent(streamingServerURL, stream.infoHash, stream.fileIdx, stream.announce, seriesInfo)
            // .then(function(torrent) {
            //     resolve({ url: torrent.url, infoHash: torrent.infoHash, fileIdx: torrent.fileIdx });

            todo!("Create torrent in server and create torrent when magnet!");
            ConvertedStreamSource::Torrent {
                url: todo!("Convert the stream by resolving it with the server"),
                info_hash: InfoHash::new(*info_hash),
                file_idx: *file_idx,
                announce: announce.to_owned(),
            }
        }
        _ => return Ok(None),
    };

    match transport_url {
        Some(server_url) => Ok(Some(Stream {
            source: converted_stream,
            name: stream.name,
            description: stream.description,
            thumbnail: stream.thumbnail,
            subtitles: stream.subtitles,
            behavior_hints: stream.behavior_hints,
        })),
        _ => Err(EnvError::Other("Fix me!".into())),
    }
}
/// <https://github.com/Stremio/stremio-video/blob/5c50f9caa5bee2a66b5c6d3bbc967536a1305cbb/src/withStreamingServer/createTorrent.js#L18>
pub async fn create_torrent<E: Env + 'static>(
    mut server_url: Url,
    info_hash: InfoHash,
    file_idx: u64,
    sources: &[Url],
    // series_info: SeriesInfo,
) -> Result<Option<CreatedTorrent>, EnvError> {

    let request = CreateTorrentRequest {
        server_url,
        sources: sources.to_vec(),
        info_hash,
        file_idx,
    };

    // returns Null on invalid params
    Ok(E::fetch(request.into()).await?)
    
    // create_torrent_request
    // todo!()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct CreatedTorrent {
    pub torrent: Torrent,
    pub peer_search: PeerSearch,
    // Stremio-video sends `false` when no Guessing should be done.
    // #[serde_as(deserialize_as = "DefaultOnError")]
    /// If `None, the server will perform no guessing
    pub guess_file_idx: Option<SeriesInfo>,
}

// #[derive(Default, Debug, Clone, Serialize, Deserialize)]
// pub enum GuessFileIdx {
//     #[default]
//     NoGuessing,
//     SeriesInfo(SeriesInfo)
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Torrent {
    info_hash: InfoHash,
}


/// Updates the stream_source and generates a correct streaming_url
/// video params in stremio-video: <https://github.com/Stremio/stremio-video/blob/master/src/withVideoParams/withVideoParams.js#L48-L57>
///
/// ```js
/// var hash = stream.behaviorHints && typeof stream.behaviorHints.videoHash === 'string' ? stream.behaviorHints.videoHash : null;
/// var size = stream.behaviorHints && stream.behaviorHints.videoSize !== null && isFinite(stream.behaviorHints.videoSize) ? stream.behaviorHints.videoSize : null;
/// var filename = stream.behaviorHints && typeof stream.behaviorHints.filename === 'string' ? stream.behaviorHints.filename : null;
/// return { hash: hash, size: size, filename: filename };
/// ```
pub async fn get_video_params_effects<E: Env + 'static>(
    transport_url: Option<&Url>,
    stream: Stream<ConvertedStreamSource>,
) -> Result<Option<VideoParams>, EnvError> {
    let streaming_server_url = transport_url.cloned();
    let stream_opensubtitles = stream.clone();
    let opensubitiles_params_fut = async move {
        match (
            transport_url,
            stream_opensubtitles.behavior_hints.video_hash.clone(),
            stream_opensubtitles.behavior_hints.video_size.clone(),
        ) {
            (_, Some(info_hash), Some(size)) => {
                let info_hash = info_hash
                    .parse::<InfoHash>()
                    .map_err(|e| EnvError::Serde(e.to_string()))?;

                Ok(Some((info_hash, size)))
            }
            (Some(transport_url), _, _) => {
                let media_url = match stream_opensubtitles.source.clone() {
                    ConvertedStreamSource::Url(url) => url.to_owned(),
                    ConvertedStreamSource::Torrent { url, .. } => url.to_owned(),
                };

                get_opensubtitles_params::<E>(transport_url.to_owned(), media_url)
                    .await
                    .map(|response| response.map(|response| (response.hash, response.size)))
            }
            _ => Ok(None),
        }
    };

    let filename_fut = async move {
        match (
            transport_url.to_owned(),
            stream.source,
            stream.behavior_hints.filename,
        ) {
            (_, _, Some(filename)) => Ok(Some(filename)),
            (
                Some(transport_url),
                ConvertedStreamSource::Torrent {
                    info_hash,
                    file_idx: Some(file_idx),
                    ..
                },
                _,
            ) => get_filename::<E>(transport_url.to_owned(), info_hash, file_idx).await,
            _ => Ok(None),
        }
    };

    match futures::future::try_join(opensubitiles_params_fut, filename_fut).await? {
        (Some((info_hash, size)), Some(filename)) => Ok(Some(VideoParams {
            // todo: improve VideoParams struct!
            hash: Some(info_hash.to_string()),
            size: Some(size),
            filename: Some(filename),
        })),
        _ => Ok(None),
    }
}

pub async fn get_opensubtitles_params<E: Env + 'static>(
    transport_url: Url,
    media_url: Url,
) -> Result<Option<OpensubtitlesParamsResponse>, EnvError> {
    let request = OpensubtitlesParamsRequest {
        server_url: transport_url,
        media_url,
    };
    // var queryParams = new URLSearchParams([['videoUrl', mediaURL]]);
    // return fetch(url.resolve(streamingServerURL, '/opensubHash?' + queryParams.toString()))

    // returns Null on invalid params
    Ok(E::fetch(request.into()).await?)
}

/// Get the filename of the played torrent.
///
/// We need both the `info_hash` and the `file_idx` to pass to the server when
/// requesting Statistics for the filename.
///
/// # Returns
/// Returns the [`Statistics::stream_name`] field.
///
/// `None` if stats.json returns `null`, e.g. torrent has fully loaded.
pub async fn get_filename<E: Env + 'static>(
    transport_url: Url,
    info_hash: InfoHash,
    file_idx: u16,
) -> Result<Option<String>, EnvError> {
    let request = FileNameRequest {
        server_url: transport_url,
        request: StatisticsRequest {
            info_hash: info_hash.to_string(),
            file_idx,
        },
    };

    let statistics: Option<Statistics> = E::fetch(request.into()).await?;

    Ok(statistics.map(|statistics| statistics.stream_name))
}

/// Updates the stream_source and generates a correct streaming_url
pub fn update_stream_source_effects<E: Env + 'static>(
    transport_url: Option<&Url>,
    stream: Stream,
) -> Effects {
    let streaming_server_url = transport_url.cloned();

    let effect = EffectFuture::Concurrent(
        update_stream_source_streaming_url::<E>(streaming_server_url.clone(), stream.clone())
            .map(enclose!((stream) move |result|
                Msg::Internal(Internal::StreamingServerStreamSourceResult{ original: stream, result, streaming_server_url })
            ))
            .boxed_env()
    );

    Effects::one(effect.into())
}

/// Updates a StreamSource if it's RAR or ZIP urls by calling the server (if present)
/// and creating a streaming url for a given file (either with `file_idx` or `file_must_include`).
pub async fn update_stream_source_streaming_url<E: Env + 'static>(
    transport_url: Option<Url>,
    mut stream: Stream,
) -> Result<Stream, EnvError> {
    match (transport_url, stream.source.to_owned()) {
        (
            Some(streaming_server_url),
            StreamSource::Rar {
                file_idx,
                file_must_include,
                rar_urls,
            },
        ) => {
            if rar_urls.is_empty() {
                return Err(EnvError::Other("No RAR URLs provided".into()));
            }

            // error on Url::join should never happen.
            let create_stream_url = streaming_server_url
                .join("rar/stream")
                .map_err(|err| EnvError::Other(err.to_string()))?;

            let request = Request::post(create_stream_url.as_str())
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(rar_urls)
                .expect("request builder failed");

            let response = E::fetch::<_, ArchiveCreateResponse>(request).await?;
            let response_key = response
                .key
                .filter(|key| !key.is_empty())
                .ok_or_else(|| EnvError::Other("Could not create RAR key".into()))?;

            let mut stream_url = streaming_server_url
                .join("rar/stream")
                .map_err(|err| EnvError::Other(err.to_string()))?;

            let query_pairs = ArchiveStreamRequest {
                response_key,
                options: ArchiveStreamOptions {
                    file_idx,
                    file_must_include,
                },
            }
            .to_query_pairs();
            stream_url.query_pairs_mut().extend_pairs(query_pairs);

            stream.source = StreamSource::Url { url: stream_url };
            Ok(stream)
        }
        (
            Some(streaming_server_url),
            StreamSource::Zip {
                file_idx,
                file_must_include,
                zip_urls,
            },
        ) => {
            if zip_urls.is_empty() {
                return Err(EnvError::Other("No RAR URLs provided".into()));
            }

            let create_stream_url = streaming_server_url
                .join("zip/create")
                .map_err(|err| EnvError::Other(err.to_string()))?;

            let request = Request::post(create_stream_url.as_str())
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(zip_urls)
                .expect("request builder failed");

            let response = E::fetch::<_, ArchiveCreateResponse>(request).await?;
            let response_key = response
                .key
                .filter(|key| !key.is_empty())
                .ok_or_else(|| EnvError::Other("Could not create RAR key".into()))?;

            let mut stream_url = streaming_server_url
                .join("zip/stream")
                .map_err(|err| EnvError::Other(err.to_string()))?;

            let query_pairs = ArchiveStreamRequest {
                response_key,
                options: ArchiveStreamOptions {
                    file_idx,
                    file_must_include,
                },
            }
            .to_query_pairs();
            stream_url.query_pairs_mut().extend_pairs(query_pairs);

            stream.source = StreamSource::Url { url: stream_url };
            Ok(stream)
        }
        // no further changes are needed
        (_, _stream_source) => Ok(stream),
    }
}

#[cfg(test)]
mod tests {
    use magnet_url::Magnet;

    #[test]
    fn test_magnet_hash() {
        let magnet = Magnet::new("magnet:?xt=urn:btih:0d54e2339706f173ac20f4effb4ad42d9c7a84e9&dn=Halo.S02.1080p.WEBRip.x265.DDP5.1.Atmos-WAR");

        dbg!(magnet);
    }
}
