use crate::{BitField8, Error};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// (De)Serializable field that tracks which videos have been watched
/// and the latest one watched.
///
/// This is a [`WatchedBitField`] compatible field, (de)serialized
/// without the knowledge of `videos_ids`.
///
/// `{anchor:video_id}:{anchor_length}:{bitfield8}`
///
/// # Examples
///
/// ```
/// use stremio_watched_bitfield::WatchedField;
///
/// // `tt2934286:1:5` - anchor video id
/// // `5` - anchor video length
/// // `eJyTZwAAAEAAIA==` - BitField8
///
/// let watched = "tt2934286:1:5:5:eJyTZwAAAEAAIA=="
///     .parse::<WatchedField>()
///     .expect("Should parse");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchedField {
    /// The anchor video id
    ///
    /// Indicates which is the last watched video id.
    anchor_video: String,
    /// The length from the beginning of the `BitField8` to the last
    /// watched video.
    anchor_length: usize,
    bitfield: BitField8,
}

impl Display for WatchedField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.anchor_video, self.anchor_length, self.bitfield
        )
    }
}

impl From<WatchedBitField> for WatchedField {
    fn from(watched_bit_field: WatchedBitField) -> Self {
        let last_id = watched_bit_field.bitfield.last_index_of(true).unwrap_or(0);
        let last_video_id = watched_bit_field
            .video_ids
            .get(last_id)
            .map_or_else(|| "undefined".to_string(), |id| id.clone());

        Self {
            anchor_video: last_video_id,
            anchor_length: last_id + 1,
            bitfield: watched_bit_field.bitfield,
        }
    }
}

impl FromStr for WatchedField {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // serialized is formed by {id}:{len}:{serializedBuf}, but since {id} might contain : we have to pop gradually and then keep the rest
        let mut components = string.split(':').collect::<Vec<&str>>();

        if components.len() < 3 {
            return Err(Error("Not enough components".to_string()));
        }
        let bitfield_buf = components
            .pop()
            .ok_or("Cannot obtain the serialized data")?
            .to_string();

        let anchor_length = components
            .pop()
            .ok_or("Cannot obtain the length field")?
            .parse::<usize>()?;
        let anchor_video_id = components.join(":");

        let bitfield = BitField8::try_from((bitfield_buf, None))?;

        Ok(Self {
            bitfield,
            anchor_video: anchor_video_id,
            anchor_length,
        })
    }
}

/// Tracks which videos have been watched.
///
/// Serialized in the format `{id}:{len}:{serializedBuf}` but since `{id}`
/// might contain `:` we pop gradually and then keep the rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchedBitField {
    bitfield: BitField8,
    video_ids: Vec<String>,
}

impl WatchedBitField {
    pub fn construct_from_array(arr: Vec<bool>, video_ids: Vec<String>) -> WatchedBitField {
        let mut bitfield = BitField8::new(video_ids.len());
        for (i, val) in arr.iter().enumerate() {
            bitfield.set(i, *val);
        }

        WatchedBitField {
            bitfield,
            video_ids,
        }
    }

    pub fn new(bitfield: BitField8, video_ids: Vec<String>) -> WatchedBitField {
        Self {
            bitfield,
            video_ids,
        }
    }

    pub fn construct_with_videos(
        watched_field: WatchedField,
        video_ids: Vec<String>,
    ) -> Result<WatchedBitField, Error> {
        // We can shift the bitmap in any direction, as long as we can find the anchor video
        if let Some(anchor_video_idx) = video_ids
            .iter()
            .position(|s| s == &watched_field.anchor_video)
        {
            // TODO: replace with `usize` and `checked_sub` when more tests are added for negative ids
            let offset = watched_field.anchor_length as i32 - anchor_video_idx as i32 - 1;
            let bitfield =
                BitField8::new_with_values(watched_field.bitfield.values, Some(video_ids.len()));

            // in case of an previous empty array, this will be 0
            if offset != 0 {
                // Resize the buffer
                let mut resized_wbf = WatchedBitField {
                    bitfield: BitField8::new(video_ids.len()),
                    video_ids: video_ids.clone(),
                };

                // rewrite the old buf into the new one, applying the offset
                for i in 0..video_ids.len() {
                    // TODO: Check what will happen if we change it to `usize`
                    let id_in_prev = i as i32 + offset;
                    if id_in_prev >= 0 && (id_in_prev as usize) < bitfield.length {
                        resized_wbf.set(i, bitfield.get(id_in_prev as usize));
                    }
                }
                Ok(resized_wbf)
            } else {
                Ok(WatchedBitField {
                    bitfield,
                    video_ids,
                })
            }
        } else {
            // videoId could not be found, return a totally blank buf
            Ok(WatchedBitField {
                bitfield: BitField8::new(video_ids.len()),
                video_ids,
            })
        }
    }

    pub fn construct_and_resize(
        serialized: &str,
        video_ids: Vec<String>,
    ) -> Result<WatchedBitField, Error> {
        // note: videoIds.length could only be >= from serialized lastLength
        // should we assert?
        // we might also wanna assert that the bitfield.length for the returned wb is the same sa videoIds.length

        let watched_field = serialized.parse()?;

        Self::construct_with_videos(watched_field, video_ids)
    }

    pub fn get(&self, idx: usize) -> bool {
        self.bitfield.get(idx)
    }

    pub fn get_video(&self, video_id: &str) -> bool {
        if let Some(pos) = self.video_ids.iter().position(|s| *s == video_id) {
            self.bitfield.get(pos)
        } else {
            false
        }
    }
    pub fn all_watched(&self) -> bool {
        // self.video_ids.iter()
        todo!()
    }

    pub fn set(&mut self, idx: usize, v: bool) {
        self.bitfield.set(idx, v);
    }

    pub fn set_video(&mut self, video_id: &str, v: bool) {
        if let Some(pos) = self.video_ids.iter().position(|s| *s == video_id) {
            self.bitfield.set(pos, v);
        }
    }
}

impl fmt::Display for WatchedBitField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let packed = String::try_from(&self.bitfield).expect("bitfield failed to compress");
        let last_id = self.bitfield.last_index_of(true).unwrap_or(0);
        let last_video_id = self
            .video_ids
            .get(last_id)
            .map_or("undefined", |id| id.as_str());

        write!(f, "{}:{}:{}", last_video_id, last_id + 1, packed)
    }
}

impl From<WatchedBitField> for BitField8 {
    fn from(watched: WatchedBitField) -> Self {
        watched.bitfield
    }
}

/// Module containing all the impls of the `serde` feature
#[cfg(feature = "serde")]
mod serde {
    use std::str::FromStr;

    use serde::{de, Serialize};

    use super::WatchedField;

    impl<'de> serde::Deserialize<'de> for WatchedField {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let serialized = String::deserialize(deserializer)?;

            WatchedField::from_str(&serialized).map_err(de::Error::custom)
        }
    }

    impl Serialize for WatchedField {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BitField8, WatchedBitField, WatchedField};

    #[test]
    fn parse_and_modify() {
        let videos = [
            "tt2934286:1:1",
            "tt2934286:1:2",
            "tt2934286:1:3",
            "tt2934286:1:4",
            "tt2934286:1:5",
            "tt2934286:1:6",
            "tt2934286:1:7",
            "tt2934286:1:8",
            "tt2934286:1:9",
        ];
        let watched = "tt2934286:1:5:5:eJyTZwAAAEAAIA==";
        let mut wb = WatchedBitField::construct_and_resize(
            watched,
            videos.iter().map(|v| v.to_string()).collect(),
        )
        .unwrap();

        assert!(wb.get_video("tt2934286:1:5"));
        assert!(!wb.get_video("tt2934286:1:6"));

        assert_eq!(watched, wb.to_string());

        wb.set_video("tt2934286:1:6", true);
        assert!(wb.get_video("tt2934286:1:6"));
    }

    #[test]
    fn construct_from_array() {
        let arr = vec![false; 500];
        let mut video_ids = vec![];
        for i in 1..500 {
            video_ids.push(format!("tt2934286:1:{}", i));
        }
        let mut wb = WatchedBitField::construct_from_array(arr, video_ids.clone());

        // All should be false
        for (i, val) in video_ids.iter().enumerate() {
            assert!(!wb.get(i));
            assert!(!wb.get_video(val));
        }

        // Set half to true
        for (i, _val) in video_ids.iter().enumerate() {
            wb.set(i, i % 2 == 0);
        }

        // Serialize and deserialize to new structure
        let watched = wb.to_string();
        let wb2 = WatchedBitField::construct_and_resize(
            &watched,
            video_ids.iter().map(|v| v.to_string()).collect(),
        )
        .unwrap();

        // Half should still be true
        for (i, val) in video_ids.iter().enumerate() {
            assert_eq!(wb2.get(i), i % 2 == 0);
            assert_eq!(wb2.get_video(val), i % 2 == 0);
        }
    }

    #[test]
    fn to_string_empty() {
        let watched = WatchedBitField::construct_from_array(vec![], vec![]);
        let serialized = watched.to_string();
        assert_eq!(serialized, "undefined:1:eJwDAAAAAAE=");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_watched_field_de_serialize() {
        let string = "tt7767422:3:8:24:eJz7//8/AAX9Av4=";
        let json_value = serde_json::json!(string);

        let expected = string.parse::<WatchedField>().expect("Should parse field");

        let actual_from_json = serde_json::from_value::<WatchedField>(json_value.clone())
            .expect("Should deserialize ");
        assert_eq!(expected, actual_from_json);
        assert_eq!("eJz7//8/AAX9Av4=", &actual_from_json.bitfield.to_string());
        assert_eq!(24, actual_from_json.anchor_length);
        assert_eq!("tt7767422:3:8", actual_from_json.anchor_video);

        let actual_to_json = serde_json::to_value(&expected).expect("Should serialize");
        assert_eq!(json_value, actual_to_json);
    }

    #[test]
    fn deserialize_empty() {
        let watched = WatchedBitField::construct_and_resize("undefined:1:eJwDAAAAAAE=", vec![]);
        assert_eq!(
            watched,
            Ok(WatchedBitField {
                bitfield: BitField8::new(0),
                video_ids: vec![]
            })
        );
    }
}
