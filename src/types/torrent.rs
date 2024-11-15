use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use stremio_serde_hex::{SerHex, Strict};

///
/// # Examples
/// ```
/// use stremio_core::types::torrent::InfoHash;
///
/// let info_hash = "df389295484b3059a4726dc6d8a57f71bb5f4c81"
///     .parse::<InfoHash>()
///     .unwrap();
///
/// dbg!(info_hash);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InfoHash(#[serde(with = "SerHex::<Strict>")] [u8; 20]);

impl InfoHash {
    pub fn new(info_hash: [u8; 20]) -> Self {
        Self(info_hash)
    }

    pub fn as_array(&self) -> [u8; 20] {
        self.0
    }
}

impl FromStr for InfoHash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut array = [0_u8; 20];
        hex::decode_to_slice(s, &mut array)?;

        Ok(Self(array))
    }
}

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
