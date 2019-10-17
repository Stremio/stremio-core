use crate::state_types::msg::{Action, ActionSettings, Event};
use crate::state_types::msg::Internal::{CtxLoaded, StreamingServerSettingsLoaded};
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

#[derive(Deserialize, Serialize, Clone, Debug)]
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
    pub server_version: String,
    pub app_path: String,
    pub cache_root: String,
    pub cache_size: f64,
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: f64,
    #[serde(default = "SsProfile::default_profile")]
    pub bt_profile: SsProfile,
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

fn fetch_server_settings(local_settings: &Settings) -> Option<Request<()>> {
    let url = format!("{}{}", local_settings.server_url, "settings");
    match Request::get(url).body(()) {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamingServerSettings {
    pub is_loaded: bool,
    pub cache_size: String,
    pub profile: String,
}

impl Default for StreamingServerSettings {
    fn default() -> Self {
        StreamingServerSettings {
            is_loaded: false,
            cache_size: "2000".to_string(),
            profile: SsProfile::default_profile()
                .to_string()
                .to_ascii_lowercase(),
        }
    }
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for StreamingServerSettings {
    fn update(&mut self, _ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        // web_sys::console::log_1(&format!("Update Settings!").into());
        match msg {
            // This is triggered after loading the settings from local storage
            Msg::Internal(CtxLoaded(opt_content)) => {
                web_sys::console::log_1(&format!("Load Ss Settings!").into());
                match fetch_server_settings(&opt_content.to_owned().unwrap_or_default().settings) {
                    Some(resp) => {
                        // web_sys::console::log_1(&format!("Something").into());
                        Effects::one(Box::new(
                            Env::fetch_serde::<_, SsSettings>(resp)
                                .and_then(|settings: SsSettings| {
                                    // web_sys::console::log_1(&format!("We have settings {}", settings.base_url ).into());
                                    Ok(Msg::Internal(StreamingServerSettingsLoaded(settings)))
                                })
                                .or_else(|e| {
                                    web_sys::console::log_1(&format!("Streaming server settings error: {}", e).into());
                                    Err(Msg::Event(Event::CtxFatal(e.into())))
                                }),
                        ))
                    }
                    None => {
                        // web_sys::console::log_1(&format!("Nothing").into());
                        self.is_loaded = true;
                        Effects::none()
                    }
                }
            }
            Msg::Internal(StreamingServerSettingsLoaded(settings)) => {
                self.cache_size = settings.values.cache_size.to_string();
                self.is_loaded = true;
                // Perhaps dispatch custom event for streaming_server_settings_loaded
                Effects::none()
            }
            Msg::Action(Action::Settings(ActionSettings::StoreStreamingServer(settings))) => {
                web_sys::console::log_1(&format!("We have new settings {}", settings.cache_size ).into());
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}
