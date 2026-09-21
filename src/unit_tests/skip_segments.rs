use std::collections::HashMap;

use crate::models::skip_segments::{
    introdb_candidates_from_json, resolve_skip_segment, stremio_native_candidates,
};
use crate::types::{
    api::{SeekEvent, SkipGaps, SkipGapsResponse},
    skip_segments::{
        SkipSegmentCandidate, SkipSegmentKind, SkipSegmentMatch, SkipSegmentSource,
        SkipSegmentStreamSpecificity,
    },
};

#[test]
fn skip_segments_native_uses_strongest_valid_intro_seek() {
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
                outro: None,
            },
        )]),
    };

    let candidates = stremio_native_candidates(&response, 1_100_000);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].start_ms, 110_000);
    assert_eq!(candidates[0].end_ms, 176_000);
    assert_eq!(candidates[0].evidence_count, 25);
}

#[test]
fn skip_segments_introdb_json_maps_to_common_candidate() {
    let candidates = introdb_candidates_from_json(
        r#"{
            "imdb_id":"tt0903747",
            "season":1,
            "episode":2,
            "intro":{
                "start_ms":61000,
                "end_ms":91000,
                "confidence":0.92,
                "submission_count":12
            },
            "recap":null,
            "outro":null
        }"#,
        Some(120_000),
    )
    .expect("valid IntroDB response");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].source, SkipSegmentSource::IntroDb);
    assert_eq!(candidates[0].source_confidence, Some(0.92));
    assert_eq!(candidates[0].evidence_count, 12);
}

#[test]
fn skip_segments_independent_agreement_can_resolve_weak_evidence() {
    let candidates = vec![
        SkipSegmentCandidate {
            kind: SkipSegmentKind::Intro,
            start_ms: 60_000,
            end_ms: 90_000,
            source: SkipSegmentSource::IntroDb,
            source_match: SkipSegmentMatch::ExactEpisode,
            source_confidence: None,
            adjusted: false,
            evidence_count: 1,
            stream_specificity: SkipSegmentStreamSpecificity::Episode,
        },
        SkipSegmentCandidate {
            kind: SkipSegmentKind::Intro,
            start_ms: 61_000,
            end_ms: 91_000,
            source: SkipSegmentSource::TheIntroDb,
            source_match: SkipSegmentMatch::ExactEpisode,
            source_confidence: None,
            adjusted: false,
            evidence_count: 1,
            stream_specificity: SkipSegmentStreamSpecificity::Episode,
        },
    ];

    let resolved =
        resolve_skip_segment(SkipSegmentKind::Intro, &candidates).expect("agreed segment");

    assert_eq!(resolved.provenance.len(), 2);
}

#[test]
fn skip_segments_conflicting_trusted_external_evidence_fails_closed() {
    let candidates = vec![
        SkipSegmentCandidate {
            kind: SkipSegmentKind::Intro,
            start_ms: 60_000,
            end_ms: 90_000,
            source: SkipSegmentSource::IntroDb,
            source_match: SkipSegmentMatch::ExactEpisode,
            source_confidence: None,
            adjusted: false,
            evidence_count: 10,
            stream_specificity: SkipSegmentStreamSpecificity::Episode,
        },
        SkipSegmentCandidate {
            kind: SkipSegmentKind::Intro,
            start_ms: 120_000,
            end_ms: 155_000,
            source: SkipSegmentSource::TheIntroDb,
            source_match: SkipSegmentMatch::ExactEpisode,
            source_confidence: None,
            adjusted: false,
            evidence_count: 10,
            stream_specificity: SkipSegmentStreamSpecificity::Episode,
        },
    ];

    assert!(resolve_skip_segment(SkipSegmentKind::Intro, &candidates).is_none());
}
