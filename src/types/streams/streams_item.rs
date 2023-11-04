use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

use crate::types::resource::Stream;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamsItem {
    pub stream: Stream,
    pub r#type: String,
    pub meta_id: String,
    pub video_id: String,
    pub meta_transport_url: Url,
    pub stream_transport_url: Url,
    pub state: Option<StreamItemState>,
    /// Modification time
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamItemState {
    pub subtitle_track_id: Option<String>,
    pub subtitle_language: Option<String>,
    pub subtitle_delay_ms: Option<i64>,
    pub audio_track_id: Option<String>,
    pub audio_language: Option<String>,
    pub audio_delay_ms: Option<i64>,
    pub playback_speed: Option<f32>,
    pub player_type: Option<String>,
}

impl StreamsItem {
    /// Retrieve adjusted stream state based on given stream:
    ///     If stream source is the same we want to retain the same state;
    ///     If stream binge group matches we want to retain same state except
    ///         for subtitle and audio delay, as these are not relevant when playing
    ///         a stream for next binge video as audio/subtitle sync might be different,
    ///         but we want to retain track ids as next binge group might have
    ///         the same embedded tracks with same ids;
    ///     Otherwise retain only playback speed and player type as these should not change
    ///         regardless of the video, since you usually want to maintain these throughout
    ///         the whole series;
    #[inline]
    pub fn adjusted_state(&self, new_stream: &Stream) -> Option<StreamItemState> {
        self.state.clone().map(|state| {
            let is_exact_match = self.stream.source == new_stream.source;
            let is_binge_match = self.stream.is_binge_match(new_stream);
            if is_exact_match {
                return state;
            } else if is_binge_match {
                return StreamItemState {
                    subtitle_delay_ms: None,
                    audio_delay_ms: None,
                    ..state
                };
            }
            StreamItemState {
                subtitle_track_id: None,
                subtitle_language: None,
                subtitle_delay_ms: None,
                audio_track_id: None,
                audio_language: None,
                audio_delay_ms: None,
                ..state
            }
        })
    }
}
