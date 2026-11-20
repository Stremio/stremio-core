use std::marker::PhantomData;

use crate::{
    models::{common::Loadable, ctx::Ctx, player::Player},
    runtime::{
        msg::{Internal, Msg},
        UpdateWithCtx,
    },
    types::{
        api::{SkipGaps, SkipGapsRequest, SkipGapsResponse},
        library::LibraryItem,
        profile::AuthKey,
        resource::{MetaItemPreview, SeriesInfo},
    },
    unit_tests::TestEnv,
};

#[test]
fn delayed_skip_gaps_results_do_not_replace_the_current_episode() {
    let _guard = TestEnv::reset().unwrap();
    let request = |episode| SkipGapsRequest {
        auth_key: AuthKey("test-key".to_owned()),
        os_hash: format!("file-{episode}"),
        item_id: "tt123456".to_owned(),
        series_info: SeriesInfo { season: 1, episode },
        stream_name_hash: "stream".to_owned(),
        duration: 3_600_000,
    };
    let previous = request(1);
    let current = request(2);
    let duration = current.duration;
    let result = |outro| SkipGapsResponse {
        accuracy: "hash".to_owned(),
        gaps: [(
            duration,
            SkipGaps {
                seek_history: vec![],
                outro: Some(outro),
            },
        )]
        .into(),
    };
    let mut player = Player {
        skip_gaps: Some((current.clone(), Loadable::Loading)),
        library_item: Some(LibraryItem::from((
            &MetaItemPreview::default(),
            PhantomData::<TestEnv>,
        ))),
        ..Default::default()
    };
    let ctx = Ctx::default();
    let stale_result = Msg::Internal(Internal::SkipGapsResult(previous, Ok(result(3_200_000))));

    <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &stale_result, &ctx);
    assert_eq!(player.skip_gaps, Some((current.clone(), Loadable::Loading)));
    assert!(player.intro_outro.is_none());

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Internal(Internal::SkipGapsResult(
            current.clone(),
            Ok(result(3_400_000)),
        )),
        &ctx,
    );
    <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &stale_result, &ctx);
    assert_eq!(
        player.skip_gaps,
        Some((current, Loadable::Ready(result(3_400_000))))
    );
    assert_eq!(
        player
            .intro_outro
            .as_ref()
            .and_then(|timings| timings.outro),
        Some(3_400_000)
    );
}
