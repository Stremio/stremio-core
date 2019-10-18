use crate::state_types::msg::Internal::{CtxLoaded, StreamingServerSettingsLoaded};
use crate::state_types::msg::{Action, ActionSettings, Event};
use crate::state_types::{Ctx, Effects, Environment, Msg, Request, UpdateWithCtx};
use futures::future::Future;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

extern crate web_sys;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SsOption {
    pub id: String,
    pub label: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SsProfile {
    Default,
    Soft,
    Fast,
    Custom,
}
impl SsProfile {
    fn default_profile() -> Self {
        SsProfile::Default
    }
}
impl fmt::Display for SsProfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bt_max_connections: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bt_handshake_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bt_request_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bt_download_speed_soft_limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bt_download_speed_hard_limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bt_min_peers_for_stable: Option<f64>,
    #[serde(default = "SsProfile::default_profile")]
    pub bt_profile: SsProfile,
}
impl Default for SsValues {
    fn default() -> Self {
        SsValues {
            server_version: None,
            app_path: None,
            cache_root: None,
            cache_size: None,
            bt_max_connections: None,
            bt_handshake_timeout: None,
            bt_request_timeout: None,
            bt_download_speed_soft_limit: None,
            bt_download_speed_hard_limit: None,
            bt_min_peers_for_stable: None,
            bt_profile: SsProfile::default_profile(),
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
    pub profile: SsProfile,
}

impl Default for StreamingServerSettings {
    fn default() -> Self {
        StreamingServerSettings {
            is_loaded: false,
            cache_size: "2147483648".to_string(),
            profile: SsProfile::Default,
        }
    }
}

fn get_settings_endpoint(server_url: &String) -> String {
    format!("{}{}", server_url, "settings")
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
struct SsResponse {
    success: bool,
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
                self.profile = settings.to_owned().values.bt_profile;
                self.is_loaded = true;
                // Perhaps dispatch custom event for streaming_server_settings_loaded
                Effects::none()
            }
            Msg::Action(Action::Settings(ActionSettings::StoreStreamingServer(settings))) => {
                // The format for the streaming server settings is basically SsValues,
                // where the omitted values stay unchanged
                let url = get_settings_endpoint(&ctx.content.settings.server_url);
                let settings = settings.to_owned();

                // TODO: set all bt_fields according to the selected profile
                let values = SsValues {
                    cache_size: settings.cache_size.parse::<f64>().ok(),
                    bt_profile: settings.profile,
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
