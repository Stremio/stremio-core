use serde::Deserialize;
use stremio_core::runtime::msg::Event;

#[derive(Deserialize)]
pub enum UIEvent {
    FlushAnalytics,
    PlayerPaused,
}

pub enum WebEvent {
    CoreEvent(Event),
    UIEvent(UIEvent),
}
