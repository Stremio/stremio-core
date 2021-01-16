use serde::Deserialize;
use stremio_core::runtime::msg::{Action, Event};

#[derive(Deserialize)]
pub enum UIEvent {
    PlayerPaused,
}

pub enum WebEvent {
    CoreAction(Action),
    CoreEvent(Event),
    UIEvent(UIEvent),
}
