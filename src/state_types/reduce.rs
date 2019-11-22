use crate::state_types::Effects;

pub type Reducer<T, Deps> = fn(prev: &T, deps: Deps) -> (T, bool);

pub fn reduce<T, Deps>(
    prev: &T,
    deps: Deps,
    reducer: Reducer<T, Deps>,
    effects: Effects,
) -> (T, Effects) {
    let (next, changed) = reducer(prev, deps);
    let effects = if changed {
        effects
    } else {
        Effects::none().unchanged()
    };
    (next, effects)
}
