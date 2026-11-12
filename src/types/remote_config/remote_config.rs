use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use url::Url;

#[derive(Clone, Default, PartialEq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    pub notifications: Vec<RemoteNotification>,
    pub experiments: BTreeMap<String, ExperimentAssignment>,
    #[serde(skip)]
    pub last_attempt_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub applied_started_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub applied_geo_started_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub flushing_since: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteNotification {
    pub id: String,
    pub rev: u32,
    pub template: String,
    pub title: String,
    pub body: Option<String>,
    pub image: Option<Url>,
    pub buttons: Vec<RemoteNotificationButton>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteNotificationButton {
    pub id: String,
    pub label: String,
    pub style: String,
    pub open_url: Option<Url>,
    pub navigate: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentAssignment {
    pub variant: String,
    pub payload: Option<serde_json::Value>,
    #[serde(skip)]
    pub salt: String,
    #[serde(skip)]
    pub unit_key: String,
}
