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
    let q_json = match &req.resource_ref.resource as &str {
        // @TODO
        "catalog" => {
            // We need to make a struct, cause we want to skip `genre`
            #[derive(Serialize)]
            struct CatalogQuery<'a> {
                #[serde(rename="type")]
                type_name: &'a str,
                #[serde(skip_serializing_if="Option::is_none")]
                genre: Option<&'a str>,
            }

            // Just follows the convention set out by stremboard
            // L287 cffb94e4a9c57f5872e768eff25164b53f004a2b
            let sort = if req.resource_ref.id == "top" {
                Value::Null
            } else {
                json!({
                    req.resource_ref.id.to_owned(): -1,
                    "popularity": -1
                })
            };
            json!({
                "params": [Value::Null, {
                    "query": CatalogQuery {
                        type_name: &req.resource_ref.type_name,
                        genre: req.resource_ref.get_extra_first_val("genre"),
                    },
                    "limit": 100,
                    "sort": sort,
                    "skip": req.resource_ref.get_extra_first_val("skip")
                        .map(|s| s.parse::<i32>().unwrap_or(0))
                        .unwrap_or(0),
                }],
                "method": "meta.find",
                "id": 1,
                "jsonrpc": "2.0",
            })
        }
        // @TODO
        "meta" => json!({}),
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
