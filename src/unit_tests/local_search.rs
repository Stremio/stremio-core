use assert_matches::assert_matches;

use crate::{
    constants::CINEMETA_CATALOGS_URL,
    models::{
        ctx::Ctx,
        local_search::{ImdbRating, LocalSearch, Searchable},
    },
    runtime::{
        msg::{Internal, Msg},
        UpdateWithCtx,
    },
    unit_tests::TestEnv,
};

#[test]
fn local_search_scores_a_zero_rated_title_with_no_positive_ratings() {
    let mut model = LocalSearch::default();
    let title = Searchable {
        id: "tt1".to_owned(),
        name: "Nebula".to_owned(),
        r#type: "movie".to_owned(),
        poster: None,
        imdb_rating: Some(ImdbRating::try_from(0.0).unwrap()),
        popularity: Some(10),
        release_info: None,
    };

    <LocalSearch as UpdateWithCtx<TestEnv>>::update(
        &mut model,
        &Msg::Internal(Internal::LoadLocalSearchResult(
            CINEMETA_CATALOGS_URL.clone(),
            Ok(vec![title]),
        )),
        &Ctx::default(),
    );

    let matches = model.searcher.as_ref().unwrap().search("Nebula", 5);
    assert_matches!(matches.as_slice(), [(title, score)] if title.id == "tt1" && score.is_finite());
}
