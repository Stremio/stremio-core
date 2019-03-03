use crate::state_types::{EnvFuture, Environment, Request};
use crate::types::addons::{ResourceRequest, ResourceResponse};
use crate::types::*;
use futures::{future, Future};
use serde_json::json;
use std::error::Error;
use std::marker::PhantomData;

// @TODO facilitate detect from URL (manifest)
pub trait AddonTransport {
    fn get(resource_req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>>;
}

const MANIFEST_PATH: &str = "/manifest.json";
const LEGACY_PATH: &str = "/stremio/v1";
#[derive(Default)]
pub struct AddonHTTPTransport<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> AddonTransport for AddonHTTPTransport<T> {
    fn get(req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        if req.transport_url.ends_with(MANIFEST_PATH) {
            Self::get_http(req)
        } else if req.transport_url.ends_with(LEGACY_PATH) {
            Self::get_legacy(req)
        } else {
            Box::new(future::err("invalid transport_url".into()))
        }
    }
}

impl<T: Environment> AddonHTTPTransport<T> {
    fn get_http(req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        let url = req
            .transport_url
            .replace(MANIFEST_PATH, &req.resource_ref.to_string());
        match Request::get(&url).body(()) {
            Ok(r) => T::fetch_serde::<_, ResourceResponse>(r),
            Err(e) => Box::new(future::err(e.into())),
        }
    }

    // @TODO isolate this into legacy.rs
    fn get_legacy(req: &ResourceRequest) -> EnvFuture<Box<ResourceResponse>> {
        let fetch_req = match build_legacy_req(req) {
            Ok(r) => r,
            Err(e) => return Box::new(future::err(e.into())),
        };

        match &req.resource_ref.resource as &str {
            "catalog" => Box::new(
                T::fetch_serde::<_, Vec<MetaPreview>>(fetch_req)
                    .map(|r| Box::new(ResourceResponse::Metas { metas: *r }))
            ),
            "meta" => Box::new(
                T::fetch_serde::<_, MetaItem>(fetch_req)
                    .map(|r| Box::new(ResourceResponse::Meta{ meta: *r }))
            ),
            // @TODO streams
            // @TODO better error
            _ => Box::new(future::err("unsupported request".into())),
        }
    }
}

fn build_legacy_req(req: &ResourceRequest) -> Result<Request<()>, Box<dyn Error>> {
    let q_json = match &req.resource_ref.resource as &str {
        // @TODO
        "catalog" => json!({}),
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
