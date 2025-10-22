use crate::constants::STREAMING_SERVER_URL;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub interface_language: String,
    pub hide_spoilers: bool,
    pub gamepad_support: bool,
    pub streaming_server_url: Url,
    pub player_type: Option<String>,
    pub binge_watching: bool,
    pub play_in_background: bool,
    pub hardware_decoding: bool,
    pub video_mode: Option<String>,
    pub frame_rate_matching_strategy: FrameRateMatchingStrategy,
    pub next_video_notification_duration: u32,
    pub audio_passthrough: bool,
    pub audio_language: Option<String>,
    pub secondary_audio_language: Option<String>,
    pub subtitles_language: Option<String>,
    pub secondary_subtitles_language: Option<String>,
    pub subtitles_size: u8,
    pub subtitles_font: String,
    pub subtitles_bold: bool,
    pub subtitles_offset: u8,
    pub subtitles_text_color: String,
    pub subtitles_background_color: String,
    pub subtitles_outline_color: String,
    pub subtitles_opacity: u8,
    /// Whether or not the Escape key should exists from the app when in Full screen.
    pub esc_exit_fullscreen: bool,
    /// The Seek time duration (in milliseconds) is when using the Arrow keys
    pub seek_time_duration: u32,
    /// The Seek short time duration (in milliseconds) is when we are finer seeking,
    /// e.g. using the Arrow keys + Shift
    pub seek_short_time_duration: u32,
    /// Whether we should pause the playback when the application get's minimized
    pub pause_on_minimize: bool,
    /// Wheter we should quit the application when closing the window
    pub quit_on_close: bool,
    pub surround_sound: bool,
    pub streaming_server_warning_dismissed: Option<DateTime<Utc>>,
    pub server_in_foreground: bool,
    pub send_crash_reports: bool,
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
            video_mode: None,
            hide_spoilers: false,
            gamepad_support: false,
            frame_rate_matching_strategy: FrameRateMatchingStrategy::FrameRateOnly,
            next_video_notification_duration: 35000,
            audio_passthrough: false,
            streaming_server_url: STREAMING_SERVER_URL.to_owned(),
            interface_language: "eng".to_owned(),
            audio_language: Some("eng".to_owned()),
            secondary_audio_language: None,
            subtitles_language: Some("eng".to_owned()),
            secondary_subtitles_language: None,
            subtitles_size: 100,
            subtitles_font: "Roboto".to_owned(),
            subtitles_bold: false,
            subtitles_offset: 5,
            subtitles_text_color: "#FFFFFFFF".to_owned(),
            subtitles_background_color: "#00000000".to_owned(),
            subtitles_outline_color: "#000000".to_owned(),
            subtitles_opacity: 100,
            esc_exit_fullscreen: true,
            seek_time_duration: 10000,
            seek_short_time_duration: 3000,
            pause_on_minimize: false,
            quit_on_close: true,
            surround_sound: false,
            streaming_server_warning_dismissed: None,
            server_in_foreground: false,
            send_crash_reports: true,
        }
    }
}
