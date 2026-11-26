use std::marker::PhantomData;

use crate::{
    models::{
        continue_watching_preview::ContinueWatchingPreview,
        library_with_filters::{ContinueWatchingFilter, LibraryFilter},
    },
    types::{
        library::{LibraryBucket, LibraryItem},
        notifications::NotificationsBucket,
        resource::MetaItemPreview,
    },
    unit_tests::TestEnv,
};

#[test]
fn continue_watching_hides_movies_at_the_completion_threshold() {
    let _guard = TestEnv::reset().unwrap();
    for (label, content_type, offset, watched, duration, visible) in [
        ("completed movie", "movie", 990, 990, 1000, false),
        ("at the threshold", "movie", 950, 950, 1000, false),
        ("below the threshold", "movie", 949, 949, 1000, true),
        ("fractional threshold", "movie", 950, 950, 1001, true),
        ("seeked ahead", "movie", 950, 100, 1000, true),
        ("early rewatch", "movie", 100, 1000, 1000, true),
        ("unknown duration", "movie", 950, 950, 0, true),
        ("series episode", "series", 950, 950, 1000, true),
    ] {
        let preview = MetaItemPreview {
            id: "tt1".to_owned(),
            r#type: content_type.to_owned(),
            ..Default::default()
        };
        let mut item = LibraryItem::from((&preview, PhantomData::<TestEnv>));
        item.state.time_offset = offset;
        item.state.time_watched = watched;
        item.state.duration = duration;
        let notifications = NotificationsBucket::default();

        assert_eq!(
            ContinueWatchingFilter::predicate(&item, &notifications),
            visible,
            "{label}"
        );
        let library = LibraryBucket::new(None, vec![item]);
        let (preview, _) = ContinueWatchingPreview::new(&library, &notifications);
        assert_eq!(!preview.items.is_empty(), visible, "{label}");
    }
}
