use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// @TODO: u64 vs u32
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct LibItemState {
    pub last_watched: Option<DateTime<Utc>>,
    pub time_watched: u64,
    pub time_offset: u64,
    pub overall_time_watched: u64,
    pub times_watched: u32,
    // @TODO: consider bool that can be deserialized from an integer
    pub flagged_watched: u32,
    pub duration: u64,
    #[serde(rename = "video_id")]
    pub video_id: Option<String>,
    // @TODO bitfield, special type
    pub watched: Option<String>,
    pub no_notif: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct LibItem {
    #[serde(rename = "_id")]
    pub id: String,

    pub removed: bool,
    pub temp: bool,

    #[serde(rename = "_ctime")]
    pub ctime: DateTime<Utc>,
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,

    pub state: LibItemState,

    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub year: Option<String>,
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
        //dbg!(&l);
        // @TODO assertions
    }
}
