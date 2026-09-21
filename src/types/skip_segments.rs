use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipSegmentKind {
    Intro,
    Recap,
    Outro,
    Preview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkipSegmentSource {
    StremioNative,
    IntroDb,
    SkipDb,
    TheIntroDb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipSegmentMatch {
    ExactStream,
    ExactEpisode,
    DurationAdjusted,
    Estimated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SkipSegmentStreamSpecificity {
    Episode,
    Duration,
    StreamName,
    OpenSubtitlesHash,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkipSegmentCandidate {
    pub kind: SkipSegmentKind,
    pub start_ms: u64,
    pub end_ms: u64,
    pub source: SkipSegmentSource,
    pub source_match: SkipSegmentMatch,
    pub source_confidence: Option<f64>,
    pub adjusted: bool,
    pub evidence_count: u32,
    pub stream_specificity: SkipSegmentStreamSpecificity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedSkipSegment {
    pub kind: SkipSegmentKind,
    pub from_ms: u64,
    pub to_ms: u64,
    pub confidence: f64,
    pub provenance: Vec<SkipSegmentSource>,
    pub adjusted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkipSegmentContext {
    pub item_id: String,
    pub media_type: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub duration_ms: Option<u64>,
    pub open_subtitles_hash: Option<String>,
    pub stream_name_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkipSegmentCacheEntry {
    pub context: SkipSegmentContext,
    pub source: SkipSegmentSource,
    pub candidates: Vec<SkipSegmentCandidate>,
    pub cached_at: DateTime<Utc>,
}
