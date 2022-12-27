use derivative::Derivative;
use serde::Serialize;

#[derive(Derivative, Clone, PartialEq, Eq, Serialize, Debug)]
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
        matches!(self, Loadable::Ready(_))
    }
    #[inline]
    pub fn is_err(&self) -> bool {
        matches!(self, Loadable::Err(_))
    }
    #[inline]
    pub fn is_loading(&self) -> bool {
        matches!(self, Loadable::Loading)
    }
    #[inline]
    pub fn as_ref(&self) -> Loadable<&R, &E> {
        match *self {
            Loadable::Err(ref e) => Loadable::Err(e),
            Loadable::Ready(ref r) => Loadable::Ready(r),
            Loadable::Loading => Loadable::Loading,
        }
    }
    #[inline]
    pub fn ready(&self) -> Option<&R> {
        match self {
            Loadable::Ready(r) => Some(r),
            _ => None,
        }
    }
    #[inline]
    pub fn err(&self) -> Option<&E> {
        match self {
            Loadable::Err(e) => Some(e),
            _ => None,
        }
    }
    #[inline]
    pub fn expect(self, msg: &str) -> R {
        match self {
            Self::Ready(r) => r,
            _ => panic!("{}", msg),
        }
    }
    #[inline]
    pub fn expect_err(self, msg: &str) -> E {
        match self {
            Self::Err(e) => e,
            _ => panic!("{}", msg),
        }
    }
    #[inline]
    pub fn expect_loading(self, msg: &str) {
        match self {
            Self::Loading => {}
            _ => panic!("{}", msg),
        }
    }
}
