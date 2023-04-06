//! Local autocompletion search
//!

use enclose::enclose;
use futures::FutureExt;
use http::Request;
use num::{rational::Ratio, ToPrimitive};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError, NoneAsEmptyString};
use url::Url;

use localsearch::{self, LocalSearch as Searcher, Score, DEFAULT_SCORE_THRESHOLD};

use crate::{
    constants::{CINEMETA_FEED_CATALOG_ID, CINEMETA_URL},
    models::{
        common::{eq_update, Loadable},
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionSearch, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvError, EnvFutureExt, UpdateWithCtx,
    },
    types::NumberAsString,
};

pub use imdb_rating::*;

/// The response returned when fetching the searchable items list.
///
/// Currently this is fetched from Cinemeta's `feed.json`
#[derive(Deserialize, Serialize, Clone)]
#[serde(transparent)]
pub struct SearchableItemsResponse(pub Vec<Searchable>);

#[derive(Copy, Clone)]
pub struct IndexOptions {
    imdb_rating_weight: f64,
    popularity_weight: f64,
}

/// A searchable item
#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Searchable {
    pub id: String,
    pub name: String,
    pub r#type: String,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError<NoneAsEmptyString>")]
    pub poster: Option<Url>,
    pub imdb_rating: Option<ImdbRating>,
    pub popularity: Option<u64>,
    pub release_info: Option<String>,
}

#[derive(Default, Clone, Serialize, Debug)]
// #[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalSearch {
    /// The Searchable items that will be used for the local search.
    ///
    pub current_records: Vec<Searchable>,
    pub searcher: Option<Searcher<Searchable>>,
    /// A loadable resource in order to be able to search for items while
    /// a new set of items is being loaded (i.e. refreshed)
    pub latest_records: Loadable<Vec<Searchable>, EnvError>,
}

impl LocalSearch {
    pub fn init<E: Env + 'static>() -> (Self, Effects) {
        let effects = Effects::one(Self::get_searchable_items::<E>(&CINEMETA_URL));

        (
            Self {
                current_records: vec![],
                searcher: None,
                latest_records: Loadable::Loading,
            },
            effects.unchanged(),
        )
    }

    fn get_searchable_items<E: Env + 'static>(url: &Url) -> Effect {
        let endpoint = url
            .join(CINEMETA_FEED_CATALOG_ID)
            .expect("url builder failed");

        let request = Request::get(endpoint.as_str())
            .body(())
            .expect("request builder failed");

        EffectFuture::Concurrent(
            E::fetch::<_, SearchableItemsResponse>(request)
                .map(enclose!((url) move |response| {
                    let result = response.map(|response| response.0);

                    Msg::Internal(Internal::LoadLocalSearchResult(
                        url, result,
                    ))
                }))
                .boxed_env(),
        )
        .into()
    }

    fn index(&self, index_options: IndexOptions, score_threshold: f64) -> Searcher<Searchable> {
        let max_imdb_rating = self
            .current_records
            .iter()
            // it's ok to set rating to 0 for the max if non is present
            .map(|searchable| searchable.imdb_rating.unwrap_or_default())
            .max_by(|rating_a, rating_b| rating_a.partial_cmp(rating_b).unwrap())
            .unwrap_or_default();

        let max_popularity = self
            .current_records
            .iter()
            .map(|searchable| searchable.popularity.unwrap_or_default())
            // it's ok to set popularity to 0 for the max if non is present
            .max_by(|popularity_a, popularity_b| popularity_a.partial_cmp(&popularity_b).unwrap())
            .unwrap_or_default();

        let score_computer = move |searchable: &Searchable| {
            let imdb_rating_boost = searchable
                .imdb_rating
                .map(|imdb_rating| {
                    (imdb_rating.to_f64() / max_imdb_rating.to_f64()
                        * index_options.imdb_rating_weight)
                        .exp()
                })
                .unwrap_or(1.0);

            let popularity_boost = searchable
                .popularity
                .and_then(|popularity| {
                    // make sure we always have > 0, because ratio will panic if denom is 0!
                    let popularity_percent =
                        Ratio::new(popularity, max_popularity.max(1)).to_f64()?;

                    Some((popularity_percent * index_options.imdb_rating_weight).exp())
                })
                .unwrap_or(1.0);

            imdb_rating_boost * popularity_boost
        };

        Searcher::builder(self.current_records.clone(), |item| &item.name)
            .boost_computer(score_computer)
            .score_threshold(score_threshold)
            .build()
    }
}

impl<E: Env + 'static> UpdateWithCtx<E> for LocalSearch {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        let cinemeta_addon = ctx.profile.addons.first().unwrap();

        match msg {
            Msg::Action(Action::Search(ActionSearch::Autocomplete(search_text))) => {
                match self.searcher {
                    // local search can be performed
                    Some(searcher) => {}
                    // we first need to load the Searchable records from Cinemeta
                    None => return Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::LoadLocalSearchResult(url, result)) => {
                match result {
                    Ok(searchable) => {
                        // update the latest records, used for refreshing the list
                        let last_records_effects = eq_update(
                            &mut self.latest_records,
                            Loadable::Ready(searchable.to_owned()),
                        );

                        // and the current, used for the search itself
                        let current_records_effects =
                            eq_update(&mut self.current_records, searchable.to_owned());

                        const INDEX_OPTIONS: IndexOptions = IndexOptions {
                            imdb_rating_weight: 0.5,
                            popularity_weight: 0.5,
                        };
                        let searcher = self.index(INDEX_OPTIONS, DEFAULT_SCORE_THRESHOLD);

                        let searcher_effects = eq_update(&mut self.searcher, Some(searcher));

                        last_records_effects
                            .join(current_records_effects)
                            .join(searcher_effects)
                    }
                    Err(error) => {
                        // update the latest records, but leave the current_records
                        // this will ensure that the user can still search locally with autocomplete
                        eq_update(&mut self.latest_records, Loadable::Err(error.to_owned()))
                    }
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}

mod imdb_rating {
    use std::{
        convert::{TryFrom, TryInto},
        fmt,
        num::{ParseFloatError, ParseIntError},
        str::FromStr,
    };

    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use thiserror::Error;

    /// With a scale between 0 and 10 in either a floating or whole numbers
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Default)]
    #[serde(transparent)]
    pub struct ImdbRating(f64);

    impl ImdbRating {
        pub fn to_f64(self) -> f64 {
            self.0
        }
    }

    impl<'de> Deserialize<'de> for ImdbRating {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum Helper {
                Float(f64),
                String(String),
            }

            let rating = match Helper::deserialize(deserializer)? {
                Helper::Float(float) => float,
                Helper::String(maybe_float) => {
                    maybe_float.parse().map_err(serde::de::Error::custom)?
                }
            };

            Self::try_from(rating).map_err(serde::de::Error::custom)
        }
    }

    impl TryFrom<String> for ImdbRating {
        type Error = anyhow::Error;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            let imdb_rating = value.parse::<ImdbRating>()?;

            Ok(imdb_rating)
        }
    }

    #[derive(Error, Debug)]
    pub enum ParseError {
        /// Failed to parse number.
        #[error("Failed to parse percent number: {0:?}")]
        Parsing(ParseFloatError),
        /// A percentage can only be between 0 and 100.
        #[error("Rating should be between 0.0 and 10.0")]
        OutOfRange,
    }

    impl TryFrom<f64> for ImdbRating {
        type Error = ParseError;

        fn try_from(value: f64) -> Result<Self, Self::Error> {
            if value >= 0.0 && value <= 10.0 {
                Ok(Self(value))
            } else {
                Err(ParseError::OutOfRange)
            }
        }
    }

    impl FromStr for ImdbRating {
        type Err = ParseError;

        fn from_str(value: &str) -> Result<Self, Self::Err> {
            let rating = value
                .parse::<f64>()
                .map_err(|err| ParseError::Parsing(err))
                .and_then(|parsed_value| Self::try_from(parsed_value))?;

            Ok(rating)
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use serde_json::Value;

    use super::*;

    #[test]
    fn test_imdb_rating_parsing() {
        // Strings
        {
            let good_values = [
                ("10", ImdbRating::try_from(10.0_f64).unwrap()),
                ("0", ImdbRating::try_from(0.0_f64).unwrap()),
                ("5.5", ImdbRating::try_from(5.5_f64).unwrap()),
            ];

            let bad_values = [
                // larger than 10
                "11",
                // lower than 0
                "-10",
                // a text
                "Not a number",
            ];

            let good_results = good_values
                .iter()
                .map(|(percent_str, expected)| (percent_str.parse::<ImdbRating>(), expected))
                .collect::<Vec<_>>();

            for (good_result, expected) in good_results {
                assert_eq!(&good_result.expect("Should parse successfully"), expected);
            }

            let bad_results = bad_values
                .iter()
                .map(|bad_str| bad_str.parse::<ImdbRating>())
                .collect::<Vec<_>>();

            assert_eq!(3, bad_results.len());
            assert!(matches!(bad_results[0], Err(ParseError::OutOfRange)));
            assert!(matches!(bad_results[1], Err(ParseError::OutOfRange)));
            assert!(matches!(bad_results[2], Err(ParseError::Parsing(_))));
        }
    }
}
