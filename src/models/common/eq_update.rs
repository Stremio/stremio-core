use crate::runtime::Effects;

#[inline]
pub fn eq_update<T: PartialEq>(value: &mut T, next_value: T) -> Effects {
    if *value != next_value {
        *value = next_value;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
