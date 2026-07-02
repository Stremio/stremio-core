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

## Next
- Iteration 4: `LiveTvDeepLinks` (`stremio:///livetv/{date}`).
- Iteration 5: web bridge — `WebModel.live_tv_guide` + `serialize_live_tv_guide.rs`
  (per-show `deepLinks` + computed `isLive`).
- Later (see TODO.md): filter EPG channels out of Continue Watching; "Live channels"
  Board row.
