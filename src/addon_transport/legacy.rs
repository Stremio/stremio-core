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

const IMDB_PREFIX: &str = "tt";
const YT_PREFIX: &str = "UC";

// @TODO this can also be an error, so consider using that and turning it into a meaningful err
// @TODO: also, mapping to ResourceResponse can be done here
#[derive(Deserialize)]
struct JsonRPCResp<T> {
    result: T,
}

// @TODO tests
// test whether we can map some pre-defined request to the proper expected result,
// which we'll take from the JS adapter and the legacy JS system
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
                    .map(|r| Box::new(ResourceResponse::Metas { metas: (*r).result })),
            ),
            "meta" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<MetaItem>>(fetch_req)
                    .map(|r| Box::new(ResourceResponse::Meta { meta: (*r).result })),
            ),
            "stream" => Box::new(
                T::fetch_serde::<_, JsonRPCResp<Vec<Stream>>>(fetch_req).map(|r| {
                    Box::new(ResourceResponse::Streams {
                        streams: (*r).result,
                    })
                }),
            ),
            // @TODO better error
            _ => Box::new(future::err("legacy transport: unsupported response".into())),
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
            build_jsonrpc("meta.find", json!({
                "query": query,
                "limit": 100,
                "sort": sort,
                "skip": req.resource_ref.get_extra_first_val("skip")
                    .map(|s| s.parse::<i32>().unwrap_or(0))
                    .unwrap_or(0),
            }))
        }
        "meta" => build_jsonrpc("meta.get", json!({
            "query": query_from_id(id),
        })),
        // @TODO
        "streams" => json!({}),
        // @TODO better error
        _ => return Err("legacy transport: unsupported resource".into()),
    };
    // NOTE: tihs is not using a URL safe base64 standard, which means that technically this is
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
    if id.starts_with(IMDB_PREFIX) {
        return json!({ "imdb_id": id });
    }
    if id.starts_with(YT_PREFIX) {
        return json!({ "yt_id": id });
    }
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() == 2 {
        return json!({ parts[0].to_owned(): parts[1] });
    }
    Value::Null
}
