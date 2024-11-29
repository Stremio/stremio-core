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

/// StreamItemState is to be used when user intentionally changes some values from the defaults,
/// so that they would be persisted and restored when returning to the same stream,
/// or some of them reapplied when moving to the next video/stream.
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamItemState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle_track: Option<SubtitleTrack>,
    /// In milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle_delay: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Subtitles size, platform dependent units
    pub subtitle_size: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Vertical offset of the subtitles, platform dependent units
    pub subtitle_offset: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_track: Option<AudioTrack>,
    /// In milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_delay: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playback_speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    /// Id of the subtitle track
    pub id: String,
    /// Flag indicating whether this is an embedded subtitles or an addon subtitle
    pub embedded: bool,
    /// Optional string indicating subtitle language.
    /// This value should be used when playing next stream with the same bingeGroup
    /// and user had selected an embedded subtitle previously, as next stream in the same bingeGroup
    /// usually will have the same embedded subtitle tracks, but might be the case that
    /// they are not in the same order, and just checking based on id might select an incorrect track.
    /// Thus when setting the embedded subtitle based on stream state,
    /// we should set it based on id and language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    /// Id of the audio track
    pub id: String,
    /// Optional string indicating audio language.
    /// This value should be used when playing next stream with the same bingeGroup
    /// and user had selected an audio track previously, as next stream in the same bingeGroup
    /// usually will have the same audio tracks, but might be the case that they are not
    /// in the same order, and just checking based on id might select an incorrect track.
    /// Thus when setting the audio track based on stream state,
    /// we should set it based on id and language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
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
            let is_source_match = self.stream.is_source_match(new_stream);
            let is_binge_match = self.stream.is_binge_match(new_stream);
            if is_source_match {
                return state;
            } else if is_binge_match {
                return StreamItemState {
                    subtitle_track: state.subtitle_track.filter(|track| track.embedded),
                    subtitle_delay: None,
                    audio_delay: None,
                    ..state
                };
            }
            StreamItemState {
                playback_speed: state.playback_speed,
                player_type: state.player_type,
                ..Default::default()
            }
        })
    }
}
