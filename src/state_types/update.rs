use crate::state_types::Effects;

pub type Reducer<Value, Action> = fn(prev: &Value, action: Action) -> (Value, bool);

pub fn update<Value, Action>(
    value: &mut Value,
    action: Action,
    reducer: Reducer<Value, Action>,
    effects: Effects,
) -> (Effects) {
    let (next, changed) = reducer(value, action);
    if changed {
        *value = next;
        effects
    } else {
        Effects::none().unchanged()
    }
}
