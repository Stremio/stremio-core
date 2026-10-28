use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoScale {
    Contain,
    Cover,
    Fill,
}

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
    pub duration: Option<u64>,
}
