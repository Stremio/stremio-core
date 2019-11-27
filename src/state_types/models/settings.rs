use crate::state_types::models::Ctx;
use crate::state_types::msg::Internal::{
    CtxLoaded, StreamingServerSettingsErrored, StreamingServerSettingsLoaded,
};
use crate::state_types::msg::{Action, ActionSettings, Event, Msg};
use crate::state_types::{Effects, Environment, Request, UpdateWithCtx};
use futures::future::Future;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
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

// These are the user settings from local storage.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Settings {
    pub language: String,
    pub subtitles_size: String,
    pub subtitles_language: String,
    pub subtitles_background: String,
    pub subtitles_color: String,
    pub subtitles_outline_color: String,
    pub autoplay_next_vid: String,
    pub server_url: String,
    pub use_external_player: String,
    // We can't override Esc in browser so this option is pointless here
    // pub player_esc_exits_fullscreen:  String,
    pub pause_on_lost_focus: String,
    pub show_vid_overview: String,
}

impl Settings {
    fn get_endpoint(&self) -> String {
        Path::new(&self.server_url)
            .join("settings")
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| {
                Path::new(&Settings::default().server_url)
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
            language: "eng".to_string(),
            subtitles_size: "100%".to_string(),
            subtitles_language: "eng".to_string(),
            subtitles_background: "".to_string(),
            subtitles_color: "#fff".to_string(),
            subtitles_outline_color: "#000".to_string(),
            autoplay_next_vid: "false".to_string(),
            server_url: "http://127.0.0.1:11470/".to_string(),
            use_external_player: "false".to_string(),
            pause_on_lost_focus: "false".to_string(),
            show_vid_overview: "false".to_string(),
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
