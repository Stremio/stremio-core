use super::AddonInterface;
use crate::state_types::{EnvError, EnvFuture, Environment, Request};
use crate::types::addon::*;
use crate::types::resource::*;
use futures::{future, Future};
use serde_derive::*;
use serde_json::json;
use serde_json::value::Value;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

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
impl fmt::Display for LegacyErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for LegacyErr {
    fn description(&self) -> &str {
        match self {
            LegacyErr::JsonRPC(err) => &err.message,
            LegacyErr::UnsupportedResource => "legacy transport: unsupported resource",
            LegacyErr::UnsupportedRequest => "legacy transport: unsupported request",
        }
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

#[allow(clippy::boxed_local)]
fn map_response<T: 'static + Sized>(resp: JsonRPCResp<T>) -> EnvFuture<T> {
    Box::new(match resp {
        JsonRPCResp::Result { result } => future::ok(result),
        JsonRPCResp::Error { error } => future::err(LegacyErr::JsonRPC(error).into()),
    })
}

//
// Transport implementation
//
pub struct AddonLegacyTransport<'a, T: Environment> {
    env: PhantomData<T>,
    transport_url: &'a str,
}

impl<'a, T: Environment> AddonLegacyTransport<'a, T> {
    pub fn from_url(transport_url: &'a str) -> Self {
        AddonLegacyTransport {
            env: PhantomData,
            transport_url,
        }
    }
}

impl<'a, T: Environment> AddonInterface for AddonLegacyTransport<'a, T> {
    fn get(&self, path: &ResourceRef) -> EnvFuture<ResourceResponse> {
        let fetch_req = match build_legacy_req(self.transport_url, path) {
            Ok(r) => r,
            Err(e) => return Box::new(future::err(e)),
        };

        match &path.resource as &str {
            "catalog" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<Vec<MetaItemPreview>>>(fetch_req)
                    .and_then(map_response)
                    .map(Into::into),
            ),
            "meta" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<MetaItem>>(fetch_req)
                    .and_then(map_response)
                    .map(Into::into),
            ),
            "stream" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<Vec<Stream>>>(fetch_req)
                    .and_then(map_response)
                    .map(Into::into),
            ),
            "subtitles" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<SubtitlesResult>>(fetch_req)
                    .and_then(map_response)
                    .map(Into::into),
            ),
            _ => Box::new(future::err(LegacyErr::UnsupportedResource.into())),
        }
    }
    fn manifest(&self) -> EnvFuture<Manifest> {
        let url = format!("{}/q.json?b={}", self.transport_url, MANIFEST_REQUEST_PARAM);
        let r = Request::get(url).body(()).expect("builder cannot fail");
        Box::new(
            T::fetch_serde::<_, JsonRPCResp<LegacyManifestResp>>(r)
                .and_then(map_response)
                .map(Into::into),
        )
    }
}

fn build_legacy_req(transport_url: &str, path: &ResourceRef) -> Result<Request<()>, EnvError> {
    // Limitations of this legacy adapter:
    // * does not support subtitles
    // * does not support searching (meta.search)
    // Those limitations are intentional, to avoid this getting complicated
    // They affect functionality very little - there are no subtitles add-ons using the legacy
    // protocol (other than OpenSubtitles, which will be ported) and there's only one
    // known legacy add-on that support search (Stremio/stremio #379)
    let type_name = &path.type_name;
    let id = &path.id;
    let q_json = match &path.resource as &str {
        "catalog" => {
            let genre = path.get_extra_first_val("genre");
            let query = if let Some(genre) = genre {
                json!({ "type": type_name, "genre": genre })
            } else {
                json!({ "type": type_name })
            };
            // Just follows the convention set out by stremboard
            // L287 cffb94e4a9c57f5872e768eff25164b53f004a2b
            let sort = if id != "top" {
                json!({ id.to_owned(): -1, "popularity": -1 })
            } else {
                Value::Null
            };
            build_jsonrpc(
                "meta.find",
                json!({
                    "query": query,
                    "limit": 100,
                    "sort": sort,
                    "skip": path.get_extra_first_val("skip")
                        .map(|s| s.parse::<u32>().unwrap_or(0))
                        .unwrap_or(0),
                }),
            )
        }
        "meta" => build_jsonrpc("meta.get", json!({ "query": query_from_id(id) })),
        "stream" => {
            // Just use the query, but add "type" to it
            let mut query = match query_from_id(id) {
                Value::Object(q) => q,
                _ => return Err("legacy: stream request without a valid id".into()),
            };
            query.insert("type".into(), Value::String(type_name.to_owned()));
            build_jsonrpc("stream.find", json!({ "query": query }))
        }
        "subtitles" => build_jsonrpc(
            "subtitles.find",
            json!({ "query": json!({ "itemHash": id }) }),
        ),
        _ => return Err(LegacyErr::UnsupportedRequest.into()),
    };
    // NOTE: this is not using a URL safe base64 standard, which means that technically this is
    // not safe; however, the original implementation of stremio-addons work the same way,
    // so we're technically replicating a legacy bug on purpose
    // https://github.com/Stremio/stremio-addons/blob/v2.8.14/rpc.js#L53
    let param_str = base64::encode(&serde_json::to_string(&q_json)?);
    let url = format!("{}/q.json?b={}", transport_url, param_str);
    Ok(Request::get(&url).body(())?)
}

fn build_jsonrpc(method: &str, params: Value) -> Value {
    json!({
        "params": [Value::Null, params],
        "method": method,
        "id": 1,
        "jsonrpc": "2.0",
    })
}

fn query_from_id(id: &str) -> Value {
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
    Value::Null
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::addon::ResourceRef;

    // Those are a bit sensitive for now, but that's a good thing, since it will force us
    // to pay attention to minor details that might matter with the legacy system
    // (e.g. omitting values vs `null` values)
    #[test]
    fn catalog() {
        let transport_url = "https://stremio-mixer.schneider.ax/stremioget/stremio/v1".to_owned();
        let path = ResourceRef::without_extra("catalog", "tv", "popularities.mixer");
        assert_eq!(
            &build_legacy_req(&transport_url, &path).unwrap().uri().to_string(),
            "https://stremio-mixer.schneider.ax/stremioget/stremio/v1/q.json?b=eyJpZCI6MSwianNvbnJwYyI6IjIuMCIsIm1ldGhvZCI6Im1ldGEuZmluZCIsInBhcmFtcyI6W251bGwseyJsaW1pdCI6MTAwLCJxdWVyeSI6eyJ0eXBlIjoidHYifSwic2tpcCI6MCwic29ydCI6eyJwb3B1bGFyaXRpZXMubWl4ZXIiOi0xLCJwb3B1bGFyaXR5IjotMX19XX0=",
        );
    }

    #[test]
    fn stream_imdb() {
        let transport_url = "https://legacywatchhub.strem.io/stremio/v1".to_owned();
        let path = ResourceRef::without_extra("stream", "series", "tt0386676:5:1");
        assert_eq!(
            &build_legacy_req(&transport_url, &path).unwrap().uri().to_string(),
            "https://legacywatchhub.strem.io/stremio/v1/q.json?b=eyJpZCI6MSwianNvbnJwYyI6IjIuMCIsIm1ldGhvZCI6InN0cmVhbS5maW5kIiwicGFyYW1zIjpbbnVsbCx7InF1ZXJ5Ijp7ImVwaXNvZGUiOjEsImltZGJfaWQiOiJ0dDAzODY2NzYiLCJzZWFzb24iOjUsInR5cGUiOiJzZXJpZXMifX1dfQ=="
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
