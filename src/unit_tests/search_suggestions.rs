use crate::{
    constants::CINEMETA_CATALOGS_URL,
    models::{
        ctx::Ctx,
        local_search::{LocalSearch, Searchable},
    },
    runtime::{
        msg::{Action, ActionSearch, Internal, Msg},
        UpdateWithCtx,
    },
    unit_tests::TestEnv,
};

#[test]
fn magnet_search_clears_suggestions_from_the_previous_query() {
    let mut local_search = LocalSearch::default();
    let ctx = Ctx::default();
    let update = |local_search: &mut LocalSearch, msg| {
        <LocalSearch as UpdateWithCtx<TestEnv>>::update(local_search, &msg, &ctx)
    };
    let search = |query: &str| {
        Msg::Action(Action::Search(ActionSearch::Search {
            search_query: query.to_owned(),
            max_results: 5,
        }))
    };
    let title = Searchable {
        id: "tt1".to_owned(),
        name: "Nebula".to_owned(),
        r#type: "movie".to_owned(),
        poster: None,
        imdb_rating: None,
        popularity: None,
        release_info: None,
    };

    update(
        &mut local_search,
        Msg::Internal(Internal::LoadLocalSearchResult(
            CINEMETA_CATALOGS_URL.clone(),
            Ok(vec![title.clone()]),
        )),
    );
    update(&mut local_search, search("Nebula"));
    assert_eq!(local_search.search_results, vec![title.clone()]);

    let effects = update(
        &mut local_search,
        search("magnet:?xt=urn:btih:0000000000000000000000000000000000000000"),
    );
    assert!(local_search.search_results.is_empty());
    assert!(effects.has_changed);

    update(&mut local_search, search("Nebula"));
    assert_eq!(local_search.search_results, vec![title]);
}
