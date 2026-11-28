use chrono::{Duration, TimeZone, Utc};

use crate::{
    constants::LIBRARY_RECENT_COUNT,
    models::library_with_filters::Sort,
    types::library::{LibraryBucket, LibraryItem, LibraryItemState},
};

fn bucket(mtime_offsets: &[(&str, i64)]) -> LibraryBucket {
    let base = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let items = mtime_offsets
        .iter()
        .map(|(id, offset)| LibraryItem {
            id: id.to_string(),
            name: id.to_string(),
            r#type: "movie".to_owned(),
            poster: None,
            poster_shape: Default::default(),
            removed: false,
            temp: false,
            ctime: None,
            mtime: base + Duration::seconds(*offset),
            state: LibraryItemState::default(),
            behavior_hints: Default::default(),
        })
        .collect();
    LibraryBucket::new(Some("uid".into()), items)
}

fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

#[test]
fn are_ids_in_recent_boundary() {
    let names = (0..=LIBRARY_RECENT_COUNT)
        .map(|index| index.to_string())
        .collect::<Vec<_>>();
    let library = bucket(
        &names
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index as i64))
            .collect::<Vec<_>>(),
    );
    let newest = LIBRARY_RECENT_COUNT.to_string();
    assert!(library.are_ids_in_recent(&[]));
    assert!(library.are_ids_in_recent(&ids(&[&newest])));
    assert!(library.are_ids_in_recent(&ids(&[&newest, "1"])));
    assert!(!library.are_ids_in_recent(&ids(&["0"])));
    assert!(!library.are_ids_in_recent(&ids(&[&newest, "0"])));
    assert!(!library.are_ids_in_recent(&ids(&["missing"])));
    assert!(!library.are_ids_in_recent(&ids(&[&newest, "missing"])));
}

#[test]
fn are_ids_in_recent_ties_at_boundary() {
    let names = (0..LIBRARY_RECENT_COUNT - 1)
        .map(|index| index.to_string())
        .collect::<Vec<_>>();
    let mut items = names
        .iter()
        .map(|name| (name.as_str(), 10))
        .collect::<Vec<_>>();
    items.extend([("tie_a", 5), ("tie_b", 5), ("old", 0)]);
    let library = bucket(&items);
    assert!(library.are_ids_in_recent(&ids(&["0"])));
    assert!(library.are_ids_in_recent(&ids(&["0", "1"])));
    assert!(!library.are_ids_in_recent(&ids(&["tie_a"])));
    assert!(!library.are_ids_in_recent(&ids(&["0", "tie_b"])));

    let (recent_items, other_items) = library.split_items_by_recent();
    assert_eq!(recent_items.len(), LIBRARY_RECENT_COUNT);
    assert_eq!(other_items.len(), 2);
    assert!(recent_items
        .iter()
        .all(|recent| other_items.iter().all(|other| recent.mtime >= other.mtime)));
    assert_eq!(library.recent_items().len(), LIBRARY_RECENT_COUNT);
}

#[test]
fn sort_by_name_matches_lowercase_comparison() {
    let names = [
        "abc",
        "ABD",
        "Abc",
        "a",
        "",
        "Émile",
        "élan",
        "Zeta",
        "ΟΔΟΣ",
        "οδοσ",
        "οδος",
        "İstanbul",
        "istanbul",
    ];
    let item = |name: &str| LibraryItem {
        id: "id".to_owned(),
        name: name.to_owned(),
        r#type: "movie".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: Utc::now(),
        state: Default::default(),
        behavior_hints: Default::default(),
    };
    for a in names {
        for b in names {
            assert_eq!(
                Sort::Name.sort_items(&item(a), &item(b)),
                a.to_lowercase().cmp(&b.to_lowercase()),
            );
            assert_eq!(
                Sort::NameReverse.sort_items(&item(a), &item(b)),
                b.to_lowercase().cmp(&a.to_lowercase()),
            );
        }
    }
}
