//use futures::future::Future;
use serde::{Deserialize, Serialize};
//use crate::state_types::msg::Internal::*;
//use crate::state_types::Internal::*;
use crate::state_types::*;
//use std::collections::HashMap;

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

// fn fetch_server_settings(local_settings: &HashMap<String, String>) -> Option<Request<()>> {
//     let url = format!("{}{}", local_settings.get("server_url")?, "settings");
//     match Request::get(url).body(()) {
//         Ok(res) => Some(res),
//         Err(_) => None,
//     }
// }

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamingServerSettings {
    pub cache_size: String,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for StreamingServerSettings {
    fn update(&mut self, _ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Settings(ActionSettings::LoadStreamingServer)) => {
                // let smth = match fetch_server_settings(&self.content.settings) {
                //     Some(resp) => {
                //         // self.content.settings.insert("fetched".to_string(), "yes".to_string());
                //                 web_sys::console::log_1(&format!("Something").into());
                //          let _ad = Env::fetch_serde::<_, SsSettings>(resp)
                //             .and_then(|settings: SsSettings| {
                //                 web_sys::console::log_1(&format!("We have settings").into());
                //                 // web_sys::console::log_1(&format!("{}", settings.base_url ).into());
                //                 Ok(settings)
                //             }).or_else(|e| {
                //                 web_sys::console::log_1(&format!("{}", e ).into());
                //                 Err(e)
                //             });
                //     }
                //     None => {
                //                 web_sys::console::log_1(&format!("Nothing").into());
                //     }
                // };
                Effects::none().unchanged()
            }
            _ => Effects::none().unchanged(),
        }
    }
}