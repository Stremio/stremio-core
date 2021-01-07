use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub enum UIEvent {
    PlayerPaused,
}
