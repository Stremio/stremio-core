use crate::types::library::{LibraryItem, LibraryItemBehaviorHints, LibraryItemState};
use crate::types::resource::PosterShape;
use chrono::prelude::TimeZone;
use chrono::Utc;

#[test]
fn deserialize_library_item_empty_string_as_none() {
    let library_item = LibraryItem {
        id: "id1".to_owned(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        state: LibraryItemState::default(),
        name: "name".to_owned(),
        type_name: "type_name".to_owned(),
        poster: None,
        poster_shape: PosterShape::Poster,
        behavior_hints: LibraryItemBehaviorHints::default(),
    };
    let library_item_json = format!(
        r#"
            {{
                "_id": "id1",
                "name": "name",
                "type": "type_name",
                "poster": "",
                "posterShape": "poster",
                "removed": false,
                "temp": false,
                "_ctime": "",
                "_mtime": "2020-01-01T00:00:00Z",
                "state": {},
                "behaviorHints": {}
            }}
        "#,
        serde_json::to_string(&LibraryItemState::default()).unwrap(),
        serde_json::to_string(&LibraryItemBehaviorHints::default()).unwrap(),
    );
    let library_item_deserialize = serde_json::from_str(&library_item_json).unwrap();
    assert_eq!(
        library_item, library_item_deserialize,
        "LibraryItem deserialized successfully"
    );
}
