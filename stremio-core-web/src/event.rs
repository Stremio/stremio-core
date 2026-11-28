use serde::Deserialize;
use stremio_core::types::resource::Stream;

#[derive(Deserialize)]
#[serde(tag = "event", content = "args")]
pub enum UIEvent {
    #[serde(rename_all = "camelCase")]
    LocationPathChanged {
        prev_path: String,
    },
    #[serde(rename_all = "camelCase")]
    Search {
        query: String,
        responses_count: u32,
    },
    Share {
        url: String,
    },
    StreamClicked {
        stream: Box<Stream>,
    },
}
