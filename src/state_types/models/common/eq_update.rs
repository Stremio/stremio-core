use crate::state_types::Effects;

pub fn eq_update<T: Clone + PartialEq>(value: &mut T, next_value: &T) -> Effects {
    if next_value.ne(value) {
        *value = next_value.to_owned();
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
