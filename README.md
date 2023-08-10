## Stremio - the next generation media center
[![Build Workflow Status](https://img.shields.io/github/actions/workflow/status/Stremio/stremio-core/build.yml?label=Build)](https://github.com/Stremio/stremio-core/actions/workflows/build.yml)
[![Latest MSRV workflow Status](https://img.shields.io/github/actions/workflow/status/Stremio/stremio-core/msrv.yml?label=MSRV)](https://github.com/Stremio/stremio-core/actions/workflows/msrv.yml)
[![Latest deployed docs on GH pages](https://img.shields.io/github/actions/workflow/status/Stremio/stremio-core/docs.yml?event=workflow_dispatch&label=Latest%20deployed%20Docs)](https://stremio.github.io/stremio-core)

Stremio is a full-featured media center designed to help you organize and stream your favorite videos, movies and TV series. It will notify you for new episodes / movies, and allow you to find new content through Discover.

Stremio allows, using its [Add-ons system](https://github.com/Stremio/stremio-addon-sdk), to play movies, TV series and channels instantly.

## stremio-core

`stremio-core` is a rust crate that's designed to contain all the reusable logic between Stremio versions.

### Goals

* Flexibility - can be integrated into existing code bases, across the entire stack, and in different paradigms
	* use case: `types` can be used by add-ons
	* use case: can be used with existing user authentication as an addition to an existing app
	* use case: can use the `Context` model to manage the user authentication/addons, using it as a backbone to the entire Stremio app
* Emphasis on correctness
* No cruft / legacy - not burdened by obsolete decisions & solutions

### Modules

* `types`
* `addon_transport` - handles communication with add-ons, implements legacy protocol adapter
* `state_types`: types that describe application state; inspired by the Elm architecture
	* Effects and Update traits
	* `runtime`: helps using `stremio-core` in an application by handling the effects automatically
	* `environment`: a trait describes the environment (fetch, storage)
	* `msg`: messages: actions, events
	* `models`: all stateful models, such as `Context` (handling user authentication, add-ons), `Library`, `CatalogFiltered`, etc.



```
cargo clippy
cargo fmt
```

## Optimizing WASM output

WASM output binary can get large, especially if we derive Serialize/Deserialize in places we don't need to

We can optimize it by running twiggy: `twiggy top ..._bg.wasm` and seeing what the biggest code size offenders are


## Adding new actions

Defining actions and what middleware requests they should trigger is defined in `src/state_types/msg/actions`
