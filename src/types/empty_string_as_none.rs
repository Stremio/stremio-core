use serde::de::{Deserializer, IntoDeserializer};
use serde::Deserialize;

pub fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    match Option::<String>::deserialize(deserializer) {
        Ok(Some(value)) if value.is_empty() => Ok(None),
        Ok(Some(value)) => T::deserialize(value.into_deserializer()).map(Some),
        Ok(None) => Ok(None),
        Err(error) => Err(error),
    }
}
