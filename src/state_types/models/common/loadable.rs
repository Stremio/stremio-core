use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    Loading,
    Ready(R),
    Err(E),
}

impl<R, E> Default for Loadable<R, E> {
    fn default() -> Self {
        Loadable::Loading
    }
}
