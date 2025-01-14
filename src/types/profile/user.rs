use core::fmt;

#[cfg(test)]
use chrono::offset::TimeZone;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Duration, Utc};
#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError, DefaultOnNull, DurationSeconds, NoneAsEmptyString};

use crate::{constants::NEW_USER_DAYS, runtime::Env};

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct TraktInfo {
    #[serde(with = "ts_seconds")]
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp_opt(0, 0).unwrap()")))]
    pub created_at: DateTime<Utc>,
    #[serde_as(as = "DurationSeconds<i64>")]
    #[cfg_attr(test, derivative(Default(value = "Duration::zero()")))]
    pub expires_in: Duration,
    #[cfg_attr(test, derivative(Default(value = r#"String::from("token")"#)))]
    pub access_token: String,
}
impl fmt::Debug for TraktInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraktInfo")
            .field("created_at", &self.created_at)
            .field("expires_in", &self.expires_in)
            .field("access_token", &"<SENSITIVE>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct GDPRConsent {
    pub tos: bool,
    pub privacy: bool,
    pub marketing: bool,
    pub from: Option<String>,
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp_opt(0, 0).unwrap()")))]
    pub last_modified: DateTime<Utc>,
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp_opt(0, 0).unwrap()")))]
    pub date_registered: DateTime<Utc>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub trakt: Option<TraktInfo>,
    #[serde(rename = "premium_expire")]
    pub premium_expire: Option<DateTime<Utc>>,
    #[serde(rename = "gdpr_consent")]
    pub gdpr_consent: GDPRConsent,
}

impl User {
    pub fn is_new_user<E: Env + 'static>(&self) -> bool {
        let now = E::now();
        now.signed_duration_since(self.date_registered) < NEW_USER_DAYS
    }
}
