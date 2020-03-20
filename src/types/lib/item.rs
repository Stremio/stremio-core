use crate::types::PosterShape;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};

// Reference: https://github.com/Stremio/stremio-api/blob/master/types/libraryItem.go

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct LibItemModified(
    pub String,
    #[serde(with = "ts_milliseconds")] pub DateTime<Utc>,
);

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct LibItemState {
    #[serde(deserialize_with = "empty_string_as_none")]
    pub last_watched: Option<DateTime<Utc>>,
    pub time_watched: u64,
    pub time_offset: u64,
    pub overall_time_watched: u64,
    pub times_watched: u32,
    // @TODO: consider bool that can be deserialized from an integer
    pub flagged_watched: u32,
    pub duration: u64,
    #[serde(rename = "video_id", deserialize_with = "empty_string_as_none")]
    pub video_id: Option<String>,
    // @TODO bitfield, special type
    #[serde(deserialize_with = "empty_string_as_none")]
    pub watched: Option<String>,
    // release date of last observed video
    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub last_vid_released: Option<DateTime<Utc>>,
    pub no_notif: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LibItem {
    #[serde(rename = "_id")]
    pub id: String,

    pub removed: bool,
    pub temp: bool,

    #[serde(rename = "_ctime", deserialize_with = "empty_string_as_none", default)]
    pub ctime: Option<DateTime<Utc>>,
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,

    pub state: LibItemState,

    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub poster: Option<String>,
    #[serde(default, skip_serializing_if = "PosterShape::is_unspecified")]
    pub poster_shape: PosterShape,
    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub year: Option<String>,
}

impl LibItem {
    pub fn should_persist(&self) -> bool {
        !self.temp
    }
    // Must return a result that's in a logical conjunction (&&) with .should_persist()
    pub fn should_push(&self) -> bool {
        self.should_persist()
            && self.type_name != "other"
            && if self.removed {
                self.state.overall_time_watched > 60_000
            } else {
                true
            }
    }
    pub fn is_in_continue_watching(&self) -> bool {
        let is_resumable = self.state.time_offset > 0;

        // having a Some for video_id and time_offset == 0 means it's set to this video as "next"
        let is_with_nextvid = self.state.time_watched == 0
            && self.state.video_id.is_some()
            && self.state.video_id.as_ref() != Some(&self.id);

        !self.removed && (is_resumable || is_with_nextvid)
    }
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_ref().map(String::as_str);
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_deser() {
        // `serialized` is copy-pasted from Stremio v4.4
        let serialized = r#"
        {"state":{"lastWatched":"2016-06-03T08:36:42.494Z","timeWatched":0,"timeOffset":0,"overallTimeWatched":0,"timesWatched":0,"flaggedWatched":0,"duration":0,"video_id":"","watched":"","noNotif":false,"season":0,"episode":0,"watchedEpisodes":[]},"_id":"tt0004972","removed":true,"temp":true,"_ctime":"2016-06-03T08:29:46.612Z","_mtime":"2016-06-03T08:36:43.991Z","name":"The Birth of a Nation","type":"movie","poster":"https://images.metahub.space/poster/medium/tt0004972/img","background":"","logo":"","year":"","imdb_id":"tt0004972"}
        "#;
        let l: LibItem = serde_json::from_str(&serialized).unwrap();
        assert_eq!(l.background, None, "background deserialized correctly");
        assert_eq!(
            l.poster,
            Some("https://images.metahub.space/poster/medium/tt0004972/img".to_owned()),
            "poster deserialized correctly"
        );
    }
}
