use crate::error::Error;
use base64::{decode, encode};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::{
    fmt::{Display, Formatter},
    io::{Read, Write},
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitField8 {
    pub length: usize,
    pub(crate) values: Vec<u8>,
}

impl BitField8 {
    pub fn new(length: usize) -> BitField8 {
        let length = (length as f64 / 8.0).ceil() as usize;
        BitField8 {
            length,
            values: vec![0; length],
        }
    }

    pub fn get(&self, i: usize) -> bool {
        let index = i / 8;
        let bit = i % 8;

        if index >= self.values.len() {
            false
        } else {
            (self.values[index] >> bit) & 1 != 0
        }
    }

    /// Creates a new [`BitField8`] using the passed values and an optional
    /// length for the struct.
    ///
    /// If length is `None` a default value of `values.len() * 8` will be used.
    pub fn new_with_values(mut values: Vec<u8>, length: Option<usize>) -> Self {
        let length = length.unwrap_or(values.len() * 8);
        let bytes = (length as f64 / 8.0).ceil() as usize;
        if bytes > values.len() {
            values.extend(vec![0; bytes - values.len()]);
        }

        Self { length, values }
    }

    pub fn set(&mut self, i: usize, val: bool) {
        let index = i / 8;
        let mask = 1 << (i % 8);

        if index >= self.values.len() {
            self.values.extend(vec![0; index - self.values.len() + 1]);
            self.length = self.values.len() * 8;
        }

        if val {
            self.values[index] |= mask;
        } else {
            self.values[index] &= !mask;
        }
    }

    /// get the last index where the value is `true` or `false` (`val`)
    pub fn last_index_of(&self, val: bool) -> Option<usize> {
        (0..self.length).rev().find(|&i| self.get(i) == val)
    }
}

impl TryFrom<(String, Option<usize>)> for BitField8 {
    type Error = Error;
    fn try_from((encoded, length): (String, Option<usize>)) -> Result<Self, Self::Error> {
        let compressed = decode(encoded)?;
        let mut values = vec![];
        let mut decoded = ZlibDecoder::new(&compressed[..]);
        decoded.read_to_end(&mut values)?;

        Ok(Self::new_with_values(values, length))
    }
}

impl TryFrom<&BitField8> for String {
    type Error = std::io::Error;
    fn try_from(bit_field: &BitField8) -> Result<Self, Self::Error> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
        encoder.write_all(&bit_field.values)?;
        Ok(encode(encoder.finish()?))
    }
}

impl Display for BitField8 {
    /// Discards the `std::io::Error` which can be triggered by the [`String::try_from`],
    /// because `std::fmt::Error` does not have any fields
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = String::try_from(self).map_err(|_io_err| std::fmt::Error)?;

        f.write_str(&string)
    }
}

impl FromStr for BitField8 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from((s.to_string(), None))
    }
}

/// Module containing all the impls of the `serde` feature
#[cfg(feature = "serde")]
mod serde {
    use serde::Serialize;

    use super::BitField8;

    impl<'de> serde::Deserialize<'de> for BitField8 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let string = String::deserialize(deserializer)?;
            let length = None;

            BitField8::try_from((string, length)).map_err(serde::de::Error::custom)
        }
    }

    impl Serialize for BitField8 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let string = String::try_from(self).map_err(serde::ser::Error::custom)?;

            serializer.serialize_str(&string)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bitfield8::BitField8;

    #[test]
    #[cfg(feature = "serde")]
    fn test_de_serialize() {
        let json_value = serde_json::json!("eJyTZwAAAEAAIA==");

        {
            let expected = BitField8 {
                length: 16,
                values: vec![31, 0],
            };

            let actual_from_json = serde_json::from_value::<BitField8>(json_value.clone())
                .expect("Should deserialize ");
            assert_eq!(expected, actual_from_json);

            let actual_to_json = serde_json::to_value(&expected).expect("Should serialize");
            assert_eq!(json_value, actual_to_json);
        }

        // with custom length
        // should result in the same string (serialized)
        {
            let expected = BitField8 {
                // Different length!
                length: 9,
                values: vec![31, 0],
            };

            let actual_from_json = serde_json::from_value::<BitField8>(json_value.clone())
                .expect("Should deserialize ");
            // The fact that we have custom length is the reason these two values will not be the same
            assert_ne!(expected.length, actual_from_json.length);
            assert_eq!(16, actual_from_json.length);
            assert_eq!(expected.values, actual_from_json.values);

            let actual_to_json = serde_json::to_value(&expected).expect("Should serialize");
            assert_eq!(json_value, actual_to_json);
        }
    }

    #[test]
    fn parse_length() {
        let watched = "eJyTZwAAAEAAIA==".to_string();
        let bf = BitField8::try_from((watched.clone(), Some(9))).unwrap();
        assert_eq!(bf.length, 9);

        // If the value is not provided the length is rounded towards the next byte
        let bf = BitField8::try_from((watched, None)).unwrap();
        assert_eq!(bf.length, 16);
    }
}
