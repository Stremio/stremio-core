use http::Request;
use serde::Serialize;

use crate::{
    constants::USER_LIKES_API_URL,
    types::{
        profile::AuthKey,
        rating::Rating,
        resource::{MetaItemId, MetaItemType},
    },
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingGetStatusRequest {
    pub auth_key: AuthKey,
    pub meta_item_id: MetaItemId,
    pub meta_item_type: MetaItemType,
}

impl From<RatingGetStatusRequest> for Request<()> {
    fn from(val: RatingGetStatusRequest) -> Self {
        let endpoint = USER_LIKES_API_URL
            .join(&format!(
                "api/get_status?authToken={}&mediaId={}&mediaType={}",
                val.auth_key, val.meta_item_id, val.meta_item_type
            ))
            .expect("url builder failed");

        Request::get(endpoint.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(())
            .expect("request builder should never fail!")
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingSendRequestBody {
    pub auth_token: AuthKey,
    pub media_id: MetaItemId,
    pub media_type: MetaItemType,
    pub status: Option<Rating>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingSendRequest {
    pub auth_key: AuthKey,
    pub meta_item_id: MetaItemId,
    pub meta_item_type: MetaItemType,
    pub rating: Option<Rating>,
}

impl From<RatingSendRequest> for Request<RatingSendRequestBody> {
    fn from(val: RatingSendRequest) -> Self {
        let endpoint = USER_LIKES_API_URL
            .join("api/send")
            .expect("url builder failed");

        let body = RatingSendRequestBody {
            auth_token: val.auth_key,
            media_id: val.meta_item_id,
            media_type: val.meta_item_type,
            status: val.rating,
        };

        Request::post(endpoint.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(body)
            .expect("request builder should never fail!")
    }
}
