use futures::FutureExt;
use http::Request;
use serde::Deserialize;
use std::cmp::Ordering;
use url::Url;

use crate::runtime::msg::{Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Env, EnvFutureExt};

use crate::types::{
    api::{SeekEvent, SkipGaps, SkipGapsResponse},
    skip_segments::{
        ResolvedSkipSegment, SkipSegmentCandidate, SkipSegmentContext, SkipSegmentKind,
        SkipSegmentMatch, SkipSegmentSource, SkipSegmentStreamSpecificity,
    },
};

pub(crate) fn strongest_seek_event(skip_gaps: &SkipGaps) -> Option<&SeekEvent> {
    skip_gaps
        .seek_history
        .iter()
        .filter(|event| event.records > 0 && event.to > event.from)
        .max_by(|left, right| {
            left.records
                .cmp(&right.records)
                .then_with(|| {
                    left.to
                        .abs_diff(left.from)
                        .cmp(&right.to.abs_diff(right.from))
                })
                .then_with(|| right.from.cmp(&left.from))
        })
}

fn native_specificity(accuracy: &str) -> SkipSegmentStreamSpecificity {
    let accuracy = accuracy.to_ascii_lowercase();
    if accuracy.contains("hash") {
        SkipSegmentStreamSpecificity::OpenSubtitlesHash
    } else if accuracy.contains("file") || accuracy.contains("name") {
        SkipSegmentStreamSpecificity::StreamName
    } else {
        SkipSegmentStreamSpecificity::Episode
    }
}

fn scale_timestamp(value: u64, source_duration: u64, target_duration: u64) -> u64 {
    if source_duration == 0 || source_duration == target_duration {
        return value;
    }

    ((value as u128 * target_duration as u128) / source_duration as u128) as u64
}

pub fn stremio_native_candidates(
    response: &SkipGapsResponse,
    target_duration_ms: u64,
) -> Vec<SkipSegmentCandidate> {
    if target_duration_ms == 0 {
        return Vec::new();
    }

    let Some((source_duration, skip_gaps)) = response
        .gaps
        .iter()
        .min_by_key(|(duration, _)| duration.abs_diff(target_duration_ms))
    else {
        return Vec::new();
    };

    let adjusted = *source_duration != target_duration_ms;
    let source_match = if adjusted {
        SkipSegmentMatch::DurationAdjusted
    } else {
        SkipSegmentMatch::ExactStream
    };
    let specificity = native_specificity(&response.accuracy);
    let mut candidates = Vec::with_capacity(2);

    if let Some(event) = strongest_seek_event(skip_gaps) {
        candidates.push(SkipSegmentCandidate {
            kind: SkipSegmentKind::Intro,
            start_ms: scale_timestamp(event.from, *source_duration, target_duration_ms),
            end_ms: scale_timestamp(event.to, *source_duration, target_duration_ms),
            source: SkipSegmentSource::StremioNative,
            source_match,
            source_confidence: None,
            adjusted,
            evidence_count: event.records.min(u32::MAX as u64) as u32,
            stream_specificity: specificity,
        });
    }

    if let Some(outro) = skip_gaps.outro {
        let start_ms = scale_timestamp(outro, *source_duration, target_duration_ms);
        if start_ms < target_duration_ms {
            candidates.push(SkipSegmentCandidate {
                kind: SkipSegmentKind::Outro,
                start_ms,
                end_ms: target_duration_ms,
                source: SkipSegmentSource::StremioNative,
                source_match,
                source_confidence: None,
                adjusted,
                evidence_count: 1,
                stream_specificity: specificity,
            });
        }
    }

    candidates
}

#[derive(Clone, Debug, Deserialize)]
struct IntroDbResponse {
    #[serde(default)]
    pub intro: Option<IntroDbSegment>,
    #[serde(default)]
    pub recap: Option<IntroDbSegment>,
    #[serde(default)]
    pub outro: Option<IntroDbSegment>,
}

#[derive(Clone, Debug, Deserialize)]
struct IntroDbSegment {
    #[serde(default)]
    pub start_ms: Option<u64>,
    #[serde(default)]
    pub end_ms: Option<u64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub submission_count: Option<u32>,
}

fn introdb_candidates(
    response: &IntroDbResponse,
    media_duration_ms: Option<u64>,
) -> Vec<SkipSegmentCandidate> {
    [
        (SkipSegmentKind::Intro, response.intro.as_ref()),
        (SkipSegmentKind::Recap, response.recap.as_ref()),
        (SkipSegmentKind::Outro, response.outro.as_ref()),
    ]
    .into_iter()
    .filter_map(|(kind, segment)| {
        let segment = segment?;
        let start_ms = segment.start_ms?;
        let end_ms = segment.end_ms?;
        (end_ms > start_ms && media_duration_ms.map_or(true, |duration| end_ms <= duration))
            .then_some(SkipSegmentCandidate {
                kind,
                start_ms,
                end_ms,
                source: SkipSegmentSource::IntroDb,
                source_match: SkipSegmentMatch::ExactEpisode,
                source_confidence: segment.confidence,
                adjusted: false,
                evidence_count: segment.submission_count.unwrap_or(0),
                stream_specificity: SkipSegmentStreamSpecificity::Episode,
            })
    })
    .collect()
}

#[derive(Clone, Debug, Deserialize)]
struct SkipDbResponse {
    pub segments: SkipDbSegments,
}

#[derive(Clone, Debug, Deserialize)]
struct SkipDbSegments {
    #[serde(default)]
    pub intro: Option<SkipDbSegment>,
    #[serde(default)]
    pub recap: Option<SkipDbSegment>,
    #[serde(default)]
    pub outro: Option<SkipDbSegment>,
    #[serde(default)]
    pub preview: Option<SkipDbSegment>,
}

#[derive(Clone, Debug, Deserialize)]
struct SkipDbSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    #[serde(default)]
    pub adjusted: bool,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(rename = "match")]
    pub match_kind: String,
}

fn skipdb_candidates(
    response: &SkipDbResponse,
    media_duration_ms: Option<u64>,
) -> Vec<SkipSegmentCandidate> {
    [
        (SkipSegmentKind::Intro, response.segments.intro.as_ref()),
        (SkipSegmentKind::Recap, response.segments.recap.as_ref()),
        (SkipSegmentKind::Outro, response.segments.outro.as_ref()),
        (SkipSegmentKind::Preview, response.segments.preview.as_ref()),
    ]
    .into_iter()
    .filter_map(|(kind, segment)| {
        let segment = segment?;
        (segment.end_ms > segment.start_ms
            && media_duration_ms.map_or(true, |duration| segment.end_ms <= duration))
        .then_some(SkipSegmentCandidate {
            kind,
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            source: SkipSegmentSource::SkipDb,
            source_match: match segment.match_kind.as_str() {
                "exact" => SkipSegmentMatch::ExactStream,
                "shifted" => SkipSegmentMatch::DurationAdjusted,
                _ => SkipSegmentMatch::Estimated,
            },
            source_confidence: segment.confidence,
            adjusted: segment.adjusted,
            evidence_count: 0,
            stream_specificity: if matches!(segment.match_kind.as_str(), "exact" | "shifted") {
                SkipSegmentStreamSpecificity::Duration
            } else {
                SkipSegmentStreamSpecificity::Episode
            },
        })
    })
    .collect()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TheIntroDbResponse {
    #[serde(default)]
    pub intro: Vec<TheIntroDbSegment>,
    #[serde(default)]
    pub recap: Vec<TheIntroDbSegment>,
    #[serde(default)]
    pub credits: Vec<TheIntroDbSegment>,
    #[serde(default)]
    pub preview: Vec<TheIntroDbSegment>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TheIntroDbSegment {
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub submission_count: Option<u32>,
}

pub(crate) fn theintrodb_candidates(
    response: &TheIntroDbResponse,
    media_duration_ms: Option<u64>,
) -> Vec<SkipSegmentCandidate> {
    let groups = [
        (SkipSegmentKind::Intro, response.intro.as_slice()),
        (SkipSegmentKind::Recap, response.recap.as_slice()),
        (SkipSegmentKind::Outro, response.credits.as_slice()),
        (SkipSegmentKind::Preview, response.preview.as_slice()),
    ];

    groups
        .into_iter()
        .flat_map(|(kind, segments)| {
            segments.iter().filter_map(move |segment| {
                let start_ms = segment.start_ms.unwrap_or(0);
                let end_ms = segment.end_ms.or(media_duration_ms)?;
                (end_ms > start_ms && media_duration_ms.map_or(true, |duration| end_ms <= duration))
                    .then_some(SkipSegmentCandidate {
                        kind,
                        start_ms,
                        end_ms,
                        source: SkipSegmentSource::TheIntroDb,
                        source_match: SkipSegmentMatch::ExactEpisode,
                        source_confidence: segment.confidence,
                        adjusted: false,
                        evidence_count: segment.submission_count.unwrap_or(0),
                        stream_specificity: SkipSegmentStreamSpecificity::Episode,
                    })
            })
        })
        .collect()
}

fn is_imdb_id(value: &str) -> bool {
    value.strip_prefix("tt").is_some_and(|digits| {
        (6..=10).contains(&digits.len()) && digits.chars().all(|c| c.is_ascii_digit())
    })
}

fn is_theintrodb_imdb_id(value: &str) -> bool {
    value.strip_prefix("tt").is_some_and(|digits| {
        (7..=8).contains(&digits.len()) && digits.chars().all(|c| c.is_ascii_digit())
    })
}

fn provider_url(base: &str, context: &SkipSegmentContext, duration_param: &str) -> Option<Url> {
    if !is_imdb_id(&context.item_id) {
        return None;
    }

    let mut url = Url::parse(base).expect("static skip provider URL must be valid");
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("imdb_id", &context.item_id);
        if let Some(season) = context.season {
            query.append_pair("season", &season.to_string());
        }
        if let Some(episode) = context.episode {
            query.append_pair("episode", &episode.to_string());
        }
        if let Some(duration_ms) = context.duration_ms {
            let duration = if duration_param == "duration" {
                (duration_ms / 1000).to_string()
            } else {
                duration_ms.to_string()
            };
            query.append_pair(duration_param, &duration);
        }
    }
    Some(url)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn introdb_effect<E: Env + 'static>(context: SkipSegmentContext) -> Option<Effect> {
    let season = context.season?;
    let episode = context.episode?;
    if !is_imdb_id(&context.item_id) {
        return None;
    }

    let mut url =
        Url::parse("https://api.introdb.app/segments").expect("static IntroDB URL must be valid");
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("imdb_id", &context.item_id);
        query.append_pair("season", &season.to_string());
        query.append_pair("episode", &episode.to_string());
    }
    let request = Request::builder()
        .method("GET")
        .uri(url.as_str())
        .body(())
        .expect("introdb request builder failed");
    let duration_ms = context.duration_ms;
    let result_context = context.clone();

    Some(
        EffectFuture::Concurrent(
            E::fetch::<_, IntroDbResponse>(request)
                .map(move |result| {
                    Msg::Internal(Internal::SkipSegmentsResult(
                        SkipSegmentSource::IntroDb,
                        result_context,
                        result.map(|response| introdb_candidates(&response, duration_ms)),
                    ))
                })
                .boxed_env(),
        )
        .into(),
    )
}

fn skipdb_url(context: &SkipSegmentContext) -> Option<Url> {
    let mut url = provider_url("https://api.skipdb.tv/api/segments", context, "duration")?;
    url.query_pairs_mut().append_pair("adjust", "conservative");
    Some(url)
}

#[cfg(not(test))]
fn skipdb_effect<E: Env + 'static>(context: SkipSegmentContext) -> Option<Effect> {
    let url = skipdb_url(&context)?;
    let request = Request::builder()
        .method("GET")
        .uri(url.as_str())
        .body(())
        .expect("skipdb request builder failed");
    let duration_ms = context.duration_ms;
    let result_context = context.clone();

    Some(
        EffectFuture::Concurrent(
            E::fetch::<_, SkipDbResponse>(request)
                .map(move |result| {
                    Msg::Internal(Internal::SkipSegmentsResult(
                        SkipSegmentSource::SkipDb,
                        result_context,
                        result.map(|response| skipdb_candidates(&response, duration_ms)),
                    ))
                })
                .boxed_env(),
        )
        .into(),
    )
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn theintrodb_effect<E: Env + 'static>(context: SkipSegmentContext) -> Option<Effect> {
    if !is_theintrodb_imdb_id(&context.item_id) {
        return None;
    }

    let url = provider_url(
        "https://api.theintrodb.org/v3/media",
        &context,
        "duration_ms",
    )?;
    let request = Request::builder()
        .method("GET")
        .uri(url.as_str())
        .body(())
        .expect("theintrodb request builder failed");
    let duration_ms = context.duration_ms;
    let result_context = context.clone();

    Some(
        EffectFuture::Concurrent(
            E::fetch::<_, TheIntroDbResponse>(request)
                .map(move |result| {
                    Msg::Internal(Internal::SkipSegmentsResult(
                        SkipSegmentSource::TheIntroDb,
                        result_context,
                        result.map(|response| theintrodb_candidates(&response, duration_ms)),
                    ))
                })
                .boxed_env(),
        )
        .into(),
    )
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn external_provider_effects<E: Env + 'static>(context: SkipSegmentContext) -> Vec<Effect> {
    [
        skipdb_effect::<E>(context.clone()),
        introdb_effect::<E>(context.clone()),
        theintrodb_effect::<E>(context),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(all(not(test), target_arch = "wasm32"))]
pub fn external_provider_effects<E: Env + 'static>(context: SkipSegmentContext) -> Vec<Effect> {
    skipdb_effect::<E>(context).into_iter().collect()
}

#[cfg(test)]
pub fn external_provider_effects<E: Env + 'static>(_context: SkipSegmentContext) -> Vec<Effect> {
    // Unit tests use a deliberately strict TestEnv that panics on unregistered network requests.
    // Provider parsing, URL semantics and resolution are tested directly below.
    Vec::new()
}

pub fn introdb_candidates_from_json(
    json: &str,
    media_duration_ms: Option<u64>,
) -> Result<Vec<SkipSegmentCandidate>, serde_json::Error> {
    serde_json::from_str::<IntroDbResponse>(json)
        .map(|response| introdb_candidates(&response, media_duration_ms))
}

fn match_rank(source_match: SkipSegmentMatch) -> u8 {
    match source_match {
        SkipSegmentMatch::ExactStream => 3,
        SkipSegmentMatch::ExactEpisode => 2,
        SkipSegmentMatch::DurationAdjusted => 1,
        SkipSegmentMatch::Estimated => 0,
    }
}

fn plausible_segment(candidate: &SkipSegmentCandidate) -> bool {
    if candidate.end_ms <= candidate.start_ms {
        return false;
    }

    let duration = candidate.end_ms - candidate.start_ms;
    match candidate.kind {
        SkipSegmentKind::Intro => (3_000..=180_000).contains(&duration),
        SkipSegmentKind::Recap => (3_000..=300_000).contains(&duration),
        SkipSegmentKind::Outro => duration >= 1_000,
        SkipSegmentKind::Preview => (1_000..=300_000).contains(&duration),
    }
}

fn trusted_standalone(candidate: &SkipSegmentCandidate) -> bool {
    if candidate.source == SkipSegmentSource::StremioNative {
        return true;
    }

    candidate
        .source_confidence
        .is_some_and(|confidence| confidence >= 0.80)
        || candidate.evidence_count >= 5
}

fn candidates_agree(left: &SkipSegmentCandidate, right: &SkipSegmentCandidate) -> bool {
    if left.kind != right.kind {
        return false;
    }

    let overlap_start = left.start_ms.max(right.start_ms);
    let overlap_end = left.end_ms.min(right.end_ms);
    if overlap_end <= overlap_start {
        return false;
    }

    let overlap = overlap_end - overlap_start;
    let shorter = (left.end_ms - left.start_ms).min(right.end_ms - right.start_ms);
    overlap.saturating_mul(100) >= shorter.saturating_mul(60)
        || (left.start_ms.abs_diff(right.start_ms) <= 5_000
            && left.end_ms.abs_diff(right.end_ms) <= 5_000)
}

fn materially_conflicting(
    best: &SkipSegmentCandidate,
    candidates: &[&SkipSegmentCandidate],
) -> bool {
    candidates.iter().any(|candidate| {
        candidate.source != best.source
            && candidate.stream_specificity == best.stream_specificity
            && match_rank(candidate.source_match) == match_rank(best.source_match)
            && !candidates_agree(best, candidate)
    })
}

fn candidate_cmp(left: &SkipSegmentCandidate, right: &SkipSegmentCandidate) -> Ordering {
    left.stream_specificity
        .cmp(&right.stream_specificity)
        .then_with(|| match_rank(left.source_match).cmp(&match_rank(right.source_match)))
        .then_with(|| right.adjusted.cmp(&left.adjusted))
        .then_with(|| left.evidence_count.cmp(&right.evidence_count))
        .then_with(|| {
            left.source_confidence
                .unwrap_or(0.0)
                .partial_cmp(&right.source_confidence.unwrap_or(0.0))
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| right.start_ms.cmp(&left.start_ms))
}

pub fn resolve_skip_segment(
    kind: SkipSegmentKind,
    candidates: &[SkipSegmentCandidate],
) -> Option<ResolvedSkipSegment> {
    let valid = candidates
        .iter()
        .filter(|candidate| candidate.kind == kind && plausible_segment(candidate))
        .collect::<Vec<_>>();

    let best = valid
        .iter()
        .copied()
        .max_by(|left, right| candidate_cmp(left, right))?;

    let agreeing_sources = valid
        .iter()
        .copied()
        .filter(|candidate| candidate.source != best.source && candidates_agree(best, candidate))
        .count();

    if !trusted_standalone(best) && agreeing_sources == 0 {
        return None;
    }

    let trusted_peers = valid
        .iter()
        .copied()
        .filter(|candidate| trusted_standalone(candidate))
        .collect::<Vec<_>>();
    if materially_conflicting(best, &trusted_peers) {
        return None;
    }

    let mut provenance = valid
        .iter()
        .copied()
        .filter(|candidate| candidates_agree(best, candidate))
        .map(|candidate| candidate.source)
        .collect::<Vec<SkipSegmentSource>>();

    provenance.sort_by_key(|source| match source {
        SkipSegmentSource::StremioNative => 0,
        SkipSegmentSource::SkipDb => 1,
        SkipSegmentSource::IntroDb => 2,
        SkipSegmentSource::TheIntroDb => 3,
    });
    provenance.dedup();

    Some(ResolvedSkipSegment {
        kind,
        from_ms: best.start_ms,
        to_ms: best.end_ms,
        confidence: best.source_confidence.unwrap_or_else(|| {
            if provenance.len() > 1 {
                0.95
            } else if best.evidence_count >= 20 {
                0.9
            } else if best.evidence_count >= 5 {
                0.8
            } else {
                0.7
            }
        }),
        provenance,
        adjusted: best.adjusted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        source: SkipSegmentSource,
        start_ms: u64,
        end_ms: u64,
        evidence_count: u32,
        specificity: SkipSegmentStreamSpecificity,
        adjusted: bool,
    ) -> SkipSegmentCandidate {
        SkipSegmentCandidate {
            kind: SkipSegmentKind::Intro,
            start_ms,
            end_ms,
            source,
            source_match: SkipSegmentMatch::ExactEpisode,
            source_confidence: None,
            adjusted,
            evidence_count,
            stream_specificity: specificity,
        }
    }

    #[test]
    fn skipdb_provider_url_uses_duration_seconds() {
        let context = SkipSegmentContext {
            item_id: "tt0903747".into(),
            media_type: "series".into(),
            season: Some(1),
            episode: Some(2),
            duration_ms: Some(2_820_500),
            open_subtitles_hash: None,
            stream_name_hash: None,
        };

        let url = skipdb_url(&context).expect("valid SkipDB URL");
        let query = url
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();

        assert_eq!(
            query.get("duration").map(|value| value.as_ref()),
            Some("2820")
        );
        assert_eq!(
            query.get("adjust").map(|value| value.as_ref()),
            Some("conservative")
        );
    }

    #[test]
    fn skipdb_adapter_fails_closed_for_out_of_range_matches() {
        let response: SkipDbResponse = serde_json::from_value(serde_json::json!({
            "segments": {
                "intro": {
                    "start_ms": 229_500,
                    "end_ms": 246_500,
                    "adjusted": false,
                    "match": "out-of-range",
                    "confidence": 0.6
                },
                "recap": null,
                "outro": null,
                "preview": null
            }
        }))
        .expect("valid SkipDB response");

        let candidates = skipdb_candidates(&response, Some(300_000));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].source, SkipSegmentSource::SkipDb);
        assert_eq!(candidates[0].source_match, SkipSegmentMatch::Estimated);
        assert!(resolve_skip_segment(SkipSegmentKind::Intro, &candidates).is_none());
    }

    #[test]
    fn skipdb_adapter_rejects_segment_beyond_media_duration() {
        let response: SkipDbResponse = serde_json::from_value(serde_json::json!({
            "segments": {
                "intro": {
                    "start_ms": 90_000,
                    "end_ms": 130_000,
                    "adjusted": false,
                    "match": "exact",
                    "confidence": 0.95
                }
            }
        }))
        .expect("valid SkipDB response");

        assert!(skipdb_candidates(&response, Some(120_000)).is_empty());
    }

    #[test]
    fn skipdb_adapter_accepts_strong_exact_match() {
        let response: SkipDbResponse = serde_json::from_value(serde_json::json!({
            "segments": {
                "intro": {
                    "start_ms": 61_000,
                    "end_ms": 91_000,
                    "adjusted": false,
                    "match": "exact",
                    "confidence": 0.9
                },
                "recap": null,
                "outro": null,
                "preview": null
            }
        }))
        .expect("valid SkipDB response");

        let candidates = skipdb_candidates(&response, Some(300_000));
        let resolved = resolve_skip_segment(SkipSegmentKind::Intro, &candidates)
            .expect("trusted SkipDB intro");
        assert_eq!(resolved.from_ms, 61_000);
        assert_eq!(resolved.to_ms, 91_000);
    }

    #[test]
    fn theintrodb_provider_url_uses_duration_milliseconds() {
        let context = SkipSegmentContext {
            item_id: "tt0903747".into(),
            media_type: "series".into(),
            season: Some(1),
            episode: Some(2),
            duration_ms: Some(2_820_500),
            open_subtitles_hash: None,
            stream_name_hash: None,
        };

        let url = provider_url(
            "https://api.theintrodb.org/v3/media",
            &context,
            "duration_ms",
        )
        .expect("valid TheIntroDB URL");
        let query = url
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(
            query.get("duration_ms").map(|value| value.as_ref()),
            Some("2820500")
        );
    }

    #[test]
    fn theintrodb_imdb_validation_matches_its_public_contract() {
        assert!(is_theintrodb_imdb_id("tt0903747"));
        assert!(is_theintrodb_imdb_id("tt12345678"));
        assert!(!is_theintrodb_imdb_id("tt123456"));
        assert!(!is_theintrodb_imdb_id("tt123456789"));
    }

    #[test]
    fn native_adapter_selects_strongest_intro_and_scales_to_target_duration() {
        use std::collections::HashMap;

        let response = SkipGapsResponse {
            accuracy: "byEpisode".into(),
            gaps: HashMap::from([(
                1_000_000,
                SkipGaps {
                    seek_history: vec![
                        SeekEvent {
                            records: 3,
                            from: 20_000,
                            to: 60_000,
                        },
                        SeekEvent {
                            records: 25,
                            from: 100_000,
                            to: 160_000,
                        },
                    ],
                    outro: Some(900_000),
                },
            )]),
        };

        let candidates = stremio_native_candidates(&response, 1_100_000);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].kind, SkipSegmentKind::Intro);
        assert_eq!(candidates[0].start_ms, 110_000);
        assert_eq!(candidates[0].end_ms, 176_000);
        assert_eq!(candidates[0].evidence_count, 25);
        assert!(candidates[0].adjusted);
        assert_eq!(candidates[1].kind, SkipSegmentKind::Outro);
        assert_eq!(candidates[1].start_ms, 990_000);
        assert_eq!(candidates[1].end_ms, 1_100_000);
    }

    #[test]
    fn introdb_adapter_preserves_confidence_and_submission_count() {
        let response: IntroDbResponse = serde_json::from_value(serde_json::json!({
            "imdb_id": "tt0903747",
            "season": 1,
            "episode": 2,
            "intro": {
                "start_ms": 61_000,
                "end_ms": 91_000,
                "confidence": 0.92,
                "submission_count": 12
            },
            "recap": null,
            "outro": null
        }))
        .expect("valid IntroDB response");

        let candidates = introdb_candidates(&response, Some(120_000));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].source, SkipSegmentSource::IntroDb);
        assert_eq!(candidates[0].source_match, SkipSegmentMatch::ExactEpisode);
        assert_eq!(candidates[0].source_confidence, Some(0.92));
        assert_eq!(candidates[0].evidence_count, 12);
        assert!(!candidates[0].adjusted);
    }

    #[test]
    fn introdb_adapter_rejects_segment_beyond_media_duration() {
        let response: IntroDbResponse = serde_json::from_value(serde_json::json!({
            "intro": {
                "start_ms": 90_000,
                "end_ms": 130_000
            }
        }))
        .expect("valid IntroDB response");

        assert!(introdb_candidates(&response, Some(120_000)).is_empty());
    }

    #[test]
    fn theintrodb_adapter_handles_open_ended_credits() {
        let response: TheIntroDbResponse = serde_json::from_value(serde_json::json!({
            "tmdb_id": 123,
            "type": "tv",
            "credits": [{
                "start_ms": 1_800_000,
                "end_ms": null,
                "confidence": 0.8,
                "submission_count": 7
            }]
        }))
        .expect("valid TheIntroDB response");

        let candidates = theintrodb_candidates(&response, Some(1_900_000));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, SkipSegmentKind::Outro);
        assert_eq!(candidates[0].start_ms, 1_800_000);
        assert_eq!(candidates[0].end_ms, 1_900_000);
        assert_eq!(candidates[0].evidence_count, 7);
    }

    #[test]
    fn resolver_prefers_more_specific_stream_evidence() {
        let candidates = vec![
            candidate(
                SkipSegmentSource::IntroDb,
                90_000,
                150_000,
                100,
                SkipSegmentStreamSpecificity::Episode,
                false,
            ),
            candidate(
                SkipSegmentSource::StremioNative,
                92_000,
                152_000,
                10,
                SkipSegmentStreamSpecificity::OpenSubtitlesHash,
                false,
            ),
        ];

        let resolved =
            resolve_skip_segment(SkipSegmentKind::Intro, &candidates).expect("resolved segment");

        assert_eq!(resolved.from_ms, 92_000);
        assert_eq!(resolved.to_ms, 152_000);
        assert_eq!(resolved.provenance.len(), 2);
    }

    #[test]
    fn resolver_prefers_unadjusted_then_stronger_evidence() {
        let candidates = vec![
            candidate(
                SkipSegmentSource::StremioNative,
                90_000,
                150_000,
                100,
                SkipSegmentStreamSpecificity::Duration,
                true,
            ),
            candidate(
                SkipSegmentSource::IntroDb,
                91_000,
                151_000,
                5,
                SkipSegmentStreamSpecificity::Duration,
                false,
            ),
        ];

        let resolved =
            resolve_skip_segment(SkipSegmentKind::Intro, &candidates).expect("resolved segment");

        assert_eq!(resolved.from_ms, 91_000);
        assert!(!resolved.adjusted);
    }

    #[test]
    fn resolver_rejects_weak_external_standalone_candidate() {
        let candidates = vec![candidate(
            SkipSegmentSource::IntroDb,
            60_000,
            90_000,
            1,
            SkipSegmentStreamSpecificity::Episode,
            false,
        )];

        assert!(resolve_skip_segment(SkipSegmentKind::Intro, &candidates).is_none());
    }

    #[test]
    fn resolver_accepts_independent_agreement_without_standalone_strength() {
        let candidates = vec![
            candidate(
                SkipSegmentSource::IntroDb,
                60_000,
                90_000,
                1,
                SkipSegmentStreamSpecificity::Episode,
                false,
            ),
            candidate(
                SkipSegmentSource::TheIntroDb,
                61_000,
                91_000,
                1,
                SkipSegmentStreamSpecificity::Episode,
                false,
            ),
        ];

        let resolved =
            resolve_skip_segment(SkipSegmentKind::Intro, &candidates).expect("agreed segment");
        assert_eq!(resolved.provenance.len(), 2);
    }

    #[test]
    fn resolver_suppresses_material_conflict_between_trusted_peers() {
        let candidates = vec![
            candidate(
                SkipSegmentSource::IntroDb,
                60_000,
                90_000,
                10,
                SkipSegmentStreamSpecificity::Episode,
                false,
            ),
            candidate(
                SkipSegmentSource::TheIntroDb,
                120_000,
                155_000,
                10,
                SkipSegmentStreamSpecificity::Episode,
                false,
            ),
        ];

        assert!(resolve_skip_segment(SkipSegmentKind::Intro, &candidates).is_none());
    }

    #[test]
    fn resolver_rejects_implausibly_long_intro() {
        let candidates = vec![candidate(
            SkipSegmentSource::IntroDb,
            10_000,
            250_000,
            20,
            SkipSegmentStreamSpecificity::Episode,
            false,
        )];

        assert!(resolve_skip_segment(SkipSegmentKind::Intro, &candidates).is_none());
    }

    #[test]
    fn resolver_rejects_invalid_or_tiny_segments() {
        let candidates = vec![
            candidate(
                SkipSegmentSource::IntroDb,
                10_000,
                9_000,
                10,
                SkipSegmentStreamSpecificity::Episode,
                false,
            ),
            candidate(
                SkipSegmentSource::TheIntroDb,
                10_000,
                10_500,
                10,
                SkipSegmentStreamSpecificity::Episode,
                false,
            ),
        ];

        assert!(resolve_skip_segment(SkipSegmentKind::Intro, &candidates).is_none());
    }
}
