# stremio-core

`stremio-core` is a rust crate that's designed to contain all the reusable logic between Stremio versions.

### Goals

* Flexibility - can be integrated into existing code bases, across the entire stack, and in different paradigms
	* use case: `types` can be used by add-ons
	* use case: can be used with existing user authentication as an addition to an existing app
	* use case: can use `ContextM` to manage the user authentication/addons, using it as a backbone to the entire Stremio app
* Emphasis on correctness
* No cruft / legacy - not burdened by obsolete decisions & solutions

### Modules

* `types`
* `libaddon`
* `addon_transport`
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
