use serde::de::Deserializer;
use serde::Deserialize;

pub fn deserialize_single_as_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SingleOrMultiple<T> {
        Single(T),
        Multiple(Vec<T>),
    }
    Ok(match SingleOrMultiple::deserialize(deserializer)? {
        SingleOrMultiple::Single(s) => vec![s],
        SingleOrMultiple::Multiple(m) => m,
    })
}
