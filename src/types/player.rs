use serde::Serialize;

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IntroOutro {
    pub intro: Option<IntroData>,
    pub outro: Option<u64>,
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
