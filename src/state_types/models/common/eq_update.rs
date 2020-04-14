use crate::state_types::Effects;

pub fn eq_update<T: PartialEq>(value: &mut T, next_value: T) -> Effects {
    if *value != next_value {
        *value = next_value;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
