use serde::Serialize;

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IntroOutro {
    pub intro: Option<IntroData>,
    pub outro: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segments: Vec<IntroSegment>,
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IntroData {
    pub from: u64,
    pub to: u64,
    /// `Some` if the difference between the skip gap data
    /// and stream duration ([`LibraryItem.state.duration`]) > 0!
    pub duration: Option<u64>,
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IntroSegment {
    pub segment: String,
    pub from: u64,
    pub to: u64,
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct IntroDbRequest {
    pub imdb_id: String,
    pub season: u32,
    pub episode: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntroDbResponse {
    pub segments: Vec<IntroSegment>,
    pub outro: Option<SegmentRange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentRange {
    pub from: u64,
    pub to: u64,
}
