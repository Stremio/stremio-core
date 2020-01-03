use crate::state_types::messages::Internal::{
    CtxLoaded, StreamingServerSettingsErrored, StreamingServerSettingsLoaded,
};
use crate::state_types::messages::{Action, ActionSettings, Event, Msg};
use crate::state_types::models::common::RGBA;
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, Request, UpdateWithCtx};
use futures::future::Future;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::path::Path;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SsOption {
    pub id: String,
    pub label: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum SsProfileName {
    Default,
    Soft,
    Fast,
    #[serde(other)]
    Custom,
}

impl Default for SsProfileName {
    fn default() -> Self {
        SsProfileName::Default
    }
}

impl fmt::Display for SsProfileName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SsProfileParams {
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsValues {
    #[serde(skip_serializing)]
    pub server_version: Option<String>,
    #[serde(skip_serializing)]
    pub app_path: Option<String>,
    #[serde(skip_serializing)]
    pub cache_root: Option<String>,
    pub cache_size: Option<f64>,
    #[serde(default)]
    pub bt_profile: SsProfileName,
    #[serde(flatten)]
    pub bt_params: Option<SsProfileParams>,
}
impl Default for SsValues {
    fn default() -> Self {
        SsValues {
            server_version: None,
            app_path: None,
            cache_root: None,
            cache_size: None,
            bt_profile: SsProfileName::default(),
            bt_params: None,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsSettings {
    pub options: Vec<SsOption>,
    pub values: SsValues,
    pub base_url: String,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum SubtitlesSize {
    S = 1,
    M = 2,
    L = 3,
    XL = 4,
    XXL = 5,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Settings {
    pub interface_language: String,
    pub binge_watching: bool,
    pub play_in_background: bool,
    pub play_in_external_player: bool,
    pub streaming_server_url: String,
    pub subtitles_language: String,
    pub subtitles_size: SubtitlesSize,
    pub subtitles_background_color: RGBA,
    pub subtitles_text_color: RGBA,
    pub subtitles_outline_color: RGBA,
}

impl Settings {
    fn get_endpoint(&self) -> String {
        Path::new(&self.streaming_server_url)
            .join("settings")
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| {
                Path::new(&Settings::default().streaming_server_url)
                    .join("settings")
                    .into_os_string()
                    .into_string()
                    .expect("Default streaming server endpint is broken")
            })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            interface_language: "eng".to_owned(),
            binge_watching: false,
            play_in_background: true,
            play_in_external_player: false,
            streaming_server_url: "http://127.0.0.1:11470/".to_owned(),
            subtitles_language: "eng".to_owned(),
            subtitles_size: SubtitlesSize::M,
            subtitles_background_color: RGBA::transparent(),
            subtitles_text_color: RGBA::new(255, 255, 255, 0),
            subtitles_outline_color: RGBA::transparent(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamingServerSettings {
    pub cache_size: String,
    pub profile: SsProfileName,
}

impl Default for StreamingServerSettings {
    fn default() -> Self {
        StreamingServerSettings {
            cache_size: "2147483648".to_string(),
            profile: SsProfileName::Default,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
struct SsResponse {
    success: bool,
}

/*
End of data structure defs
*/

lazy_static! {
    static ref PROFILES: HashMap<SsProfileName, SsProfileParams> = {
        [
            (
                SsProfileName::Default,
                SsProfileParams {
                    bt_max_connections: 35,
                    bt_handshake_timeout: 20000,
                    bt_request_timeout: 4000,
                    bt_download_speed_soft_limit: 1_677_721.6,
                    bt_download_speed_hard_limit: 2_621_440.0,
                    bt_min_peers_for_stable: 5,
                },
            ),
            (
                SsProfileName::Soft,
                SsProfileParams {
                    bt_max_connections: 35,
                    bt_handshake_timeout: 20000,
                    bt_request_timeout: 4000,
                    bt_download_speed_soft_limit: 1_677_721.6,
                    bt_download_speed_hard_limit: 1_677_721.6,
                    bt_min_peers_for_stable: 5,
                },
            ),
            (
                SsProfileName::Fast,
                SsProfileParams {
                    bt_max_connections: 200,
                    bt_handshake_timeout: 20000,
                    bt_request_timeout: 4000,
                    bt_download_speed_soft_limit: 4_194_304.0,
                    bt_download_speed_hard_limit: 39_321_600.0,
                    bt_min_peers_for_stable: 10,
                },
            ),
        ]
        .iter()
        .cloned()
        .collect()
    };
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StreamingServerSettingsModel {
    NotLoaded,
    Loading,
    Ready(StreamingServerSettings),
    Error(String),
}

impl Default for StreamingServerSettingsModel {
    fn default() -> Self {
        StreamingServerSettingsModel::NotLoaded
    }
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for StreamingServerSettingsModel {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            // This is triggered after loading the settings from local storage
            Msg::Internal(CtxLoaded(_))
            | Msg::Action(Action::Settings(ActionSettings::LoadStreamingServer)) => {
                *self = StreamingServerSettingsModel::Loading;
                let url = &ctx.content.settings.get_endpoint();
                match Request::get(url).body(()) {
                    Ok(req) => Effects::one(Box::new(
                        Env::fetch_serde::<_, SsSettings>(req)
                            .and_then(|settings: SsSettings| {
                                let is_custom_profile = PROFILES.get(&settings.values.bt_profile)
                                    != settings.values.bt_params.as_ref();
                                let settings = if is_custom_profile {
                                    let mut settings = settings;
                                    settings.values.bt_profile = SsProfileName::Custom;
                                    settings
                                } else {
                                    settings
                                };
                                Ok(Msg::Internal(StreamingServerSettingsLoaded(settings)))
                            })
                            .or_else(|e| {
                                Ok(Msg::Internal(StreamingServerSettingsErrored(format!(
                                    "{}",
                                    e
                                ))))
                            }),
                    )),
                    Err(e) => {
                        *self = StreamingServerSettingsModel::Error(format!("{}", e));
                        Effects::none()
                    }
                }
            }
            Msg::Internal(StreamingServerSettingsLoaded(settings)) => {
                *self = StreamingServerSettingsModel::Ready(StreamingServerSettings {
                    cache_size: match settings.values.cache_size {
                        Some(size) => size.to_string(),
                        None => "Infinity".to_string(),
                    },
                    profile: settings.values.bt_profile.clone(),
                });
                // Perhaps dispatch custom event for streaming_server_settings_loaded
                Effects::none()
            }
            Msg::Action(Action::Settings(ActionSettings::StoreStreamingServer(settings))) => {
                // The format for the streaming server settings is basically SsValues,
                // where the omitted values stay unchanged
                let url = &ctx.content.settings.get_endpoint();
                let values = SsValues {
                    cache_size: settings.cache_size.parse::<f64>().ok(),
                    bt_profile: settings.profile.clone(),
                    bt_params: PROFILES.get(&settings.profile).copied(),
                    ..Default::default()
                };
                match Request::post(url)
                    .header("content-type", "application/json")
                    .body(values)
                    .ok()
                {
                    Some(req) => Effects::one(Box::new(
                        Env::fetch_serde::<_, SsResponse>(req)
                            .and_then(|s_resp: SsResponse| {
                                Ok(if s_resp.success {
                                    Msg::Action(Action::Settings(
                                        ActionSettings::LoadStreamingServer,
                                    ))
                                } else {
                                    Msg::Event(Event::SettingsStoreError(
                                        "Couldn't store the settings".to_string(),
                                    ))
                                })
                            })
                            .or_else(|e| {
                                Ok(Msg::Internal(StreamingServerSettingsErrored(format!(
                                    "{}",
                                    e
                                ))))
                            }),
                    )),
                    None => Effects::none().unchanged(),
                }
            }
            Msg::Internal(StreamingServerSettingsErrored(error)) => {
                *self = StreamingServerSettingsModel::Error(error.to_owned());
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}
