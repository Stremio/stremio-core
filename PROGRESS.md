# Live TV / EPG implementation progress

Branch: `feat/epg-support-addons`. Design: [LIVE_TV_EPG.md](LIVE_TV_EPG.md); home-screen
plan & guidelines: [TODO.md](TODO.md); addon alignment: [PURE_TV_ADDON_ALIGNMENT.md](PURE_TV_ADDON_ALIGNMENT.md).

## Done

- **Iteration 0 — design docs.** LIVE_TV_EPG.md, TODO.md, PURE_TV_ADDON_ALIGNMENT.md
  committed for reference.
- **Iteration 1 — `epgProvider` manifest behavior hint.**
  `ManifestBehaviorHints.epg_provider: bool` (`#[serde(default)]`, camelCase →
  `epgProvider`) in `src/types/addon/manifest.rs`; serde round-trip test updated
  (`src/unit_tests/serde/manifest_behavior_hints.rs`) and `DefaultTokens` impl extended
  (`src/unit_tests/serde/default_tokens_ext.rs`). `cargo test --lib manifest`: 7 passed.

- **Iteration 2 — `VideoEpgInfo` on `Video`.**
  New `VideoEpgInfo` struct (`start_time`/`end_time` required, `runtime`, `release_info`,
  `genres`, `cast`, `directors`, `links` optional) flattened as
  `Video.epg_info: Option<VideoEpgInfo>` in `src/types/resource/meta_item.rs`, mirroring
  the `series_info` pattern — its presence marks a video as a program show. Added
  `VideoEpgInfo::is_live(now)` (`[start, end)` semantics) and a serde test
  (`video_epg_info` in `src/unit_tests/serde/video.rs`) covering EPG payloads, absence
  (regular videos keep `epg_info: None`) and camelCase round-trip. Also fixed the fallout
  of iteration 1 that the targeted test run missed: exact-body fetch fixtures in
  `src/unit_tests/ctx/{install_addon,push_addons_to_api}.rs` now include
  `"epgProvider":false`. Full suite: `cargo test --lib` 218 passed (also with
  `--test-threads=1`).

- **Iteration 3 — `LiveTvGuide` model.**
  `EPG_DATE_EXTRA_PROP` (`date`) in `src/constants.rs`. New model
  `src/models/live_tv_guide.rs`: `Selected { request, date }` (request defaults to the
  first guide catalog, date to today via `E::now()`), `Selectable` (guide catalogs of
  `epgProvider` addons with ready-to-dispatch requests + prev/next/today dates),
  `catalog: Option<ResourceLoadable<Vec<MetaItem>>>` (parses `metasDetailed`), and derived
  `channels: Vec<ChannelGuide { channel, shows }>` — shows filtered to the selected UTC
  day (overlap semantics, midnight-spanning shows appear on both days), sorted by start
  time; videos without `epg_info` excluded. The `date` extra is appended unconditionally
  for epgProvider addons (the hint implies support; catalogs need not declare it).
  `ActionLoad::LiveTvGuide(Option<Selected>)` added. End-to-end runtime test in
  `src/unit_tests/live_tv_guide/load_action.rs` (request URL, date default, selectable,
  filtering, ordering). Full suite: 219 passed; clippy clean.
  NOT yet included: `skip` pagination (`LoadNextPage`) — follow-up after the web bridge.

- **Iteration 4 — `LiveTvGuideDeepLinks`.**
  New deep link struct in `src/deep_links/mod.rs`: `From<&NaiveDate>` →
  `stremio:///livetv/{YYYY-MM-DD}` and `From<(&ResourceRequest, &NaiveDate)>` →
  `stremio:///livetv/{base}/{type}/{id}/{date}` (Discover-style encoded catalog request).
  Show-level links intentionally reuse the existing `VideoDeepLinks` /
  `MetaItemDeepLinks`. Test: `src/unit_tests/deep_links/live_tv_guide_deep_links.rs`.
  Full suite: 220 passed.

- **Iteration 5 — web bridge.**
  `stremio-core-web`: `WebModel.live_tv_guide` field + `get_state` arm
  (`src/model/model.rs`), `serialize_live_tv_guide.rs` (selectable catalogs/dates with
  guide deep links, catalog load state as `Loadable<(), &ResourceError>` without
  duplicating metas, channels with `MetaItemDeepLinks`, shows serialized as flattened
  `Video` + computed `isLive` (via `WebEnv::now()`) + `VideoDeepLinks` built against a
  synthesized `meta/{type}/{id}` request on the guide addon's base — this is the modal's
  Play button), and a `DeepLinksExt` impl for `LiveTvGuideDeepLinks`.
  GOTCHA: `stremio-core-web`'s `model` module is declared INLINE in `src/lib.rs` (lines
  5-64) — the `src/model/mod.rs` / `deep_links_ext/mod.rs` files are DEAD leftovers; new
  serializers must be registered in `lib.rs`, not mod.rs.
  Verified: `cargo test` (root, CI-style) 220 + 18 passed; `cargo check` on
  stremio-core-web clean. NOTE: `cargo test --workspace` fails ~72 TestEnv-based tests
  even on clean `development` (pre-existing mutex poisoning under workspace feature
  unification, machine-local) — not caused by this branch.

- **Iteration 6 — `skip` pagination.**
  `LiveTvGuide.catalog` is now `Vec<ResourceLoadable<Vec<MetaItem>>>` (pages, mirroring
  `CatalogWithFilters`). `Selectable.next_page: Option<SelectablePage>` — present only
  when the selected catalog declares the `skip` extra (pagination is opt-in via the
  manifest, unlike the implied `date`) and all requested pages are loaded; `skip` = sum of
  loaded page sizes; the page request carries `skip` + `date` extras (`extend_one`
  PREPENDS, so the URL is `skip=N&date=...`). New `ActionLiveTvGuide::LoadNextPage` +
  `Action::LiveTvGuide` variant. `channels` are derived across all ready pages, deduped
  by channel id. Serializer: `catalog` = per-page `Loadable<(), &ResourceError>` states,
  `selectable.nextPage` exposed, channel meta requests built from the selected request's
  base. Runtime test extended with a second page (request URL, appended channels, skip
  count). GOTCHA (caused a local test-suite freeze): `runtime.model()` returns a read
  guard — binding `&runtime.model().unwrap().field` extends the guard's lifetime to the
  end of the test, deadlocking the next `dispatch` (write lock) and stalling every other
  TestEnv test on the env mutex; scope model reads in `{ }` blocks between dispatches.
  Full suite: 220 passed; clippy clean.

- **Iteration 7 — Discover skips epgProvider catalog content.**
  Found during live testing (2026-07-03): selecting a guide catalog fired the Discover
  poster-catalog request alongside the guide request. Per the design ("dispatch
  LiveTvGuide instead of CatalogWithFilters"), `CatalogWithFilters` now skips the content
  fetch when the selected `catalog` request targets an `epgProvider` addon
  (`is_epg_guide_request` in `src/models/catalog_with_filters.rs`) — `selectable`
  (dropdowns) still computes from manifests. Also guarded `next_page` against an empty
  catalog (would otherwise offer a bogus `skip=0` page). Regression test:
  `load_epg_guide_catalog_skips_content_request`. Full suite: 222 passed.
  NOTE: the remaining duplicate requests seen in dev come from React StrictMode
  double-mounting (upstream stremio-web behavior, dev-only).

## Next

- Later (see TODO.md): filter EPG channels out of Continue Watching; "Live channels"
  Board row; frontend (stremio-web) routes for `#/livetv/...`.
