# pure-tv-addon → core alignment plan

Direction: the addon follows core, not the other way around. `LiveTvGuide` (see
[LIVE_TV_EPG.md](LIVE_TV_EPG.md)) treats **`catalog` + `date` extra → `metasDetailed`** as
the contract; per-channel `meta` is the depth path (multi-day, MetaDetails), not the way
the grid is fed. Steps below are against `mrcanelas/pure-tv-addon` as reviewed 2026-07-02
(files: `addon.js`, `src/manifest.js`, `src/iptv.js`, `src/serverless.js`).

## 1. Manifest (`src/manifest.js`)

Declare the `date` extra on the catalog. Core sends `date` to epgProvider addons
unconditionally (defaulting to today when the user hasn't picked one) — `epgProvider:
true` itself signals `date` support, so this declaration is documentation for other
clients rather than a gate. The hard requirement is step 2: the handler must honor
`extra.date` and default to today when absent:

```js
catalogs: [{
    type: 'tv', id: `${ADDON_PREFIX}catalog`, name: 'PureTV',
    extra: [{ name: 'date' }, { name: 'skip' }]
}],
```

Keep `behaviorHints: { configurable: true, epgProvider: true }` as is.

## 2. Catalog handler (`addon.js`) — return `metasDetailed` with the day's programs

Today it returns bare `metas` (no videos, no date awareness). Change it to:

1. Parse `extra.date` (`YYYY-MM-DD`); default to the current UTC date when absent.
   Interpret it as the UTC day window `[date 00:00Z, date+1 00:00Z)` and include every
   program that **overlaps** the window (a show spanning midnight belongs to both days).
2. For the page of channels (`skip` slice), attach that day's programs as `videos` and
   return under the `metasDetailed` key (core parses this into full `MetaItem`s in one
   request):

```js
const { playlist, xmltv, index } = await getState(cfg)
const day = parseDateExtra(extra.date)            // -> { startMs, endMs }
const page = m3uChannelsToMetas(ADDON_PREFIX, playlist.channels || [])
    .slice(skip, skip + CATALOG_PAGE_SIZE)
const metasDetailed = page.map((meta) => ({
    ...meta,
    behaviorHints: { hasScheduledVideos: true },
    videos: programsForChannelOnDay(index, xmltv, meta.id, day),  // see step 4
}))
return { metasDetailed, cacheMaxAge: 300, staleRevalidate: 1800, staleError: 604800 }
```

Notes:
- Core's `ResourceResponse` deserializer requires **exactly one** top-level content key —
  return `metasDetailed` only, never `metas` alongside it.
- If `stremio-addon-sdk` validation rejects a `metasDetailed` response from
  `defineCatalogHandler`, bypass the builder for this route in `src/serverless.js` (the
  project already owns its router) rather than forking the SDK.
- Raise `CATALOG_PAGE_SIZE` from 20 toward 50–100: with one-day programs per channel the
  page stays in the low single-digit MB (guidelines in [TODO.md](TODO.md)); fewer pages =
  fewer round-trips for the grid.

## 3. Precompute the channel→programs index (`src/iptv.js`)

`xmltvToVideosForChannel` currently scans **all** XMLTV programs per channel per request
(O(programs × channels) per catalog page, including the fuzzy `includes` fallback).
Build the index once per `getState` cache entry:

- In `buildChannelIndex` (or a sibling), resolve each channel's XMLTV id candidates once
  (the display-name matching currently done in the meta handler, `addon.js:72-92`, moves
  here), then bucket `xmltv.programs` into a `Map<channelId, Video[]>`, sorted by
  `startTime`.
- Catalog and meta handlers both read from this map; day filtering is a binary
  search / linear slice over the sorted array, not a rescan.

## 4. Video shape fixes (`src/iptv.js`, `xmltvToVideosForChannel` mapper)

- **`released = startTime`** — core's compat convention (Calendar, notifications, and
  stremio-web's schedule list key on `released`). The original XMLTV air date must NOT go
  into `released`.
- Keep the original air year in `releaseInfo` (this is the "2018" shown in the show
  modal), but **guard the crash**: `p.date` is optional in XMLTV and
  `new Date(undefined).toISOString()` throws, 500-ing the whole response today:

```js
const airDate = p.date ? new Date(p.date) : null
const releaseInfo = airDate && !isNaN(airDate) ? String(airDate.getUTCFullYear()) : undefined
```

- Keep `startTime`/`endTime` as UTC ISO strings (already correct — core parses them into
  the flattened `VideoEpgInfo`).
- Keep flat `genres`/`cast`/`directors`/`runtime`/`overview`/`thumbnail` (already match
  the core shape). `subtitle` is unmodeled in core — harmless (unknown fields are
  ignored), keep or drop.
- Show ids: keep the `pure:{channelHash}:{programHash}` convention, but note
  `stableId(title + start)` concatenates a string with a `Date` object — fine as long as
  it stays deterministic per process; prefer `title + start.toISOString()` so ids are
  stable across runtimes/locales (ids feed dedup and library state in core).

## 5. Meta handler (`addon.js`) — becomes the depth path only

- Keep returning the full multi-day program (this is exactly core's per-channel depth
  path and the MetaDetails fallback).
- Add `behaviorHints: { hasScheduledVideos: true }` to the channel meta so schedule-aware
  meta-details UIs activate.
- Reuse the precomputed index from step 3 (removes the duplicated candidate-matching
  block).

## 6. Stream handler (`addon.js`) — minor hardening

- Resolution of show id → channel id via `id.split(':')[1]` works for both `pure:{ch}`
  and `pure:{ch}:{prog}`; keep it.
- Add `behaviorHints: { notWebReady: true }` to the stream when the IPTV source isn't
  CORS/HTTPS/HLS-safe for the web player — live M3U sources usually aren't; this makes
  desktop/external-player routing kick in instead of failing silently on web.

## 7. Caching for date-scoped catalogs

Per-date URLs (`.../date=2026-07-02&skip=0.json`) are CDN-friendly — everyone asks for
the same day. Current 5 min `cacheMaxAge` + SWR is fine as a start; a later refinement is
`cacheMaxAge = seconds until the earliest endTime in the page` so a page never serves a
finished show as "live" (mirrors the core-side open question in [TODO.md](TODO.md)).

## 8. Verification checklist

- `GET /{cfg}/catalog/tv/pure:catalog/date=2026-07-02.json` → single `metasDetailed` key;
  every entry has `id`, `type: "tv"`, `name`, `videos[]` with UTC `startTime`/`endTime`
  overlapping that UTC day; `released === startTime` on every video.
- Same URL with `&skip=50` pages correctly; page payload ≤ low single-digit MB.
- `date` omitted → today's programs (not the full XMLTV dump).
- `GET .../meta/tv/pure:{ch}.json` → multi-day videos + `hasScheduledVideos: true`; a
  program without an XMLTV `date` element no longer 500s.
- `GET .../stream/tv/pure:{ch}:{prog}.json` and `.../stream/tv/pure:{ch}.json` both
  return the channel stream.
- Round-trip against core once `VideoEpgInfo` lands: catalog response deserializes via
  `ResourceResponse::MetasDetailed` with `epg_info` populated (`cargo test` serde suite).
