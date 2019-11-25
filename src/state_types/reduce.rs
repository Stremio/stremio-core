use crate::state_types::Effects;

pub type Reducer<State, Value, Action> = fn(prev: &State, action: Action) -> (Value, bool);

pub fn reduce<State, Value, Action>(
    prev: &State,
    action: Action,
    reducer: Reducer<State, Value, Action>,
    effects: Effects,
) -> (Value, Effects) {
    let (next, changed) = reducer(prev, action);
    let effects = if changed {
        effects
    } else {
        Effects::none().unchanged()
    };
    (next, effects)
}
