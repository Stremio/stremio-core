# Check a movie before navigating to its sources

This draft implements the first part of the [web proposal](https://github.com/Stremio/stremio-web/pull/1454): an explicit check for one selected movie. It does not implement catalog filtering or claim that a returned video source will play successfully.

## Interface

The web model exposes a separate `source_preview` field. Start a check with:

```json
{
  "action": "Load",
  "args": { "model": "SourcePreview", "args": "tt1254207" }
}
```

Dispatch that action to `source_preview`, then read the same field. A response looks like:

```json
{ "id": "tt1254207", "status": "SourcesFound" }
```

`Unload` clears the check. It does not unload `meta_details`.

## Behavior

The model reuses the existing metadata lookup, default-video selection, embedded-source handling, and addon matching helpers. It does not run the rating, watched-state, or library synchronization work performed by a full details load. It also does not play a video or fetch the video URL.

The possible results are:

| Status | Meaning |
| --- | --- |
| `Unknown` | No active check, or the profile changed. |
| `Checking` | Relevant responses are pending. |
| `SourcesFound` | At least one non-external source was returned. Device compatibility is not verified. |
| `ExternalOnly` | Responses contain only external-service links. |
| `NoSources` | Checks completed without video sources. |
| `CheckFailed` | A necessary check failed, rather than successfully returning an empty result. |
| `NoProvider` | No matching metadata provider, or no stream provider after resolving the video and checking embedded sources. |
| `ChooseVideo` | Metadata requires an explicit video selection. Do not label the movie unavailable. |

Positive results can be shown while another addon is pending. Embedded sources have the same precedence as in the existing details serializer. `EmptyContent` counts as an empty result; transport and parsing errors remain failures.

Each attempt owns a generation number. Its resource responses use a dedicated internal message, so they cannot update an unrelated details model. Unload, retry, and profile changes invalidate earlier generations. The web serialization contains only the requested ID and status, not configured addon URLs or credentials.

## Scope and remaining work

The first consumer checks only a movie explicitly selected by the user. It uses the existing request fan-out across matching installed addons. It has no background catalog traversal, batch scheduler, shared result cache, or persisted state. No storage migration is needed.

Logical cancellation ignores late responses; it does not abort network requests already issued by the existing transport. The web dialog stops waiting after 15 seconds and unloads the model. Transport-level deadlines and cancellation remain follow-up work before extending this to background batches.

Playback support, expiring URLs, subscription requirements, and streaming-server health remain outside this result. Series need episode-specific checks and are not intercepted by the first web implementation.

## Verification

The unit tests cover success mixed with errors/pending responses, empty versus failed requests, external links, embedded-source precedence, unresolved video selection, the metadata-to-video request flow, absence of library effects, and stale responses after unload/profile changes.

Run `cargo test`, `cargo clippy --all --no-deps -- -D warnings`, and `cargo fmt --all -- --check`. Build `stremio-core-web` with its existing npm build scripts to test the browser integration. The companion web PR documents how to link the local build before a core package release.

A wider `cargo test --all` run currently exposes existing JSON key-order assertions when workspace features are unified. The legacy `stream_imdb` failure was reproduced on the unchanged development revision. The normal root `cargo test` suite passes; this draft does not alter those unrelated assertions.
