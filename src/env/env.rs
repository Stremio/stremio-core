use crate::env::EnvError;
use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
use future::Either;
use futures::{future, Future};
use http::{Method, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use stremio_core::state_types::{EnvFuture, Environment};
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;

pub enum Env {}

impl Env {
    fn local_storage() -> Result<web_sys::Storage, EnvError> {
        web_sys::window()
            .expect("window is not available")
            .local_storage()?
            .ok_or(EnvError::StorageMissing)
    }
    fn get_storage_sync<T>(key: &str) -> Result<Option<T>, EnvError>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        let storage = Self::local_storage()?;
        let value = storage.get_item(key)?;
        Ok(match value {
            Some(value) => Some(serde_json::from_str(&value)?),
            None => None,
        })
    }
    fn set_storage_sync<T: Serialize>(key: &str, value: Option<&T>) -> Result<(), EnvError> {
        let storage = Self::local_storage()?;
        match value {
            Some(value) => {
                let serialized_value = serde_json::to_string(value)?;
                storage.set_item(key, &serialized_value)?
            }
            None => storage.remove_item(key)?,
        };
        Ok(())
    }
}

impl Environment for Env {
    fn fetch<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: Serialize + 'static,
        for<'de> OUT: Deserialize<'de> + 'static,
    {
        let (parts, body) = request.into_parts();
        let url = parts.uri.to_string();
        let method = parts.method.as_str();
        let headers = {
            let mut headers = HashMap::new();
            for (key, value) in parts.headers.iter() {
                let key = key.as_str().to_owned();
                let value = String::from_utf8_lossy(value.as_bytes()).into_owned();
                headers.entry(key).or_insert_with(Vec::new).push(value)
            }
            JsValue::from_serde(&headers).unwrap()
        };
        let body = match serde_json::to_string(&body) {
            Ok(ref body) if body != "null" && parts.method != Method::GET => {
                Some(JsValue::from_str(&body))
            }
            _ => None,
        };
        let mut request_options = web_sys::RequestInit::new();
        request_options
            .method(method)
            .headers(&headers)
            .body(body.as_ref());
        let request = web_sys::Request::new_with_str_and_init(&url, &request_options)
            .expect("request builder failed");
        let promise = web_sys::window()
            .expect("window is not available")
            .fetch_with_request(&request);
        Box::new(
            JsFuture::from(promise)
                .map_err(EnvError::from)
                .and_then(|resp| {
                    let resp = resp.dyn_into::<web_sys::Response>().unwrap();
                    if resp.status() == 200 {
                        // @TODO: optimize this, as this is basically deserializing in JS -> serializing in
                        // JS -> deserializing in rust
                        // NOTE: there's no realistic scenario those unwraps fail
                        Either::A(JsFuture::from(resp.json().unwrap()).map_err(EnvError::from))
                    } else {
                        Either::B(future::err(EnvError::HTTPStatusCode(resp.status())))
                    }
                })
                .and_then(|json| match json.into_serde() {
                    Ok(r) => future::ok(r),
                    Err(e) => future::err(EnvError::from(e)),
                })
                .map_err(Into::into),
        )
    }
    fn exec(future: Box<dyn Future<Item = (), Error = ()>>) {
        spawn_local(future)
    }
    fn now() -> DateTime<Utc> {
        let millis = js_sys::Date::now() as i64;
        let (secs, millis) = (millis / 1000, millis % 1000);
        Utc.timestamp(secs, millis as u32 * 1_000_000)
    }
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        Box::new(match Self::get_storage_sync(key) {
            Ok(result) => future::ok(result),
            Err(error) => future::err(error.into()),
        })
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
        Box::new(match Self::set_storage_sync(key, value) {
            Ok(result) => future::ok(result),
            Err(error) => future::err(error.into()),
        })
    }
}
