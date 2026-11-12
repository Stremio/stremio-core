use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DefaultOnError, VecSkipError};
use url::Url;

use crate::types::True;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RemoteConfigResponse {
    pub client: serde_json::Value,
    pub notifications: serde_json::Value,
    pub experiments: serde_json::Value,
}

#[serde_as]
#[derive(Default, Deserialize, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ClientSection {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub min_refresh_seconds: Option<u64>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub language_aliases: HashMap<String, String>,
}

#[serde_as]
#[derive(Default, Deserialize, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct NotificationsSection {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub cooldown_seconds: Option<u64>,
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    pub items: Vec<NotificationDef>,
}

#[serde_as]
#[derive(Default, Deserialize, Debug)]
#[serde(default)]
pub struct ExperimentsSection {
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    pub experiments: Vec<ExperimentDef>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationDef {
    pub v: u32,
    pub id: String,
    pub rev: u32,
    pub template: String,
    #[serde(default)]
    pub priority: i64,
    pub ends_at: DateTime<Utc>,
    #[serde(default)]
    pub targeting: Targeting,
    pub content: Content,
    #[serde(default)]
    pub buttons: Vec<ButtonDef>,
}

#[derive(Clone, Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Targeting {
    #[serde(default, deserialize_with = "present")]
    pub platforms: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present")]
    pub os: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present")]
    pub countries: Option<CountriesRule>,
    #[serde(default, deserialize_with = "present")]
    pub languages: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present")]
    pub app_version: Option<VersionRange>,
    #[serde(default, deserialize_with = "present")]
    pub shell_version: Option<VersionRange>,
    #[serde(default, deserialize_with = "present")]
    pub auth: Option<AuthRule>,
    #[serde(default, deserialize_with = "present")]
    pub supporter: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    pub registered: Option<DateRange>,
    #[serde(default, deserialize_with = "present")]
    pub marketing_consent: Option<True>,
    #[serde(default, deserialize_with = "present")]
    pub rollout_percent: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    pub experiment: Option<ExperimentRule>,
    #[serde(default, deserialize_with = "present")]
    pub accounts: Option<AccountsRule>,
    #[serde(default, deserialize_with = "present")]
    pub settings: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum CountriesRule {
    Include(Vec<String>),
    Exclude(Vec<String>),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VersionRange {
    #[serde(default, deserialize_with = "present")]
    pub min: Option<Version>,
    #[serde(default, deserialize_with = "present")]
    pub max: Option<Version>,
}

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AuthRule {
    LoggedIn,
    LoggedOut,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DateRange {
    #[serde(default, deserialize_with = "present")]
    pub before: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "present")]
    pub after: Option<DateTime<Utc>>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExperimentRule {
    pub id: String,
    pub variants: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AccountsRule {
    pub salt: String,
    pub hashes: Vec<String>,
}

#[serde_as]
#[derive(Clone, Default, Deserialize, Debug)]
#[serde(default)]
pub struct Content {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub image: Option<Url>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub locales: HashMap<String, LocaleCopy>,
}

#[serde_as]
#[derive(Clone, Default, Deserialize, Debug)]
#[serde(default)]
pub struct LocaleCopy {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub title: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub body: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub buttons: HashMap<String, String>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ButtonDef {
    pub id: String,
    pub style: String,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ActionDef {
    OpenUrl {
        url: Url,
    },
    Navigate {
        to: String,
    },
    SetSettings {
        patch: serde_json::Map<String, serde_json::Value>,
    },
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ExperimentUnit {
    #[default]
    Installation,
    Account,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentDef {
    pub v: u32,
    pub id: String,
    pub active: bool,
    pub salt: String,
    #[serde(default)]
    pub unit: ExperimentUnit,
    #[serde(default = "default_traffic_percent")]
    pub traffic_percent: f64,
    #[serde(default)]
    pub targeting: Targeting,
    pub variants: Vec<VariantDef>,
    #[serde(default)]
    pub events: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VariantDef {
    pub id: String,
    pub weight: u64,
    pub label: Option<String>,
    pub payload: Option<serde_json::Value>,
}

fn default_traffic_percent() -> f64 {
    100.0
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

pub struct ParsedConfig {
    pub client: ClientSection,
    pub notifications: NotificationsSection,
    pub experiments: ExperimentsSection,
}

impl From<&RemoteConfigResponse> for ParsedConfig {
    fn from(config: &RemoteConfigResponse) -> Self {
        Self {
            client: ClientSection::deserialize(&config.client).unwrap_or_default(),
            notifications: NotificationsSection::deserialize(&config.notifications)
                .unwrap_or_default(),
            experiments: ExperimentsSection::deserialize(&config.experiments).unwrap_or_default(),
        }
    }
}

pub(crate) fn is_safe_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || char == '_' || char == '-')
}

impl NotificationDef {
    pub fn is_valid(&self) -> bool {
        let mut button_ids = self
            .buttons
            .iter()
            .map(|button| &button.id)
            .collect::<Vec<_>>();
        button_ids.sort();
        button_ids.dedup();
        self.v <= 1
            && is_safe_id(&self.id)
            && self.rev >= 1
            && button_ids.len() == self.buttons.len()
            && self.buttons.iter().all(|button| {
                let host_actions = button
                    .actions
                    .iter()
                    .filter(|action| !matches!(action, ActionDef::SetSettings { .. }))
                    .count();
                button.actions.len() <= 3
                    && host_actions <= 1
                    && button.id.len() <= 24
                    && button
                        .id
                        .starts_with(|char: char| char.is_ascii_lowercase())
                    && button.id.chars().all(|char| {
                        char.is_ascii_lowercase() || char.is_ascii_digit() || char == '_'
                    })
            })
    }
}

impl ExperimentDef {
    pub fn is_valid(&self) -> bool {
        self.v <= 1
            && self.active
            && is_safe_id(&self.id)
            && !self.salt.is_empty()
            && (1..=8).contains(&self.variants.len())
            && self.variants.iter().all(|variant| variant.weight <= 10000)
            && self.targeting.experiment.is_none()
    }
}
