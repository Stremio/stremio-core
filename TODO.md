# TODO — Live TV (`epgProvider`) addon support

## Watched channels on the Home screen (Board)

Problem: once a user plays a channel, `Player` creates/updates a `LibraryItem` for it
(`type: "tv"`), and as soon as `state.time_offset > 0` the channel satisfies
`LibraryItem::is_in_continue_watching()` (src/types/library/library_item.rs:52) and shows up
in Continue Watching. For live content this is wrong on two levels:

- `time_offset` / `duration` / `progress()` are meaningless for a live stream — the resume
  bar would render garbage.
- The poster/name stored in the `LibraryItem` describe the *channel*, not what is currently
  airing, so the CW card would be stale/uninformative.

### Options considered

1. **Filter live channels out of Continue Watching entirely (short term — do this first).**
   - Exclude items of EPG channels in `ContinueWatchingFilter` (src/models/library_with_filters.rs)
     and `ContinueWatchingPreview` (src/models/continue_watching_preview.rs).
   - Cheapest signal available today: `library_item.r#type == "tv"`. More correct: persist a
     marker on the `LibraryItem` when playback started from an EPG addon (e.g. set a flag in
     `LibraryItemBehaviorHints`, mirroring how `MetaItemBehaviorHints` flows into the library
     item), so regular "tv"-typed VOD items are unaffected.
   - Zero network cost, no UX ambiguity, fully reversible later.

2. **Show channels in Continue Watching enriched with the current live show (later).**
   - Would require fetching each watched channel's program to know "what is on now" —
     i.e. one `meta/tv/{channelId}` (or one `catalog` request with an `ids`-style extra,
     like `CALENDAR_IDS_EXTRA_PROP` / `LAST_VIDEOS_IDS_EXTRA_PROP`) per EPG addon.
   - The notifications pipeline (`ctx.notification_catalogs`, `lastVideosIds` extra) is the
     existing precedent for "periodically pull fresh video data for library items"; the same
     mechanism could pull "now airing" for watched channels.
   - Downsides: EPG data expires by the minute (CW rows update on `LibraryChanged`, not on a
     clock), and mixing "resume VOD at 42:10" cards with "live now" cards overloads the CW
     concept.

3. **Dedicated "Live channels" row on the Board (preferred end state).**
   - A small model (or an extension of `ContinueWatchingPreview`-style preview models) that:
     - takes watched/pinned channels from the library (`type == "tv"`, EPG-flagged),
     - issues one aggregated request per EPG addon for the current shows,
     - exposes `Vec<{ channel, now_playing: Option<Video>, deep_links }>`.
   - Clicking a card jumps straight into playback (channels are always "live", there is
     nothing to resume), long-press/secondary action opens the guide at that channel.
   - Keeps Continue Watching semantically clean (resumable VOD only).

### Decision (proposed)

- **Now:** option 1 — filter EPG-channel library items out of both Continue Watching
  surfaces. Keep writing the `LibraryItem` on playback (so we retain the "watched channels"
  history and can rank the future Live row by `mtime`).
- **Next:** option 3 — dedicated Board row backed by a `LiveChannelsPreview`-style model that
  fetches "now airing" per channel from the EPG addon(s).
- Option 2 is rejected as the end state; at most it becomes an implementation detail of
  option 3's data fetch.

## Addon documentation / guidelines (for the addon SDK docs)

When documenting the `epgProvider` contract, spell out the payload-size expectations —
page size is the addon author's responsibility, not something core can enforce:

- The `date` extra caps the time dimension to one day; the `skip` extra caps the channel
  dimension. Recommend pages of 50–100 channels and state the target: a catalog page
  should stay in the low single-digit MB range.
- Warn about dense programs (e.g. 5-minute shows): with ~20–50 shows/channel/day at
  ~0.5–1 KB each, a channel costs ~10–50 KB — authors with denser schedules should shrink
  their page size accordingly.
- Recommend `genre`-style slicing extras (country/category/package) for large channel
  lineups so clients rarely page through the full list.
- Document the compat conventions: `released = startTime` on program videos, show ids
  prefixed with the channel id, `behaviorHints.hasScheduledVideos: true` on channel metas.

## Open questions

- Cache policy for EPG catalog responses (`cache_max_age` from `ResourceResponseCache`):
  program data is valid until the next show boundary, not for a fixed TTL. Should the guide
  model schedule a refresh at the earliest `endTime` of the currently visible shows?
- Timezone handling: `date` extra is sent as an ISO date — decide whether it is the user's
  local date (frontend converts) or UTC (addon returns 00:00–24:00 UTC and the frontend
  re-buckets). Leaning local-date-as-string, all show times as UTC `DateTime`.
- Do channels need a full MetaDetails page? Current answer: not required for the guide UX
  (modal + play covers it), but keeping the addon's `meta` resource for channels means the
  existing MetaDetails screen works for free (deep links from search/share). Program shows
  do NOT get their own meta pages — they are `Video`s of the channel.
