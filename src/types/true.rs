use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct True;

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<True, D::Error>
    where
        D: Deserializer<'de>,
    {
        match bool::deserialize(deserializer) {
            Ok(value) if value => Ok(True),
            Ok(value) => Err(serde::de::Error::invalid_value(
                Unexpected::Bool(value),
                &"true",
            )),
            Err(error) => Err(error),
        }
    }
}

impl Serialize for True {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(true)
    }
}
