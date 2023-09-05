mod resource_request;

use stremio_core::types::addon::ResourceResponse;

fn main() {
    let meta_detailed_json = serde_json::json!({
        "metasDetailed": [{
        "imdb_id": "tt5180504",
        "name": "The Witcher",
        "year": "2019–",
        "genre": [
          "Action",
          "Adventure",
          "Drama"
        ],
        "released": "2019-12-20T00:00:00.000Z",
        "director": [
          "Tomek Baginski"
        ],
        "writer": [
          "Lauren Schmidt Hissrich"
        ],
        "imdbRating": "8.1",
        "description": "Geralt of Rivia, a solitary monster hunter, struggles to find his place in a world where people often prove more wicked than beasts.",
        "country": "Poland, United States, Hungary",
        "type": "series",
        "slug": "series/the-witcher-5180504",
        "popularities": {
          "stremio": 25.205533333333335,
          "stremio_lib": 0,
          "moviedb": 756.166,
          "trakt": 19
        },
        "cast": [
          "Henry Cavill",
          "Freya Allan",
          "Anya Chalotra"
        ],
        "poster": "https://images.metahub.space/poster/small/tt5180504/img",
        "runtime": "60 min",
        "status": "Continuing",
        "background": "https://images.metahub.space/background/medium/tt5180504/img",
        "logo": "https://images.metahub.space/logo/medium/tt5180504/img",
        "awards": "Nominated for 3 Primetime Emmys. 6 wins & 23 nominations total",
        "trailers": [
          {
            "source": "eb90gqGYP9c",
            "type": "Trailer"
          },
          {
            "source": "ndl1W4ltcmg",
            "type": "Trailer"
          }
        ],
        "popularity": 25.205533333333335,
        "id": "tt5180504",
        "videos": [
          {
            "name": "Bottled Appetites",
            "season": 1,
            "number": 5,
            "firstAired": "2019-12-20T03:00:00.000Z",
            "tvdb_id": 7428572,
            "rating": 0,
            "overview": "Heedless of warnings, Yennefer looks for a cure to restore what she's lost. Geralt inadvertently puts Jaskier in peril. The search for Ciri intensifies.",
            "thumbnail": "https://episodes.metahub.space/tt5180504/1/5/w780.jpg",
            "id": "tt5180504:1:5",
            "released": "2019-12-20T03:00:00.000Z",
            "episode": 5,
            "description": "Heedless of warnings, Yennefer looks for a cure to restore what she's lost. Geralt inadvertently puts Jaskier in peril. The search for Ciri intensifies."
          }
        ],
        "genres": [
          "Action",
          "Adventure",
          "Drama"
        ],
        "releaseInfo": "2019–",
        "trailerStreams": [
          {
            "title": "The Witcher",
            "ytId": "eb90gqGYP9c"
          },
          {
            "title": "The Witcher",
            "ytId": "ndl1W4ltcmg"
          }
        ],
        "links": [
          {
            "name": "8.1",
            "category": "imdb",
            "url": "https://imdb.com/title/tt5180504"
          },
          {
            "name": "The Witcher",
            "category": "share",
            "url": "https://www.strem.io/s/series/the-witcher-5180504"
          },
        ],
        "behaviorHints": {
          "defaultVideoId": null,
          "hasScheduledVideos": true
        }
      }]
    });

    serde_json::from_value::<ResourceResponse>(meta_detailed_json).expect("Should deserialize");
}
