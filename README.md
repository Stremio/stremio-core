# stremio-state

State container for stremio

This contains 3 crates

* `types`
* `state_types`
* `middlewares`




Also see:
* https://github.com/stremio/stremio-players
* https://github.com/Stremio/labs/issues/20

```
cargo clippy
cargo fmt
```


## Adding new actions

Defining actions and what middleware requests they should trigger is defined in `src/state_types/msg/actions`
