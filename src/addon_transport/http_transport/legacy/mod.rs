use crate::addon_transport::AddonTransport;
use crate::constants::{
    BASE64, VIDEO_FILENAME_EXTRA_PROP, VIDEO_HASH_EXTRA_PROP, VIDEO_SIZE_EXTRA_PROP,
};
use crate::runtime::{ConditionalSend, Env, EnvError, EnvFutureExt, TryEnvFuture};
use crate::types::addon::{Manifest, ResourcePath, ResourceResponse};
use crate::types::resource::{MetaItem, MetaItemPreview, Stream, Subtitles};
use base64::Engine;
use futures::{future, TryFutureExt};
use http::Request;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::marker::PhantomData;
use url::Url;

mod legacy_manifest;
use self::legacy_manifest::LegacyManifestResp;

const IMDB_PREFIX: &str = "tt";
const YT_PREFIX: &str = "UC";

// this is base64 for {"params":[],"method":"meta","id":1,"jsonrpc":"2.0"}
const MANIFEST_REQUEST_PARAM: &str =
    "eyJwYXJhbXMiOltdLCJtZXRob2QiOiJtZXRhIiwiaWQiOjEsImpzb25ycGMiOiIyLjAifQ==";

//
// Errors
//
#[derive(Debug)]
pub enum LegacyErr {
    JsonRPC(JsonRPCErr),
    UnsupportedResource,
    UnsupportedRequest,
}

impl From<LegacyErr> for EnvError {
    fn from(err: LegacyErr) -> EnvError {
        EnvError::AddonTransport(match err {
            LegacyErr::JsonRPC(error) => format!("rpc error {}: {}", error.code, error.message),
            LegacyErr::UnsupportedResource => "legacy transport: unsupported resource".to_owned(),
            LegacyErr::UnsupportedRequest => "legacy transport: unsupported request".to_owned(),
        })
    }
}

//
// JSON RPC types
//
#[derive(Deserialize, Debug)]
pub struct JsonRPCErr {
    message: String,
    #[serde(default)]
    code: i64,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum JsonRPCResp<T> {
    Result { result: T },
    Error { error: JsonRPCErr },
}

#[derive(Deserialize)]
pub struct SubtitlesResult {
    pub id: String,
    pub all: Vec<Subtitles>,
}

impl From<Vec<MetaItemPreview>> for ResourceResponse {
    fn from(metas: Vec<MetaItemPreview>) -> Self {
        ResourceResponse::Metas { metas }
    }
}
impl From<MetaItem> for ResourceResponse {
    fn from(meta: MetaItem) -> Self {
        ResourceResponse::Meta { meta }
    }
}
impl From<Vec<Stream>> for ResourceResponse {
    fn from(streams: Vec<Stream>) -> Self {
        ResourceResponse::Streams { streams }
    }
}
impl From<SubtitlesResult> for ResourceResponse {
    fn from(subtitles_result: SubtitlesResult) -> Self {
        ResourceResponse::Subtitles {
            subtitles: subtitles_result.all,
        }
    }
}

fn map_response<T: Sized + ConditionalSend + 'static>(resp: JsonRPCResp<T>) -> TryEnvFuture<T> {
    match resp {
        JsonRPCResp::Result { result } => future::ok(result).boxed_env(),
        JsonRPCResp::Error { error } => future::err(LegacyErr::JsonRPC(error).into()).boxed_env(),
    }
}

//
// Transport implementation
//
pub struct AddonLegacyTransport<'a, T: Env> {
    env: PhantomData<T>,
    transport_url: &'a Url,
}

impl<'a, T: Env> AddonLegacyTransport<'a, T> {
    pub fn new(transport_url: &'a Url) -> Self {
        AddonLegacyTransport {
            env: PhantomData,
            transport_url,
        }
    }
}

impl<T: Env> AddonTransport for AddonLegacyTransport<'_, T> {
    fn resource(&self, path: &ResourcePath) -> TryEnvFuture<ResourceResponse> {
        let fetch_req = match build_legacy_req(self.transport_url, path) {
            Ok(r) => r,
            Err(e) => return future::err(e).boxed_env(),
        };

        match &path.resource as &str {
            "catalog" => T::fetch::<_, JsonRPCResp<Vec<MetaItemPreview>>>(fetch_req)
                .and_then(map_response)
                .map_ok(Into::into)
                .boxed_env(),
            "meta" => T::fetch::<_, JsonRPCResp<MetaItem>>(fetch_req)
                .and_then(map_response)
                .map_ok(Into::into)
                .boxed_env(),
            "stream" => T::fetch::<_, JsonRPCResp<Vec<Stream>>>(fetch_req)
                .and_then(map_response)
                .map_ok(Into::into)
                .boxed_env(),
            "subtitles" => T::fetch::<_, JsonRPCResp<SubtitlesResult>>(fetch_req)
                .and_then(map_response)
                .map_ok(Into::into)
                .boxed_env(),
            _ => future::err(LegacyErr::UnsupportedResource.into()).boxed_env(),
        }
    }
    fn manifest(&self) -> TryEnvFuture<Manifest> {
        let url = format!("{}/q.json?b={}", self.transport_url, MANIFEST_REQUEST_PARAM);
        let r = Request::get(url).body(()).expect("request builder failed");
        T::fetch::<_, JsonRPCResp<LegacyManifestResp>>(r)
            .and_then(map_response)
            .map_ok(Into::into)
            .boxed_env()
    }
}

fn build_legacy_req(transport_url: &Url, path: &ResourcePath) -> Result<Request<()>, EnvError> {
    // Limitations of this legacy adapter:
    // * does not support subtitles
    // * does not support searching (meta.search)
    // Those limitations are intentional, to avoid this getting complicated
    // They affect functionality very little - there are no subtitles add-ons using the legacy
    // protocol (other than OpenSubtitles, which will be ported) and there's only one
    // known legacy add-on that support search (Stremio/stremio #379)
    let r#type = &path.r#type;
    let id = &path.id;
    let q_json = match &path.resource as &str {
        "catalog" => {
            let genre = path.get_extra_first_value("genre");
            let query = if let Some(genre) = genre {
                json!({ "type": r#type, "genre": genre })
            } else {
                json!({ "type": r#type })
            };
            // Just follows the convention set out by stremboard
            // L287 cffb94e4a9c57f5872e768eff25164b53f004a2b
            let sort = if id != "top" {
                json!({ id.to_owned(): -1, "popularity": -1 })
            } else {
                serde_json::Value::Null
            };
            build_jsonrpc(
                "meta.find",
                json!({
                    "query": query,
                    "limit": 100,
                    "sort": sort,
                    "skip": path.get_extra_first_value("skip")
                        .map(|s| s.parse::<u32>().unwrap_or(0))
                        .unwrap_or(0),
                }),
            )
        }
        "meta" => build_jsonrpc("meta.get", json!({ "query": query_from_id(id) })),
        "stream" => {
            // Just use the query, but add "type" to it
            let mut query = match query_from_id(id) {
                serde_json::Value::Object(q) => q,
                _ => {
                    return Err(EnvError::AddonTransport(
                        "legacy: stream request without a valid id".to_owned(),
                    ))
                }
            };
            query.insert("type".into(), serde_json::Value::String(r#type.to_owned()));
            build_jsonrpc("stream.find", json!({ "query": query }))
        }
        "subtitles" => {
            let mut query = HashMap::new();
            query.insert("itemHash", serde_json::Value::String(id.replace(':', " ")));
            let video_hash = path.get_extra_first_value(VIDEO_HASH_EXTRA_PROP.name.as_str());
            if let Some(video_hash) = video_hash {
                query.insert(
                    VIDEO_HASH_EXTRA_PROP.name.as_str(),
                    serde_json::Value::String(video_hash.to_owned()),
                );
            }
            let video_size = path
                .get_extra_first_value(VIDEO_SIZE_EXTRA_PROP.name.as_str())
                .and_then(|video_size| video_size.parse().ok());
            if let Some(video_size) = video_size {
                query.insert(
                    VIDEO_SIZE_EXTRA_PROP.name.as_str(),
                    serde_json::Value::Number(video_size),
                );
            }
            let video_filename =
                path.get_extra_first_value(VIDEO_FILENAME_EXTRA_PROP.name.as_str());
            if let Some(video_filename) = video_filename {
                query.insert(
                    VIDEO_FILENAME_EXTRA_PROP.name.as_str(),
                    serde_json::Value::String(video_filename.to_owned()),
                );
            }
            build_jsonrpc("subtitles.find", json!({ "query": query }))
        }
        _ => return Err(LegacyErr::UnsupportedRequest.into()),
    };
    // NOTE: this is not using a URL safe base64 standard, which means that technically this is
    // not safe; however, the original implementation of stremio-addons work the same way,
    // so we're technically replicating a legacy bug on purpose
    // https://github.com/Stremio/stremio-addons/blob/v2.8.14/rpc.js#L53
    let param_str = BASE64.encode(
        serde_json::to_string(&q_json).map_err(|error| EnvError::Serde(error.to_string()))?,
    );
    let url = format!("{transport_url}/q.json?b={param_str}");
    Ok(Request::get(&url).body(()).expect("request builder failed"))
}

fn build_jsonrpc(method: &str, params: serde_json::Value) -> serde_json::Value {
    json!({
        "params": [serde_json::Value::Null, params],
        "method": method,
        "id": 1,
        "jsonrpc": "2.0",
    })
}

fn query_from_id(id: &str) -> serde_json::Value {
    let parts: Vec<&str> = id.split(':').collect();
    // IMDb format: tt...:(season:episode)?
    if id.starts_with(IMDB_PREFIX) {
        if parts.len() == 3 {
            return json!({
                "imdb_id": parts[0],
                "season": parts[1].parse::<u16>().unwrap_or(1),
                "episode": parts[2].parse::<u16>().unwrap_or(1),
            });
        } else {
            return json!({ "imdb_id": parts[0] });
        }
    }
    // YouTube format: UC...:video_id?
    if id.starts_with(YT_PREFIX) {
        if parts.len() == 2 {
            return json!({ "yt_id": parts[0], "video_id": parts[1] });
        } else {
            return json!({ "yt_id": parts[0] });
        }
    }
    // generic format: id_prefix:id:video_id?
    if parts.len() == 3 {
        return json!({
            parts[0].to_owned(): parts[1],
            "video_id": parts[2]
        });
    }
    if parts.len() == 2 {
        return json!({ parts[0].to_owned(): parts[1] });
    }
    serde_json::Value::Null
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::addon::{ExtraExt, ResourcePath};

    // Those are a bit sensitive for now, but that's a good thing, since it will force us
    // to pay attention to minor details that might matter with the legacy system
    // (e.g. omitting values vs `null` values)
    #[test]
    fn catalog() {
        let transport_url = Url::parse("https://stremio-mixer.schneider.ax/stremioget/stremio/v1")
            .expect("url parse failed");
        let path = ResourcePath::without_extra("catalog", "tv", "popularities.mixer");
        assert_eq!(
            &build_legacy_req(&transport_url, &path).unwrap().uri().to_string(),
            "https://stremio-mixer.schneider.ax/stremioget/stremio/v1/q.json?b=eyJpZCI6MSwianNvbnJwYyI6IjIuMCIsIm1ldGhvZCI6Im1ldGEuZmluZCIsInBhcmFtcyI6W251bGwseyJsaW1pdCI6MTAwLCJxdWVyeSI6eyJ0eXBlIjoidHYifSwic2tpcCI6MCwic29ydCI6eyJwb3B1bGFyaXRpZXMubWl4ZXIiOi0xLCJwb3B1bGFyaXR5IjotMX19XX0=",
        );
    }

    #[test]
    fn stream_imdb() {
        let transport_url =
            Url::parse("https://legacywatchhub.strem.io/stremio/v1").expect("url parse failed");
        let path = ResourcePath::without_extra("stream", "series", "tt0386676:5:1");
        assert_eq!(
            &build_legacy_req(&transport_url, &path).unwrap().uri().to_string(),
            "https://legacywatchhub.strem.io/stremio/v1/q.json?b=eyJpZCI6MSwianNvbnJwYyI6IjIuMCIsIm1ldGhvZCI6InN0cmVhbS5maW5kIiwicGFyYW1zIjpbbnVsbCx7InF1ZXJ5Ijp7ImVwaXNvZGUiOjEsImltZGJfaWQiOiJ0dDAzODY2NzYiLCJzZWFzb24iOjUsInR5cGUiOiJzZXJpZXMifX1dfQ=="
        );
    }

    #[test]
    fn subtitles_only_id() {
        let transport_url =
            Url::parse("https://legacywatchhub.strem.io/stremio/v1").expect("url parse failed");
        let path = ResourcePath::without_extra("subtitles", "series", "tt0386676:5:1");
        assert_eq!(
            &build_legacy_req(&transport_url, &path).unwrap().uri().to_string(),
            "https://legacywatchhub.strem.io/stremio/v1/q.json?b=eyJpZCI6MSwianNvbnJwYyI6IjIuMCIsIm1ldGhvZCI6InN1YnRpdGxlcy5maW5kIiwicGFyYW1zIjpbbnVsbCx7InF1ZXJ5Ijp7Iml0ZW1IYXNoIjoidHQwMzg2Njc2IDUgMSJ9fV19"
        );
    }

    #[test]
    fn subtitles_with_hash() {
        let transport_url =
            Url::parse("https://legacywatchhub.strem.io/stremio/v1").expect("url parse failed");
        let extra = &vec![]
            .extend_one(&VIDEO_HASH_EXTRA_PROP, Some("ffffffffff".to_string()))
            .extend_one(&VIDEO_SIZE_EXTRA_PROP, Some("1000000000".to_string()));
        let path = ResourcePath::with_extra("subtitles", "series", "tt0386676:5:1", extra);
        assert_eq!(
            &build_legacy_req(&transport_url, &path).unwrap().uri().to_string(),
            "https://legacywatchhub.strem.io/stremio/v1/q.json?b=eyJpZCI6MSwianNvbnJwYyI6IjIuMCIsIm1ldGhvZCI6InN1YnRpdGxlcy5maW5kIiwicGFyYW1zIjpbbnVsbCx7InF1ZXJ5Ijp7Iml0ZW1IYXNoIjoidHQwMzg2Njc2IDUgMSIsInZpZGVvSGFzaCI6ImZmZmZmZmZmZmYiLCJ2aWRlb1NpemUiOjEwMDAwMDAwMDB9fV19"
        );
    }

    #[test]
    fn query_meta() {
        assert_eq!(
            query_from_id("tt0386676"),
            json!({ "imdb_id": "tt0386676" })
        );
        assert_eq!(query_from_id("UC2312"), json!({ "yt_id": "UC2312" }));
        assert_eq!(query_from_id("custom:test"), json!({ "custom": "test" }));
    }

    #[test]
    fn query_stream() {
        assert_eq!(
            query_from_id("tt0386676:5:2"),
            json!({ "imdb_id": "tt0386676", "season": 5, "episode": 2 })
        );
        assert_eq!(query_from_id("yt_id:video"), json!({ "yt_id": "video" }));
        assert_eq!(
            query_from_id("custom:test:vid"),
            json!({ "custom": "test", "video_id": "vid" })
        );
    }
}
