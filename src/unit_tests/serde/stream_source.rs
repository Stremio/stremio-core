use crate::types::resource::StreamSource;
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, Token};
use url::Url;

#[test]
fn stream_source_from_json() {
    assert_matches::assert_matches!(
        serde_json::from_str::<StreamSource>(
            "{\"infoHash\": \"0101010101010101010101010101010101010101\"}",
        ),
        Ok(_)
    );

    assert_matches::assert_matches!(
        serde_json::from_value::<StreamSource>(serde_json::json!(
            { "infoHash": "0101010101010101010101010101010101010101" }
        )),
        Ok(_)
    );
}

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
                file_must_include: vec![],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
                announce: vec![],
                file_must_include: vec![],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: Some(2),
                announce: vec![],
                file_must_include: vec!["text".into()],
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
            Token::Seq { len: Some(7) },
            // 1st
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::StructEnd,
            // 2nd
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
            // 3rd
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
            // 4th
            Token::Struct {
                name: "StreamSource",
                len: 4,
            },
            Token::Str("infoHash"),
            Token::Str("0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::Some,
            Token::U16(2),
            Token::Str("announce"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Str("fileMustInclude"),
            Token::Seq { len: Some(1) },
            Token::Str("text"),
            Token::SeqEnd,
            Token::StructEnd,
            // 5th
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("externalUrl"),
            Token::Some,
            Token::Str("https://external_url/"),
            Token::StructEnd,
            // 6th
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
                file_must_include: vec![],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: Some(1),
                announce: vec!["announce".to_owned()],
                file_must_include: vec![],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
                announce: vec![],
                file_must_include: vec![],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
                announce: vec![],
                file_must_include: vec![],
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: Some(2),
                announce: vec![],
                file_must_include: vec!["text".into()],
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
            Token::Seq { len: Some(9) },
            // Youtube
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
            // 1st Torrent
            Token::Struct {
                name: "StreamSource",
                len: 4,
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
            // 2nd Torrent
            Token::Struct {
                name: "StreamSource",
                len: 4,
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
            // 3rd Torrent
            Token::Struct {
                name: "StreamSource",
                len: 4,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::None,
            Token::Str("announce"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            // 4th Torrent
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::StructEnd,
            // 5th Torrent
            Token::Struct {
                name: "StreamSource",
                len: 4,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::Some,
            Token::U16(2),
            Token::Str("announce"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Str("fileMustInclude"),
            Token::Seq { len: Some(0) },
            Token::Str("text"),
            Token::SeqEnd,
            Token::StructEnd,
            // External Url
            Token::Struct {
                name: "StreamSource",
                len: 1,
            },
            Token::Str("externalUrl"),
            Token::Str("https://external_url/"),
            Token::StructEnd,
            // Player Frame
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
        "Valid StreamSource",
    );
}
