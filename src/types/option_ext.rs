pub trait OptionExt<T> {
    fn is_some_and(&self, f: impl FnOnce(&T) -> bool) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn is_some_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        matches!(self, Some(x) if f(x))
    }
}
