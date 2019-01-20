## DONE

* put in web
* decided to allow creating separate state containers, and do that for separate sections (board, player, etc.)
* all the basics for addons: a Vec of AddonDescriptors
* decided that there'd be no events out, just a pipeline of actions and middlewares
* basic grouping catalogs: CatalogsFiltered{ params: ...,  }, CatalogsGrouped; Actions would need some work
* CatalogsGrouped
* figure out how the player will be integrated: as a middleware that wraps around the JS (wasm-bindgen makes this easy)
* middleware model; basic rules: actions go through; user dispatches from the beginning; each middleware has one input, one output; 
* web: build a proper example with fetch()
* figure out actions polymorphism: we need to be able to easily match, serialize and etc.; for now, monolithic list of actions is OK
* think whether this whole wasm-bindgen thing violates my own philosophy against bindings: nah, it's OK
* Handler trait (`impl state_types::Handler for UserMiddleware`)
* fix middleware async shit
* environment: decide how to do the data structure: back to Traits?
* environment: implement `fetch_serde<T>`: easier ergonomics
* learn more about futures: https://aturon.github.io/blog/2016/08/11/futures/ (select, join, `or_else`, map)
* race condition protection: CatalogReq, CatalogResp matching
* go through all routes.js and figure out how we'll begin loading them
* assign time to Nikola to work on this (~1-2 months)
* make the catalogs work: middlewares: UserMiddleware (dummy-ish) and CatalogMiddleware
* remove reqwest from the core
* web environment implementation, err handling (JSON parsing, etc.)
* clippy
* reducer multiplexer handler; or just a single StateContainer wrapper, and then the user must construct a compound state container themselves; also, we have to remove the NewState actions with the full state, and make it only a notification; .dispatch of the container should return boolean whether is changed
* state container will have IDs so that Actions can target them
* CatalogGrouped: we only wanna render the first ~30
* find the reason for calls being slow: `get_state` takes ~50ms; optimized it by reducing the amount of data
* Environment: basic storage
* Optimization: ability to subscribe with a whitelist; for actions not matching the whitelist, subscribe only to the *occurrence*, so that we can manually `get_state()` if needed at the end of the tick (`setImmediate`)

## TODO
* environment: storage err handling
* environment: `fetch_serde` should support HTTP headers: pairs?
* implement UserMiddleware; think of how (or not to?) to mock storage in the test
* tests: Chain, Container, individual middlewares, individual types
* basic state: Catalog, Detail; and all the possible inner states (describe the structures)
* `AddonHTTPTransport<E: Environment>`
* consider splitting Environment into Storage and Fetcher
* load/unload dynamics and more things other than Catalog: Detail, StreamSelect
----
* Stream: new SPEC; we should have ways to filter streams too (e.g. HTTP + directly playable only)
* Trait for meta item and lib item; MetaPreview, MetaItem, MetaDetailed
* CatalogsGrouped to receive some info about the addon
* implement CatalogsFiltered
* start implementing libitem/notifitem addon
* since a lot of things are asynchronous, perhaps we should have a guard; the things to think about are: addon set hash, addon ID, user ID, etc.
* stuff to look for to be re-implemented: syncer, libitem/notifitem addons, discover ctrl, board ctrl, detail ctrl
* complex async pieces of logic: open, detectFromURL, openMedia 
* opening a file (protocol add-ons to be considered)
* https://blog.cloudflare.com/cloudflare-workers-as-a-serverless-rust-platform/
* chart everything, the entire stremio architecture, including core add-ons and such
* crates: stremio-web-environment (only the Environment), stremio-state-ng-web (general API that is exported to JS via bindgen)
* we should make it so that if a session is expired, we go to the login screen; this should be in the app
* think of how to do all edge cases in the user, such as pre-installing add-ons
* behaviorHints - pair (key, val)?
* separate crates: types, `state_types`
* when saving the last stream, save the whole object but compressed
* ensure environment caches are in place via the service worker (web)
* consider: flag `is_in_lib` for catalog items
* `get_state` takes a lot of time for a lot of data: investigate
* https://github.com/woboq/qmetaobject-rs based UI; needs reqwest (or someting else) async requests
* libitem/notifitem: https://developers.cloudflare.com/workers/kv/ https://blog.cloudflare.com/cloudflare-workers-as-a-serverless-rust-platform/
* think of whether this could be used with the Kodi codebase to make stremio embedded

UserMiddleware
CatalogsMiddleware
DetailMiddleware
AddonsMiddleware
AnalyticsMiddleware
PlayerJSMiddleware

example pipeline:
Load('catalog') => this will change the state of the `catalogs` to `Loading`
LoadFromAddons(addons, 'catalog') => emitted from the UserMiddleware
many AddonRequest(addon, 'catalog')
many AddonResponse(addon, 'catalog', resp) => each one would update the catalogs state

---------

// -> UserMiddleware -> CatalogMiddleware -> DetailMiddleware -> AddonsMiddleware ->
// PlayerMiddleware -> LibNotifMiddleware -> AnalyticsMiddleware -> join(discoverContainer, boardContainer, ...)

// perhaps insert HTTPMiddleware at the end

## Universe actions: 
UserDataLoad
InitiateSync (or separate events; see https://github.com/Stremio/stremio/issues/388)
BeforeClose
SettingsLoad
TryStreamingServer (this will try connecting to the streaming server, as well as probing it's settings and version)
NetworkStatusChanged

## Actions from the user

Load reducerType reducerId ...
	works for opening catalogs/detail/load/search
	reducerType is needed so that middlewares know to react; we can remove that by instructing the middlewares which reducerIds they should care about
	the library middleware will try to attach a selected type if there isn't one
	the catalogs middleware should dispatch msgs intended for the grouped or filtered reducers; the search should go through to the grouped; we will use separate reducerIds for board/search
Unload
TryLogin
TrySignup
TryOpen libItem|metaItem intent videoId?
TryOpenURL
LibAdd type id
LibRemove type id
LibRewind type id
LibDismissAllNotifs type id
LibSetReceiveNotifs type id
LibMarkWatched type id
LibMarkVideoWatched type id videoId true|false
TryAddonRemove
TryAddonAdd
TryAddonOpenURL - consider if this should be merged with TryOpenURL
NotifDismiss id
PlayerSetProp
PlayerCommand

## Settings middleware:

It will persist settings in storage

## User middleware:
...
LoginOrSignupError 
UserChanged
AddonCollectionChanged
AddonAdded
AddonRemoved
UserDataPersist
LoadWithUser user addons ...

uses Storage to save authKey, user and AddonCollection (to 1 storage key)

## Catalog middleware
transforms LoadWithUser, and then AddonAdded/AddonRemoved into -> AddonRequest + AddonResponse

## Detail middleware
transforms LoadUserUser, and then AddonAdded/AddonRemoved into -> AddonRequest + AddonResponse
this goes for the LibItem, for meta and for the streams 
Think of how to architect the StreamsPicker; it might need to be a separate reducer; in this case the middleware must be renamed to "DetailAndStream"

## AddonCatalog mdidleware

## Player (player spec wrapper) middleware:
LibItemPlayerSave (will be consumed by library addon middleware)
alternatively, LibItemSetTime/LibItemSetVideoID
... everything from the player spec
ProposeWatchNext
this should also start loading the subtitles from addons and such
all/most player actions should carry context in some way (stream, or at least stream ID, and maybe video ID, item ID)
this middleware uses Storage to persist PlayerPreferences (volume, subtitles, subtitles size, etc.); we must keep preferences for last N watched `(item_id, video_id)`
the algo to do this is simple; when we play something, we bump it to the end of the array; when we need to add something, we add to the end and pop from the middle (if `len>N`)
This should save the selected `(video_id, stream)` for the given `item_id`), when we start playing
we also need to load `meta` to be able to `ProposeWatchNext`

## Analytics middleware:

needs to take installationID as an arg

## Library/Notifications middleware:
ItemUpdated
SyncCompleted


Final reducers will be catalog, library, notifications, detail, player, settings, intro

player reducer should accurately reflect states like subtitles (from addons) or subtitle files (vtt) loading

------

Initial flow to be implemented:
LoadCatalog -> (user middleware does this) WithUser(user, addons, LoadCatalog) -> AddonRequest, AddonResponse

The reducer, upon a LoadCatalog, should .clone() the action into it's state, and then discard any AddonRequest/AddonResponse that doesn't match that


