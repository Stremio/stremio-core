#[cfg(test)]
use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull, NoneAsEmptyString};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
pub struct GDPRConsent {
    pub tos: bool,
    pub privacy: bool,
    pub marketing: bool,
    pub from: Option<String>,
}

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull<NoneAsEmptyString>")]
    pub fb_id: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull<NoneAsEmptyString>")]
    pub avatar: Option<String>,
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp(0, 0)")))]
    pub last_modified: DateTime<Utc>,
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp(0, 0)")))]
    pub date_registered: DateTime<Utc>,
    #[serde(rename = "premium_expire")]
    pub premium_expire: Option<DateTime<Utc>>,
    #[serde(rename = "gdpr_consent")]
    pub gdpr_consent: GDPRConsent,
}
