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
* CatalogsGrouped: we only wanna render the first ~30
* find the reason for calls being slow: `get_state` takes ~50ms; optimized it by reducing the amount of data
* Environment: basic storage
* Optimization: ability to subscribe with a whitelist; for actions not matching the whitelist, subscribe only to the *occurrence*, so that we can manually `get_state()` if needed at the end of the tick (`setImmediate`)
* environment: storage err handling
* SPEC: decide if a separate resource will be used for library/notifications; a separate return type (.libItems rather than .metas) is a must; DONE: seems it must be a catalog, otherwise it breaks the semantics of manifest.catalogs; we will restrict it via extraRequired
* Stream: new SPEC; we should have ways to filter streams too (e.g. HTTP + directly playable only)
* think whether stateful middlewares can be eliminated or mitigated with some memoization-inspired pattern

## TODO
* refactor: error handling: consider making an enum that will hold JsValue or other error types; see https://www.youtube.com/watch?v=B5xYBrxVSiE 
* environment: `fetch_serde` should support advanced HTTP requests: https://developer.mozilla.org/en-US/docs/Web/API/Request/Request
* statefulness can be mitigated by using a memoization where the addon transport `get` would return the same results if invoked with the same args again; however, this needs to be out of the transport impl and needs to be explicit
* implement UserMiddleware; think of how (or not to?) to mock storage in the test
* basic state: Catalog, Detail; and all the possible inner states (describe the structures); StreamSelect
* tests: Chain, Container, individual middlewares, individual types
* https://github.com/Stremio/stremio-aggregators/blob/master/lib/isCatalogSupported.js
* refactor: perhaps we can use Load(Target), where Target is an enum, and then wrap it in LoadWithUser(user, addons, Target) - if Load is the only place we need addons; we won't need Box<> and we can pattern match
* consider a Trait for the Load family of actions that will return an AddonAggrReq(OfResouce(resource, type, id, extra)) or AddonAggrReq(Catalogs(extra)); consider also OfAddon (for CatalogsFiltered); then, our AddonAggr middleware will spawn AddonReq/AddonResp; given a `transport_url`, OfAddon will try to find the addon in the collection, to possibly apply `flags.stremioAuth` or `flags.transport`; of course, it doesn't need to find it, `transport_url` is sufficient to request
* `get_state` is very slow: it takes a lot of time for large-ish amounts of data: investigate & open a github issue

* construct `AddonHTTPTransport<E: Environment>` and give it to the interested middlewares; introduce a long-lived transport
* consider splitting Environment into Storage and Fetcher; and maybe take AddonsClient in

* spec: notifItems: rethink that spec, crystallize it
* load/unload dynamics and more things other than Catalog: Detail, StreamSelect
* Trait for meta item and lib item; MetaPreview, MetaItem, MetaDetailed
* CatalogsGrouped to receive some info about the addon
* implement CatalogsFiltered; CatalogsFilteredPreview
* start implementing libitem/notifitem addon
* since a lot of things are asynchronous, perhaps we should have a guard; the things to think about are: addon set hash, addon ID, user ID, etc.
* stuff to look for to be re-implemented: syncer, libitem/notifitem addons, discover ctrl, board ctrl, detail ctrl
* environment: consider allowing a dynamic instance, esp for storage
* environment: the JS side should (1) TRY to load the WASM and (2) TRY to sanity-check the environment; if it doesn't succeed, it should show an error to the user
* complex async pieces of logic: open, detectFromURL, openMedia 
* opening a file (protocol add-ons to be considered)
* https://blog.cloudflare.com/cloudflare-workers-as-a-serverless-rust-platform/
* crates: stremio-web-environment (only the Environment), stremio-state-ng-web (general API that is exported to JS via bindgen)
* we should make it so that if a session is expired, we go to the login screen; this should be in the app
* think of how to do all edge cases in the user, such as pre-installing add-ons (implicit input)
* behaviorHints - pair (key, val)?
* separate crates: types, `state_types`
* when saving the last stream, save the whole object but compressed
* ensure environment caches are in place via the service worker (web)
* consider: flag `is_in_lib` for catalog items
* https://github.com/woboq/qmetaobject-rs based UI; needs reqwest (or someting else) async requests
* libitem/notifitem: https://developers.cloudflare.com/workers/kv/ https://blog.cloudflare.com/cloudflare-workers-as-a-serverless-rust-platform/
* think of whether this could be used with the Kodi codebase to make stremio embedded
* all the cinemeta improvements this relies on: e.g. behaviorHints.isNotReleased will affect the Stream view
* graph everything, the entire stremio architecture, including core add-ons and such
* ensure that every time a network error happens, it's properly reflected in the state; and the UI should allow to "Retry" each such operation

example pipeline:
LoadCatalogs => this will change the state of the `catalogs` to `Loading`
LoadFromAddons(addons, 'catalog') => emitted from the UserMiddleware
many AddonRequest(addon, 'catalog')
many AddonResponse(addon, 'catalog', resp) => each one would update the catalogs state

---------

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
we also need to load `meta` to be able to `ProposeWatchNext` (meant to be handled by asking the user or by implicit input)

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

# Routes

Presumes the following reducers

0: CatalogsGrouped (for board)
1: CatalogsFilteredWithPreview (for discover); @TODO: this might be two separate reducers: CatalogsFiltered, CatalogsFilteredPreview
2: CatalogsGrouped (for search)
3: Detail
4: Streams
5: CatalogsFiltered (for library)
6: CatalogsFiltered (for notifications; could be specific: CatalogsNotifGrouped)
7: AddonCatalog
8: PlayerView
9: SettingsView
@TODO a container for Continue Watching

@TODO figure reload/force policies for all of these; for now, we'll just always load everything (naively)

### ?apiURL

overrides the API URL
this will simply tweak the Environment

### ?addonURL=url

prompts the user to install an add-on or a collection
this should dispatch Actions::OpenAddonURL

### ?addonURLForce=Url

force adds the given add-on or collection of add-ons; dispatch Actions::InstallAddonURL
@TODO consider the security aspect of this

### /board

Dispatch LoadCatalogsGrouped(0) -> AddonAggrReq(Catalogs())

### /discover/:type/:addonID/:catalogID/:filters?&preview=ID

Dispatch LoadCatalogsFiltered(1, type, addonID, catalogID, filtered) -> AddonAggrReq(OfResource("catalog", type, catalogID, filters)) but match it only against the addon with addonID

@TODO addonTransportURL and OfAddon instead of addonID; more concise, allows URLs to work for other pepole too, and simplifies the middleware

If, for some reason, we use a `type` that's not available, the particular addon will return an error, which will be transformed into Loadable::Message and handled elegantly 

@TODO routing problem: if /discover is opened, we need to auto-select some (type, catalog, filters); we might just hardcode Cinemeta's top and always go to that

### /detail/:type/:id/:videoID?

Dispatch LoadDetail(3, type, id) -> AddonAggrReq(OfResource("meta", type, id))
if videoID, dispatch LoadStreams(4, type, id, videoID) -> AddonAggrReq(OfResource("stream", type, videoID)) ; this also needs to read the last selected stream from storage

The Library item and the notifications will be loaded through the AddonAggrReq(OfResource("meta", type, id)); that will match the library/notif addon, and return the results

### /library/:type

Dispatch LoadCatalogsFiltered(5, type, "org.stremio.library", "library", { library: 1 }) -> AddonAggrReq(OfResource("catalogs", type, "library", { library: 1 })) but match against library addon

If we do addonTransportURL+OfAddon, and we save the last selected `type` in the UI, If, for some reason, we use a `type` that's not available, the particular addon will return an error, which will be transformed into Loadable::Message and handled elegantly 

### Notifications (not a route, but a popover)

Dispatch LoadCatalogsGrouped(6) -> AddonAggrReq(Catalogs({ notifs: 1 }))

### /addons/:category/:type?

Category is Official, ThirdParty, Installed

Dispatch LoadAddonCatalog(7, category, type) -> middleware loads latest collection of the given category and filters by type 

### /player/:type/:id/:videoId/:streamSerialized

Dispatch LoadPlayer(8, type, id, videoId, streamSerialized) -> this will trigger many things, one of them AddonAggrReq(OfResource("meta", type, id))
	another one will be to load the libitem/notifications
	the player middleware should also request subtitles from the add-on system (AddonAggrReq(OfResource("subtitles", meta, id)))
	the player middleware should also keep an internal state of what the player is doing, and persist libitem/last played stream

### /calendar

@TODO
CalendarMIddleware needs to get the calendar from the stremio-web-services

### /intro

@TODO

### /settings

We need ot load the existing settings (settingsmiddleware might hold them anyway)
and we have to try to connect to the streaming server

@TODO
