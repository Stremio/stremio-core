use derivative::Derivative;
use serde::Serialize;

#[derive(Derivative, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    #[derivative(Default)]
    Loading,
    Ready(R),
    Err(E),
}

impl<R, E> Loadable<R, E> {
    #[inline]
    pub fn is_ready(&self) -> bool {
        match self {
            Loadable::Ready(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_err(&self) -> bool {
        match self {
            Loadable::Err(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_loading(&self) -> bool {
        match self {
            Loadable::Loading => true,
            _ => false,
        }
    }
}
