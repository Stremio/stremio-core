use crate::types::resource::StreamSource;
use serde_test::{assert_de_tokens, assert_ser_tokens, Token};
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
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
            },
            StreamSource::External {
                external_url: Url::parse("https://external_url").unwrap(),
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
                len: 2,
            },
            Token::Str("infoHash"),
            Token::Str("0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::Some,
            Token::U16(1),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 2,
            },
            Token::Str("infoHash"),
            Token::Str("0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::None,
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
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
            },
            StreamSource::Torrent {
                info_hash: [1; 20],
                file_idx: None,
            },
            StreamSource::External {
                external_url: Url::parse("https://external_url").unwrap(),
            },
            StreamSource::PlayerFrame {
                player_frame_url: Url::parse("https://player_frame_url").unwrap(),
            },
        ],
        &[
            Token::Seq { len: Some(7) },
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
                len: 2,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::Some,
            Token::U16(1),
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 2,
            },
            Token::Str("infoHash"),
            Token::BorrowedBytes(b"0101010101010101010101010101010101010101"),
            Token::Str("fileIdx"),
            Token::None,
            Token::StructEnd,
            Token::Struct {
                name: "StreamSource",
                len: 2,
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
}
