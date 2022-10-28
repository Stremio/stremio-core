pub trait OptionInspectExt<T> {
    fn inspect_some<F: FnOnce(&T)>(self, f: F) -> Self;
}

impl<T> OptionInspectExt<T> for Option<T> {
    #[inline]
    fn inspect_some<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Some(ref x) = self {
            f(x);
        }

        self
    }
}
