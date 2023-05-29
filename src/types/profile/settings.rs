use crate::constants::STREAMING_SERVER_URL;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub interface_language: String,
    pub streaming_server_url: Url,
    pub player_type: Option<String>,
    pub binge_watching: bool,
    pub play_in_background: bool,
    pub hardware_decoding: bool,
    pub frame_rate_matching_strategy: FrameRateMatchingStrategy,
    pub next_video_notification_duration: u32,
    pub audio_passthrough: bool,
    pub audio_language: String,
    pub secondary_audio_language: Option<String>,
    pub subtitles_language: String,
    pub secondary_subtitles_language: Option<String>,
    pub subtitles_size: u8,
    pub subtitles_font: String,
    pub subtitles_bold: bool,
    pub subtitles_offset: u8,
    pub subtitles_text_color: String,
    pub subtitles_background_color: String,
    pub subtitles_outline_color: String,
    pub seek_time_duration: u32,
    pub streaming_server_warning_dismissed: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameRateMatchingStrategy {
    Disabled,
    FrameRateOnly,
    FrameRateAndResolution,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            player_type: None,
            binge_watching: true,
            play_in_background: true,
            hardware_decoding: true,
            frame_rate_matching_strategy: FrameRateMatchingStrategy::FrameRateOnly,
            next_video_notification_duration: 35000,
            audio_passthrough: false,
            streaming_server_url: STREAMING_SERVER_URL.to_owned(),
            interface_language: "eng".to_owned(),
            audio_language: "eng".to_owned(),
            secondary_audio_language: None,
            subtitles_language: "eng".to_owned(),
            secondary_subtitles_language: None,
            subtitles_size: 100,
            subtitles_font: "Roboto".to_owned(),
            subtitles_bold: false,
            subtitles_offset: 5,
            subtitles_text_color: "#FFFFFFFF".to_owned(),
            subtitles_background_color: "#00000000".to_owned(),
            subtitles_outline_color: "#000000".to_owned(),
            seek_time_duration: 10000,
            streaming_server_warning_dismissed: None,
        }
    }
}
