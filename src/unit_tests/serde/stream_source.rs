use crate::types::resource::StreamSource;
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, Token};
use url::Url;

#[test]
fn stream_source() {
    assert_ser_tokens(
        &vec![
            StreamSource::Url {
                url: Url::parse("https://url").unwrap(),
            },
            StreamSource::YouTube {
                yt_id: "yt_id".to_owned(),
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: Some(1),
                announce: vec!["announce".to_owned()],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
                announce: vec![],
            },
            StreamSource::External {
                external_url: Some(Url::parse("https://external_url").unwrap()),
                android_tv_url: None,
                tizen_url: None,
                webos_url: None,
            },
            StreamSource::PlayerFrame {
                player_frame_url: Url::parse("https://player_frame_url").unwrap(),
            },
        ],
        &[
            Token::Seq { len: Some(6) },
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("ytId"),
            Token::Str("yt_id"),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 3,
            },
            Token::Str("infoHash"),
            Token::Str("0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::Some,
            Token::U16(1),
            Token::Str("announce"),
            Token::Seq { len: Some(1) },
            Token::Str("announce"),
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 3,
            },
            Token::Str("infoHash"),
            Token::Str("0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::None,
            Token::Str("announce"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("externalUrl"),
            Token::Some,
            Token::Str("https://external_url/"),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("playerFrameUrl"),
            Token::Str("https://player_frame_url/"),
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &vec![
            StreamSource::Url {
                url: Url::parse("https://url").unwrap(),
            },
            StreamSource::YouTube {
                yt_id: "yt_id".to_owned(),
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: Some(1),
                announce: vec!["announce".to_owned()],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: Some(1),
                announce: vec!["announce".to_owned()],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
                announce: vec![],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
                announce: vec![],
            },
            StreamSource::External {
                external_url: Some(Url::parse("https://external_url").unwrap()),
                android_tv_url: None,
                tizen_url: None,
                webos_url: None,
            },
            StreamSource::PlayerFrame {
                player_frame_url: Url::parse("https://player_frame_url").unwrap(),
            },
        ],
        &[
            Token::Seq { len: Some(8) },
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("ytId"),
            Token::Str("yt_id"),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 3,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::Some,
            Token::U16(1),
            Token::Str("announce"),
            Token::Seq { len: Some(1) },
            Token::Str("announce"),
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 3,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::Some,
            Token::U16(1),
            Token::Str("sources"),
            Token::Seq { len: Some(1) },
            Token::Str("announce"),
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 3,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::None,
            Token::Str("announce"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("externalUrl"),
            Token::Str("https://external_url/"),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("playerFrameUrl"),
            Token::Str("https://player_frame_url/"),
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens_error::<StreamSource>(
        &[
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("externalUrl"),
            Token::None,
            Token::StructEnd,
        ],
        "data did not match any variant of untagged enum StreamSource",
    );
}
