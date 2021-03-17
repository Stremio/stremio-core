use crate::types::resource::Stream;
use serde::de::Deserializer;
use serde::Deserialize;

pub fn deserialize_video_streams<'de, D>(deserializer: D) -> Result<Vec<Stream>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StreamOrStreams<Stream> {
        Single(Stream),
        Multiple(Vec<Stream>),
    }
    Ok(match StreamOrStreams::deserialize(deserializer)? {
        StreamOrStreams::Single(stream) => vec![stream],
        StreamOrStreams::Multiple(streams) => streams,
    })
}
