use crate::state_types::msg::Internal::{CtxLoaded, StreamingServerSettingsLoaded};
use crate::state_types::msg::Event;
use crate::state_types::*;
use futures::future::Future;
use serde::{Deserialize, Serialize};

extern crate web_sys;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SsOption {
    pub id: String,
    pub label: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsValues {
    pub app_path: String,
    pub cache_size: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsSettings {
    pub options: Vec<SsOption>,
    pub values: SsValues,
    pub base_url: String,
}

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
}

impl Default for StreamingServerSettings {
    fn default() -> Self {
        StreamingServerSettings{
            is_loaded: false,
            cache_size: "2000".to_string(),
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
                                    // web_sys::console::log_1(&format!("{}", e).into());
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
            _ => Effects::none().unchanged(),
        }
    }
}
