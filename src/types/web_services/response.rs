use std::collections::HashMap;

use chrono::{DateTime, Utc};
use http::Request;
use serde::{Deserialize, Serialize};

pub struct TraktWatchedRequest {
    pub access_token: String,
}

impl From<TraktWatchedRequest> for Request<()> {
    fn from(value: TraktWatchedRequest) -> Self {
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
pub struct TraktWatchedResponse(Vec<TraktType>);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum TraktType {
    Movies(Vec<Movie>),
    Shows(Vec<Show>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movie {
    plays: u64,
    last_watched_at: DateTime<Utc>,
    last_updated_at: DateTime<Utc>,
    movie: Data,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Data {
    pub title: String,
    pub year: u32,
    pub ids: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Show {
    pub plays: u64,
    pub last_watched_at: DateTime<Utc>,
    pub last_updated_at: DateTime<Utc>,
    pub reset_at: Option<DateTime<Utc>>,
    pub show: Data,
    pub seasons: Vec<Season>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Season {
    pub number: u64,
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Episode {
    pub number: u64,
    pub plays: u64,
    pub last_watched_at: DateTime<Utc>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_movie_deserialization() {
        let movie_json = serde_json::json!({
            "type": "movies",
                "data": [
                    {
                        "plays": 3,
                        "last_watched_at": "2023-12-31T15:46:51.000Z",
                        "last_updated_at": "2023-12-31T15:46:51.000Z",
                        "movie": {
                            "title": "The Beanie Bubble",
                            "year": 2023,
                            "ids": {
                                "trakt": 741930,
                                "slug": "the-beanie-bubble-2023",
                                "imdb": "tt17007120",
                                "tmdb": 926008
                            }
                        }
                    },
                ],
        });

        // serde_path_to_error::
        let response = serde_json::from_value::<TraktWatchedResponse>(movie_json)
            .expect("Should deserialize movie");

        match response.0.get(0).unwrap() {
            TraktType::Movies(movies) => {
                // assert!
                // assert_eq!
                // ....
            }
            _ => panic!("We expected a movie, found different TraktType"),
        }
    }

    #[test]
    fn test_show_deserialization() {
        let show_json = serde_json::json!([{
            "type": "shows",
            "data": [{
                "plays": 12,
                "last_watched_at": "2024-08-22T21:30:08.000Z",
                "last_updated_at": "2024-08-22T21:30:08.000Z",
                "reset_at": null,
                "show": {
                    "title": "The Marvelous Mrs. Maisel",
                    "year": 2017,
                    "ids": {
                        "trakt": 118164,
                        "slug": "the-marvelous-mrs-maisel",
                        "tvdb": 326791,
                        "imdb": "tt5788792",
                        "tmdb": 70796,
                        "tvrage": null
                    }
                },
                "seasons": [
                    {
                        "number": 1,
                        "episodes": [
                            {
                                "number": 1,
                                "plays": 1,
                                "last_watched_at": "2024-08-16T20:47:26.000Z"
                            },
                            {
                                "number": 2,
                                "plays": 2,
                                "last_watched_at": "2024-08-17T18:05:16.000Z"
                            },
                            {
                                "number": 3,
                                "plays": 2,
                                "last_watched_at": "2024-08-18T18:15:45.000Z"
                            },
                            {
                                "number": 4,
                                "plays": 1,
                                "last_watched_at": "2024-08-18T19:31:33.000Z"
                            },
                            {
                                "number": 6,
                                "plays": 1,
                                "last_watched_at": "2024-08-19T19:37:19.000Z"
                            }
                        ]
                    },
                    {
                        "number": 2,
                        "episodes": [
                            {
                                "number": 1,
                                "plays": 1,
                                "last_watched_at": "2024-08-20T21:28:05.000Z"
                            },
                            {
                                "number": 2,
                                "plays": 1,
                                "last_watched_at": "2024-08-21T20:41:39.000Z"
                            },
                            {
                                "number": 3,
                                "plays": 1,
                                "last_watched_at": "2024-08-22T19:21:07.000Z"
                            },
                            {
                                "number": 4,
                                "plays": 2,
                                "last_watched_at": "2024-08-22T21:30:08.000Z"
                            }
                        ]
                    }
                ]
            }]
        }]);

        let response = serde_json::from_value::<TraktWatchedResponse>(show_json)
            .expect("Should deserialize movie");

        match &response.0.get(0).unwrap() {
            TraktType::Shows(shows) => {
                // assert!
                // assert_eq!
                // ....
                dbg!(&shows);
            }
            _ => panic!("We expected a show, found different TraktType"),
        }
    }
}
