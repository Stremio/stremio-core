use crate::state_types::msg::Internal::{CtxLoaded, StreamingServerSettingsLoaded};
use crate::state_types::msg::{Action, ActionSettings, Event};
use crate::state_types::{Ctx, Effects, Environment, Msg, Request, UpdateWithCtx};
use futures::future::Future;
use lazy_static::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Debug};

extern crate web_sys;

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
    Custom,
}
impl SsProfileName {
    fn default_profile() -> Self {
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
    pub bt_profile: Option<String>,
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
            bt_profile: Some(SsProfileName::default_profile().to_string()),
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
    pub is_loaded: bool,
    pub cache_size: String,
    pub profile: SsProfileName,
}

impl Default for StreamingServerSettings {
    fn default() -> Self {
        StreamingServerSettings {
            is_loaded: false,
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
        let mut profiles = HashMap::new();
        profiles.insert(
            SsProfileName::Default,
            SsProfileParams {
                bt_max_connections: 35,
                bt_handshake_timeout: 20000,
                bt_request_timeout: 4000,
                bt_download_speed_soft_limit: 1677721.6,
                bt_download_speed_hard_limit: 2621440.0,
                bt_min_peers_for_stable: 5,
            },
        );
        profiles.insert(
            SsProfileName::Soft,
            SsProfileParams {
                bt_max_connections: 35,
                bt_handshake_timeout: 20000,
                bt_request_timeout: 4000,
                bt_download_speed_soft_limit: 1677721.6,
                bt_download_speed_hard_limit: 1677721.6,
                bt_min_peers_for_stable: 5,
            },
        );
        profiles.insert(
            SsProfileName::Fast,
            SsProfileParams {
                bt_max_connections: 200,
                bt_handshake_timeout: 20000,
                bt_request_timeout: 4000,
                bt_download_speed_soft_limit: 4194304.0,
                bt_download_speed_hard_limit: 39321600.0,
                bt_min_peers_for_stable: 10,
            },
        );
        profiles
    };
}

fn get_settings_endpoint(server_url: &String) -> String {
    format!("{}{}", server_url, "settings")
}

fn to_profile_enum(str_profile: &Option<String>) -> SsProfileName {
    match str_profile {
        Some(str_profile) => match &str_profile[..] {
            "default" => SsProfileName::Default,
            "soft" => SsProfileName::Soft,
            "fast" => SsProfileName::Fast,
            _ => SsProfileName::Custom,
        },
        None => SsProfileName::Custom,
    }
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for StreamingServerSettings {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        // web_sys::console::log_1(&format!("Update Settings!").into());
        match msg {
            // This is triggered after loading the settings from local storage
            Msg::Internal(CtxLoaded(_))
            | Msg::Action(Action::Settings(ActionSettings::LoadStreamingServer)) => {
                web_sys::console::log_1(&format!("Load Ss Settings!").into());
                let url = get_settings_endpoint(&ctx.content.settings.server_url);
                match Request::get(url).body(()).ok() {
                    Some(resp) => Effects::one(Box::new(
                        Env::fetch_serde::<_, SsSettings>(resp)
                            .and_then(|settings: SsSettings| {
                                let is_custom_profile = match PROFILES
                                    .get(&to_profile_enum(&settings.values.bt_profile))
                                {
                                    Some(bt_params) => match settings.values.bt_params {
                                        Some(remote_bt_params) => remote_bt_params != *bt_params,
                                        None => true,
                                    },
                                    None => true,
                                };
                                let settings = if is_custom_profile {
                                    let mut settings = settings.to_owned();
                                    settings.values.bt_profile =
                                        Some(SsProfileName::Custom.to_string());
                                    settings
                                } else {
                                    settings
                                };
                                Ok(Msg::Internal(StreamingServerSettingsLoaded(settings)))
                            })
                            .or_else(|e| {
                                web_sys::console::log_1(
                                    &format!("Streaming server settings error: {}", e).into(),
                                );
                                Err(Msg::Event(Event::CtxFatal(e.into())))
                            }),
                    )),
                    None => {
                        self.is_loaded = true;
                        Effects::none()
                    }
                }
            }
            Msg::Internal(StreamingServerSettingsLoaded(settings)) => {
                let settings = settings.to_owned();
                self.cache_size = match settings.values.cache_size {
                    Some(size) => size.to_string(),
                    None => "Infinity".to_string(),
                };
                self.profile = to_profile_enum(&settings.to_owned().values.bt_profile);
                self.is_loaded = true;
                // Perhaps dispatch custom event for streaming_server_settings_loaded
                Effects::none()
            }
            Msg::Action(Action::Settings(ActionSettings::StoreStreamingServer(settings))) => {
                // The format for the streaming server settings is basically SsValues,
                // where the omitted values stay unchanged
                let url = get_settings_endpoint(&ctx.content.settings.server_url);
                let settings = settings.to_owned();
                let bt_params = PROFILES
                    .get(&settings.profile)
                    .and_then(|params| Some(*params));
                let values = SsValues {
                    cache_size: settings.cache_size.parse::<f64>().ok(),
                    bt_profile: Some(settings.profile.to_string()),
                    bt_params,
                    ..Default::default()
                };
                match Request::post(url)
                    .header("content-type", "application/json")
                    .body(values)
                    .ok()
                {
                    Some(resp) => Effects::one(Box::new(
                        Env::fetch_serde::<_, SsResponse>(resp)
                            .and_then(|s_resp: SsResponse| {
                                web_sys::console::log_1(
                                    &format!(
                                        "Streaming server settings stored: {}",
                                        s_resp.success
                                    )
                                    .into(),
                                );
                                Ok(Msg::Action(Action::Settings(
                                    ActionSettings::LoadStreamingServer,
                                )))
                            })
                            .or_else(|e| {
                                web_sys::console::log_1(
                                    &format!("Streaming server settings error: {}", e).into(),
                                );
                                Err(Msg::Event(Event::CtxFatal(e.into())))
                            }),
                    )),
                    None => Effects::none().unchanged(),
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}
