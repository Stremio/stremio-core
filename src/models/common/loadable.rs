use serde::Serialize;

#[derive(Clone, PartialEq, Serialize)]
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
