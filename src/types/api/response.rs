use std::collections::HashMap;

use chrono::{serde::ts_milliseconds, DateTime, Utc};
use derive_more::TryInto;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

use crate::types::{
    addon::Descriptor,
    library::LibraryItem,
    profile::{AuthKey, User},
    True,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum APIResult<T> {
    Err { error: APIError },
    Ok { result: T },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct APIError {
    pub message: String,
    pub code: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionResponse {
    pub addons: Vec<Descriptor>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "authKey")]
    pub key: AuthKey,
    pub user: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataExportResponse {
    pub export_id: String,
}

#[derive(PartialEq, Eq, Deserialize, Debug)]
pub struct LibraryItemModified(
    pub String,
    #[serde(with = "ts_milliseconds")] pub DateTime<Utc>,
);

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: True,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LinkCodeResponse {
    pub code: String,
    pub link: String,
    pub qrcode: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinkAuthKey {
    pub auth_key: String,
}

#[derive(Clone, TryInto, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum LinkDataResponse {
    AuthKey(LinkAuthKey),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModalAddon {
    pub name: String,
    pub manifest_url: Url,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetModalResponse {
    pub id: String,
    pub title: String,
    pub message: String,
    pub image_url: Url,
    pub addon: Option<ModalAddon>,
    pub external_url: Option<Url>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNotificationResponse {
    pub id: String,
    pub title: String,
    pub message: String,
    pub external_url: Option<Url>,
}

/// API response for the [`LibraryItem`]s which skips invalid items
/// when deserializing.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde_as]
#[serde(transparent)]
pub struct LibraryItemsResponse(#[serde_as(as = "VecSkipError<_>")] pub Vec<LibraryItem>);

impl LibraryItemsResponse {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkipGapsResponse {
    /// Returns the matched attribute: Opensubtitles Hash, File name or season/episode
    ///
    /// Primarily used for debugging
    pub accuracy: String,
    /// The key of the map is the duration of video
    /// and the value is the skip gaps - skip history and outro
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub gaps: HashMap<u64, SkipGaps>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkipGaps {
    #[serde(default)]
    pub seek_history: Vec<SeekEvent>,
    #[serde(default)]
    pub outro: Option<u64>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SeekEvent {
    pub records: u64,
    #[serde(rename = "seekFrom")]
    pub from: u64,
    #[serde(rename = "seekTo")]
    pub to: u64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_skip_gaps_response() {
        {
            let skip_outro_response = serde_json::json!({
                "result": {
                    "accuracy": "",
                    "gaps": {},
                },
                "error":null
            });

            let response =
                serde_json::from_value::<APIResult<SkipGapsResponse>>(skip_outro_response)
                    .expect("Should deserialize empty response");

            match response {
                APIResult::Ok { result } => {
                    assert_eq!(
                        result,
                        SkipGapsResponse {
                            accuracy: "".into(),
                            gaps: HashMap::default(),
                        }
                    )
                }
                APIResult::Err { error } => panic!("Expected success and not an error: {error:?}"),
            }
        }

        // Gaps returned - intro and outro gaps
        {
            let skip_outro_response = serde_json::json!({
                "result": {
                    "accuracy": "byEpisode",
                    "gaps": {
                        "1295200": {
                            "seekHistory": [],
                            "outro":null
                        },
                        "1295189": {
                            "seekHistory": [],
                            "outro": 1167627
                        },
                        "1340131": {
                            "seekHistory": [
                                {"records": 71, "seekFrom": 103856, "seekTo": 147857}
                            ],
                            "outro": 1218668
                        },
                        "1340171": {
                            "seekHistory": [
                                {"records": 15, "seekFrom": 104865, "seekTo": 147452}
                            ],
                            "outro": 1238020
                        }
                    }
                },
                "error":null
            });

            let response =
                serde_json::from_value::<APIResult<SkipGapsResponse>>(skip_outro_response)
                    .expect("Should deserialize response");
            match response {
                APIResult::Ok { result } => {
                    assert_eq!("byEpisode", result.accuracy,);
                    assert_eq!(4, result.gaps.len());
                }
                APIResult::Err { error } => panic!("Expected success and not an error: {error:?}"),
            }
        }
    }
}
