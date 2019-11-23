use crate::state_types::Effects;

pub type Reducer<T, Action> = fn(prev: &T, action: Action) -> (T, bool);

pub fn reduce<T, Action>(
    prev: &T,
    action: Action,
    reducer: Reducer<T, Action>,
    effects: Effects,
) -> (T, Effects) {
    let (next, changed) = reducer(prev, action);
    let effects = if changed {
        effects
    } else {
        Effects::none().unchanged()
    };
    (next, effects)
}
