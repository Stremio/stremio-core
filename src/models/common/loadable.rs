use serde::Serialize;

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    Loading,
    Ready(R),
    Err(E),
}

impl<R, E> Default for Loadable<R, E> {
    fn default() -> Self {
        Self::Loading
    }
}

impl<R, E> Loadable<R, E> {
    #[inline]
    pub fn map<U, F>(self, f: F) -> Loadable<U, E>
    where
        F: FnOnce(R) -> U,
    {
        match self {
            Loadable::Loading => Loadable::Loading,
            Loadable::Ready(ready) => Loadable::Ready(f(ready)),
            Loadable::Err(err) => Loadable::Err(err),
        }
    }
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

    #[inline]
    pub fn map_ready<F, U>(self, f: F) -> Loadable<U, E>
    where
        F: FnOnce(R) -> U,
    {
        match self {
            Loadable::Err(e) => Loadable::Err(e),
            Loadable::Ready(r) => Loadable::Ready(f(r)),
            Loadable::Loading => Loadable::Loading,
        }
    }
}

impl<R, E> From<Result<R, E>> for Loadable<R, E> {
    fn from(result: Result<R, E>) -> Self {
        match result {
            Ok(x) => Loadable::Ready(x),
            Err(err) => Loadable::Err(err),
        }
    }
}
