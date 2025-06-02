use http::{header::CONTENT_TYPE, Method, Request};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    constants::USER_LIKES_API_URL,
    types::profile::{AuthKey, UserId},
};

pub trait RequestParameters<T> {
    /// Version path prefix for the request
    const VERSION: &'static str = "";

    fn endpoint(&self) -> Url;
    fn method(&self) -> Method;
    fn path(&self) -> String;

    /// Returns the versioned path for the API request.
    ///
    /// In case of v1 we do not have any prefix and the default [`FetchRequestParams::VERSION`] is an empty string.
    ///
    /// V1 path: `create`
    /// V2 path: `v2/create` (where version prefix is `"v2"`)
    fn version_path(&self) -> String {
        if Self::VERSION.is_empty() {
            self.path()
        } else {
            format!(
                "{version}/{path}",
                version = Self::VERSION,
                path = &self.path(),
            )
        }
    }
    fn query(&self) -> Option<String>;
    fn body(self) -> T;

    fn build(self) -> Result<Request<Option<serde_json::Value>>, anyhow::Error>
    where
        Self: Sized,
    {
        anyhow::bail!("Not implemented")
    }
}

/// `userId` - The user's ID
/// `authToken` - Either auth token or user id must be provided
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum UserAuthentication {
    UserId(UserId),
    AuthToken(AuthKey),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusQuery {
    // - `userId` - The user's ID
    // - `authToken` - Either auth token or user id must be provided
    #[serde(flatten)]
    pub user_auth: UserAuthentication,
    // `mediaId` - The IMDb ID, TMDB ID or Kitsu ID of the movie or series (examples: `tt30988739`, `kitsu:7442`, `tmdb:1197306`)
    pub media_id: String,
    /// `movie`, `series`, `anime`, etc
    pub media_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum APIRequest {
    /// `/api/send` - Record user ratings (watched, liked, very liked)
    Send(SendRequest),
    /// `/api/get_status` - Get the status of a specific item for a user
    GetStatus(GetStatusRequest),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetStatusRequest {
    pub query: GetStatusQuery,
}

impl RequestParameters<Option<serde_json::Value>> for APIRequest {
    fn endpoint(&self) -> Url {
        USER_LIKES_API_URL.to_owned()
    }
    fn method(&self) -> Method {
        match self {
            APIRequest::Send(_request) => Method::POST,
            _ => Method::GET,
        }
    }
    fn path(&self) -> String {
        match self {
            APIRequest::Send(_request) => "send",
            APIRequest::GetStatus(_) => "get_status",
        }
        .into()
    }
    fn query(&self) -> Option<String> {
        match self {
            APIRequest::Send(..) => None,
            APIRequest::GetStatus(request) => Some(
                serde_url_params::to_string(&request.query).expect("Serialize query params failed"),
            ),
        }
    }
    fn body(self) -> Option<serde_json::Value> {
        match self {
            APIRequest::Send(request) => {
                Some(serde_json::to_value(&request).expect("Should always work"))
            }
            _ => None,
        }
    }

    fn build(self) -> Result<Request<Option<serde_json::Value>>, anyhow::Error> {
        let mut url = self
            .endpoint()
            .join("api/")
            .expect("url builder failed")
            .join(&self.version_path())
            .expect("url builder failed");
        url.set_query(self.query().as_deref());

        let req = Request::builder()
            .method(self.method())
            .uri(url.as_str())
            .header(CONTENT_TYPE, "application/json")
            .body(self.body())?;

        Ok(req)
    }
}

/// ```
/// use stremio_core::types::{user_likes::{like, SendRequest, UserAuthentication}, profile::AuthKey};
///
/// let json = serde_json::json!({
///   "authToken": "token123",
///   "mediaId": "tt0111161",
///   "mediaType": "movie",
///   "status": "loved"
/// });
///
/// let request = SendRequest {
///     user_auth: UserAuthentication::AuthToken(AuthKey("token123".to_string())),
///     media_id: "tt0111161".to_string(),
///     media_type: "movie".to_string(),
///     status: Some(like::Status::Loved),
/// };
///
/// let value_actual = serde_json::to_value(&request).expect("Should serialize");
/// assert_eq!(value_actual, json);
///
/// let de_actual = serde_json::from_value::<SendRequest>(json).expect("Should deserialize");
/// assert_eq!(de_actual, request);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendRequest {
    // - `userId` - The user's ID
    // - `authToken` - Either auth token or user id must be provided
    #[serde(flatten)]
    pub user_auth: UserAuthentication,
    /// The IMDb ID, TMDB ID or Kitsu ID of the movie or series (examples: `tt30988739`, `kitsu:7442`, `tmdb:1197306`)
    pub media_id: String,
    /// `movie`, `series`, `anime`, etc
    pub media_type: String,
    /// To clear a rating, omit the status field entirely
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<like::Status>,
}

pub mod like {
    use serde::{Deserialize, Serialize};

    #[derive(
        parse_display::Display,
        parse_display::FromStr,
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
    )]
    #[serde(try_from = "String", into = "String")]
    pub enum Status {
        #[display("watched")]
        Watched,
        #[display("liked")]
        Liked,
        #[display("loved")]
        Loved,
    }

    impl TryFrom<String> for Status {
        type Error = parse_display::ParseError;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            value.parse()
        }
    }

    impl From<Status> for String {
        fn from(status: Status) -> Self {
            status.to_string()
        }
    }
}
