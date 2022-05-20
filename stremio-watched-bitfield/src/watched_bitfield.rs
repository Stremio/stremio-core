use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

use crate::bitfield8::BitField8;

#[derive(Debug)]
pub struct WatchedBitFieldError(String);
impl fmt::Display for WatchedBitFieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for WatchedBitFieldError {}

#[derive(Debug, Clone)]
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
    pub fn construct_and_resize(
        serialized: &str,
        video_ids: Vec<String>,
    ) -> Result<WatchedBitField, Box<dyn std::error::Error>> {
        // note: videoIds.length could only be >= from serialized lastLength
        // should we assert?
        // we might also wanna assert that the bitfield.length for the returned wb is the same sa videoIds.length

        // serialized is formed by {id}:{len}:{serializedBuf}, but since {id} might contain : we have to pop gradually and then keep the rest
        let mut components = serialized.split(':').collect::<Vec<&str>>();

        if components.len() < 3 {
            return Err(Box::new(WatchedBitFieldError(
                "Not enough components".to_string(),
            )));
        }
        let serialized_buf = components
            .pop()
            .ok_or("Cannot obtain the serialized data")?
            .to_string();

        let anchor_length = components
            .pop()
            .ok_or("Cannot obtain the length field")?
            .parse::<i32>()?;
        let anchor_video_id = components.join(":");

        // We can shift the bitmap in any direction, as long as we can find the anchor video
        if let Some(anchor_video_idx) = video_ids.iter().position(|s| *s == anchor_video_id) {
            let offset = anchor_length - anchor_video_idx as i32 - 1;
            let bitfield = BitField8::try_from((serialized_buf, Some(video_ids.len())))?;

            // in case of an previous empty array, this will be 0
            if offset != 0 {
                // Resize the buffer
                let mut resized_wbf = WatchedBitField {
                    bitfield: BitField8::new(video_ids.len()),
                    video_ids: video_ids.clone(),
                };
                // rewrite the old buf into the new one, applying the offset
                for i in 0..video_ids.len() {
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

    pub fn get(&self, idx: usize) -> bool {
        self.bitfield.get(idx)
    }

    pub fn get_video(&self, video_id: String) -> bool {
        if let Some(pos) = self.video_ids.iter().position(|s| *s == video_id) {
            self.bitfield.get(pos)
        } else {
            false
        }
    }

    pub fn set(&mut self, idx: usize, v: bool) {
        self.bitfield.set(idx, v);
    }

    pub fn set_video(&mut self, video_id: String, v: bool) {
        if let Some(pos) = self.video_ids.iter().position(|s| *s == video_id) {
            self.bitfield.set(pos, v);
        }
    }
}

impl fmt::Display for WatchedBitField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let packed = String::try_from(&self.bitfield).expect("bitfield failed to compress");
        let last_id = self.bitfield.last_index_of(true).unwrap_or(0);
        write!(f, "{}:{}:{}", self.video_ids[last_id], last_id + 1, packed)
    }
}
