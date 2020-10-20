use crate::types::resource::MetaItemBehaviorHints;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn meta_item_behavior_hints() {
    assert_tokens(
        &vec![
            MetaItemBehaviorHints {
                default_video_id: Some("default_video_id".to_owned()),
                featured_video_id: Some("featured_video_id".to_owned()),
                has_scheduled_videos: true,
            },
            MetaItemBehaviorHints {
                default_video_id: None,
                featured_video_id: None,
                has_scheduled_videos: false,
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "MetaItemBehaviorHints",
                len: 3,
            },
            Token::Str("defaultVideoId"),
            Token::Some,
            Token::Str("default_video_id"),
            Token::Str("featuredVideoId"),
            Token::Some,
            Token::Str("featured_video_id"),
            Token::Str("hasScheduledVideos"),
            Token::Bool(true),
            Token::StructEnd,
            Token::Struct {
                name: "MetaItemBehaviorHints",
                len: 3,
            },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::Str("featuredVideoId"),
            Token::None,
            Token::Str("hasScheduledVideos"),
            Token::Bool(false),
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &MetaItemBehaviorHints {
            default_video_id: None,
            featured_video_id: None,
            has_scheduled_videos: false,
        },
        &[
            Token::Struct {
                name: "MetaItemBehaviorHints",
                len: 2,
            },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::Str("featuredVideoId"),
            Token::None,
            Token::StructEnd,
        ],
    );
}
