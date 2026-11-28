use chrono::{TimeZone, Utc};
use serde_test::{assert_ser_tokens, Token};

use crate::types::library::{LibraryBucket, LibraryBucketRef, LibraryBucketSliceRef, LibraryItem};

#[test]
fn library_bucket_ref() {
    assert_ser_tokens(
        &LibraryBucketRef {
            uid: &Some("uid".into()),
            items: [].iter().cloned().collect(),
        },
        &[
            Token::Struct {
                name: "LibraryBucketRef",
                len: 2,
            },
            Token::Str("uid"),
            Token::Some,
            Token::Str("uid"),
            Token::Str("items"),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn library_bucket_slice_ref_matches_library_bucket_ref() {
    let mtime = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let library = LibraryBucket::new(
        Some("uid".into()),
        ["a", "b", "c"]
            .into_iter()
            .map(|id| LibraryItem {
                id: id.to_owned(),
                name: id.to_owned(),
                r#type: "movie".to_owned(),
                poster: None,
                poster_shape: Default::default(),
                removed: false,
                temp: false,
                ctime: None,
                mtime,
                state: Default::default(),
                behavior_hints: Default::default(),
            })
            .collect(),
    );
    let items = library.items.values().collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(LibraryBucketSliceRef {
            uid: &library.uid,
            items: &items,
        })
        .unwrap(),
        serde_json::to_value(LibraryBucketRef::new(&library.uid, &items)).unwrap(),
    );
}
