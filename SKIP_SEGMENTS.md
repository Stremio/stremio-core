# Skip segment provider integration

This branch adds provider neutral skip segment evidence while preserving Stremio's existing native Skip Gaps entitlement rules.

## Sources

Stremio native Skip Gaps remains unchanged and is only requested when the existing Premium checks pass.

SkipDB is the portable external source. Its public read API is browser compatible and is used on every platform. SkipDB data is treated as transient read only evidence and is never written into Stremio's persistent skip segment cache. SkipDB attribution and data licence information are available at https://skipdb.tv and https://skipdb.tv/data.

On non WebAssembly clients, IntroDB and TheIntroDB provide additional independent evidence. They are not called directly from WebAssembly clients because live browser origin testing showed that IntroDB does not allow the Stremio Web origin and TheIntroDB may be blocked by Cloudflare from automated environments.

## Failure isolation

Providers are requested independently. No external provider is awaited before playback continues and failure from one provider does not suppress evidence from another provider or Stremio native Skip Gaps.

Every provider result carries the playback context that initiated it. Results for an earlier episode or duration are discarded after playback moves on.

## Resolution

Provider responses are converted into a common candidate model before entering the player decision path. The resolver prefers more stream specific evidence, then better match quality, unadjusted evidence, stronger community evidence and provider confidence.

The resolver fails closed. Weak standalone external evidence is not exposed. Independent sources can establish a segment when they materially agree. Trusted peers at the same evidence level suppress the result when their timings materially conflict. Invalid ranges, tiny segments and implausibly long intro ranges are rejected.

SkipDB results reported as out of range or otherwise non exact are treated as estimated evidence. They do not establish a skip segment on their own at low confidence.

External evidence only changes the segment kinds it actually supplies. Existing native intro or outro values are preserved when external providers have no trusted candidate for that kind.

## Cache

Successful cache eligible external candidates are cached locally per provider using IMDb identity, season, episode and exact duration. IntroDB and TheIntroDB therefore retain independent outage fallbacks. Cache entries expire after 30 days and a stale entry is deleted when encountered.

SkipDB candidates are explicitly excluded from persistent caching. This keeps the SkipDB integration within its read only usage model instead of using its data to populate a Stremio skip segment database.

Cached data is available as an outage fallback while fresh cache eligible provider reads run independently. A successful fresh result replaces only that provider's cached evidence. A failed fresh read leaves the other provider's valid cached fallback untouched.

## Privacy and credentials

The integration sends only public media identifiers and the minimum episode or duration information required by each provider. It does not send Stremio authentication material to external providers and does not embed shared provider API keys.

## Platform behaviour

WebAssembly clients use SkipDB directly because its read API returns permissive CORS headers for the Stremio Web origin.

Native clients use SkipDB plus IntroDB and TheIntroDB. Stremio native Skip Gaps remains an additional source whenever the existing Premium entitlement permits it.

This keeps the feature available across Stremio clients without relying on an operating system specific helper or weakening the existing Premium boundary.
