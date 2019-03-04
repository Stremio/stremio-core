use super::AddonTransport;
use crate::state_types::{EnvFuture, Environment, Request};
use crate::types::addons::{ResourceRequest, ResourceResponse};
use crate::types::*;
use futures::{future, Future};
use serde_derive::*;
use serde_json::json;
use serde_json::value::Value;
use std::error::Error;
use std::marker::PhantomData;
use std::fmt;


const IMDB_PREFIX: &str = "tt";
const YT_PREFIX: &str = "UC";

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

#[derive(Deserialize, Debug)]
pub struct JsonRPCErr {
    message: String,
    #[serde(default)]
    code: i64,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum JsonRPCResp<T> {
    Result{ result: T },
    Error { error: JsonRPCErr },
}

impl From<Vec<MetaPreview>> for ResourceResponse {
    fn from(metas: Vec<MetaPreview>) -> Self {
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

fn map_response<T: 'static + Sized>(resp: Box<JsonRPCResp<T>>) -> EnvFuture<T> {
    match *resp {
        JsonRPCResp::Result{ result } => Box::new(future::ok(result)),
        JsonRPCResp::Error{ error } => Box::new(future::err(LegacyErr::JsonRPC(error).into())),
    }
}


pub struct AddonLegacyTransport<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonTransport for AddonLegacyTransport<T> {
    fn get(req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        let fetch_req = match build_legacy_req(req) {
            Ok(r) => r,
            Err(e) => return Box::new(future::err(e.into())),
        };

        match &req.resource_ref.resource as &str {
            "catalog" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<Vec<MetaPreview>>>(fetch_req)
                    .and_then(map_response)
                    .map(|r| Box::new(r.into())),
            ),
            "meta" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<MetaItem>>(fetch_req)
                    .and_then(map_response)
                    .map(|r| Box::new(r.into())),
            ),
            "stream" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<Vec<Stream>>>(fetch_req)
                    .and_then(map_response)
                    .map(|r| Box::new(r.into())),
            ),
            _ => Box::new(future::err(LegacyErr::UnsupportedResource.into())),
        }
    }
}

fn build_legacy_req(req: &ResourceRequest) -> Result<Request<()>, Box<dyn Error>> {
    // Limitations of this legacy adapter:
    // * does not support subtitles
    // * does not support searching (meta.search)
    // Those limitations are intentional, to avoid this getting complicated
    // They affect functionality very little - there are no subtitles add-ons using the legacy
    // protocol (other than OpenSubtitles, which will be ported) and there's only one
    // known legacy add-on that support search (Stremio/stremio #379)
    let type_name = &req.resource_ref.type_name;
    let id = &req.resource_ref.id;
    let q_json = match &req.resource_ref.resource as &str {
        "catalog" => {
            let genre = req.resource_ref.get_extra_first_val("genre");
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
                    "skip": req.resource_ref.get_extra_first_val("skip")
                        .map(|s| s.parse::<u32>().unwrap_or(0))
                        .unwrap_or(0),
                }),
            )
        }
        "meta" => build_jsonrpc("meta.get", json!({ "query": query_from_id(id) })),
        "stream" => {
            let mut query = match query_from_id(id) {
                Value::Object(q) => q,
                _ => return Err("legacy: stream request without a valid id".into()),
            };
            query.insert("type".into(), Value::String(type_name.to_owned()));
            build_jsonrpc("stream.find", json!({ "query": query }))
        }
        _ => return Err(LegacyErr::UnsupportedRequest.into()),
    };
    // NOTE: this is not using a URL safe base64 standard, which means that technically this is
    // not safe; however, the original implementation of stremio-addons work the same way,
    // so we're technically replicating a legacy bug on purpose
    // https://github.com/Stremio/stremio-addons/blob/v2.8.14/rpc.js#L53
    let param_str = base64::encode(&serde_json::to_string(&q_json)?);
    let url = format!("{}/q.json?b={}", &req.transport_url, param_str);
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
    use crate::types::addons::ResourceRef;

    // Those are a bit sensitive for now, but that's a good thing, since it will force us
    // to pay attention to minor details that might matter with the legacy system
    // (e.g. omitting values vs `null` values)
    #[test]
    fn catalog() {
        let resource_req = ResourceRequest {
            transport_url: "https://stremio-mixer.schneider.ax/stremioget/stremio/v1".to_owned(),
            resource_ref: ResourceRef::without_extra("catalog", "tv", "popularities.mixer"),
        };
        assert_eq!(
            &build_legacy_req(&resource_req).unwrap().uri().to_string(),
            "https://stremio-mixer.schneider.ax/stremioget/stremio/v1/q.json?b=eyJpZCI6MSwianNvbnJwYyI6IjIuMCIsIm1ldGhvZCI6Im1ldGEuZmluZCIsInBhcmFtcyI6W251bGwseyJsaW1pdCI6MTAwLCJxdWVyeSI6eyJ0eXBlIjoidHYifSwic2tpcCI6MCwic29ydCI6eyJwb3B1bGFyaXRpZXMubWl4ZXIiOi0xLCJwb3B1bGFyaXR5IjotMX19XX0=",
        );
    }

    #[test]
    fn stream_imdb() {
        let resource_req = ResourceRequest {
            transport_url: "https://legacywatchhub.strem.io/stremio/v1".to_owned(),
            resource_ref: ResourceRef::without_extra("stream", "series", "tt0386676:5:1"),
        };
        assert_eq!(
            &build_legacy_req(&resource_req).unwrap().uri().to_string(),
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
