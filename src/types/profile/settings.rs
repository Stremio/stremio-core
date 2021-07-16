use crate::constants::STREAMING_SERVER_URL;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Settings {
    pub interface_language: String,
    pub streaming_server_url: Url,
    pub binge_watching: bool,
    pub play_in_background: bool,
    pub play_in_external_player: bool,
    pub hardware_decoding: bool,
    pub subtitles_language: String,
    pub subtitles_size: u8,
    pub subtitles_font: String,
    pub subtitles_bold: bool,
    pub subtitles_offset: u8,
    pub subtitles_text_color: String,
    pub subtitles_background_color: String,
    pub subtitles_outline_color: String,
    pub seek_time_duration: u8,
    pub streaming_server_warning_dismissed: Option<DateTime<Utc>>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            binge_watching: false,
            play_in_background: true,
            play_in_external_player: false,
            hardware_decoding: false,
            streaming_server_url: STREAMING_SERVER_URL.to_owned(),
            interface_language: "eng".to_owned(),
            subtitles_language: "eng".to_owned(),
            subtitles_size: 100,
            subtitles_font: "Roboto".to_owned(),
            subtitles_bold: false,
            subtitles_offset: 5,
            subtitles_text_color: "#FFFFFFFF".to_owned(),
            subtitles_background_color: "#00000000".to_owned(),
            subtitles_outline_color: "#00000000".to_owned(),
            seek_time_duration: 20,
            streaming_server_warning_dismissed: None,
        }
    }
}
