use serde::{Deserialize, Serialize};
use url::Url;
use url_serde;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub binge_watching: bool,
    pub play_in_background: bool,
    pub play_in_external_player: bool,
    #[serde(with = "url_serde")]
    pub streaming_server_url: Url,
    pub interface_language: String,
    pub subtitles_language: String,
    pub subtitles_size: u8,
    pub subtitles_text_color: String,
    pub subtitles_background_color: String,
    pub subtitles_outline_color: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            binge_watching: false,
            play_in_background: true,
            play_in_external_player: false,
            streaming_server_url: Url::parse("http://127.0.0.1:11470").unwrap(),
            interface_language: "eng".to_owned(),
            subtitles_language: "eng".to_owned(),
            subtitles_size: 2,
            subtitles_text_color: "#FFFFFF00".to_owned(),
            subtitles_background_color: "#00000000".to_owned(),
            subtitles_outline_color: "#00000000".to_owned(),
        }
    }
}
