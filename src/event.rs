use serde::Deserialize;
use stremio_core::runtime::msg::{Action, Event};

#[derive(Deserialize)]
#[serde(tag = "event", content = "args")]
pub enum UIEvent {
    PlayerPaused,
    #[serde(rename_all = "camelCase")]
    LocationPathChanged {
        prev_path: String,
    },
    #[serde(rename_all = "camelCase")]
    Search {
        query: String,
        responses_count: u32,
    },
}

pub enum WebEvent {
    CoreAction(Action),
    CoreEvent(Event),
    UIEvent(UIEvent),
}
