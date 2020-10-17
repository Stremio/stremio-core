use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
use futures::future::Either;
use futures::{future, Future, FutureExt, TryFutureExt};
use http::{Method, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use stremio_core::runtime::{Env, EnvError, EnvFuture};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};

pub enum WebEnv {}

impl Env for WebEnv {
    fn fetch<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: Serialize,
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
                headers.entry(key).or_insert_with(Vec::new).push(value);
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
        JsFuture::from(promise)
            .map_err(|error| EnvError::Fetch(js_error_message(error)))
            .and_then(|resp| {
                let resp = resp.dyn_into::<web_sys::Response>().unwrap();
                if resp.status() != 200 {
                    Either::Right(future::err(EnvError::Fetch(format!(
                        "Unexpected HTTP status code {}",
                        resp.status(),
                    ))))
                } else {
                    Either::Left(
                        JsFuture::from(resp.json().unwrap())
                            .map_err(|error| EnvError::Fetch(js_error_message(error))),
                    )
                }
            })
            .and_then(|resp| future::ready(resp.into_serde().map_err(EnvError::from)))
            .boxed_local()
    }
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        future::ready(get_storage_sync(key)).boxed_local()
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
        future::ready(set_storage_sync(key, value)).boxed_local()
    }
    fn exec<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future)
    }
    fn now() -> DateTime<Utc> {
        let millis = js_sys::Date::now() as i64;
        let (secs, millis) = (millis / 1000, millis % 1000);
        Utc.timestamp(secs, millis as u32 * 1_000_000)
    }
    #[cfg(debug_assertions)]
    fn log(message: String) {
        web_sys::console::log_1(&JsValue::from(message));
    }
}

fn local_storage() -> Result<web_sys::Storage, EnvError> {
    web_sys::window()
        .expect("window is not available")
        .local_storage()
        .map_err(|_| EnvError::StorageUnavailable)?
        .ok_or(EnvError::StorageUnavailable)
}

fn get_storage_sync<T>(key: &str) -> Result<Option<T>, EnvError>
where
    for<'de> T: Deserialize<'de> + 'static,
{
    let storage = local_storage()?;
    let value = storage
        .get_item(key)
        .map_err(|_| EnvError::StorageUnavailable)?;
    Ok(match value {
        Some(value) => Some(serde_json::from_str(&value)?),
        None => None,
    })
}

fn set_storage_sync<T: Serialize>(key: &str, value: Option<&T>) -> Result<(), EnvError> {
    let storage = local_storage()?;
    match value {
        Some(value) => {
            let serialized_value = serde_json::to_string(value)?;
            storage
                .set_item(key, &serialized_value)
                .map_err(|_| EnvError::StorageUnavailable)?;
        }
        None => storage
            .remove_item(key)
            .map_err(|_| EnvError::StorageUnavailable)?,
    };
    Ok(())
}

fn js_error_message(error: JsValue) -> String {
    error
        .dyn_into::<js_sys::Error>()
        .map(|error| String::from(error.message()))
        .unwrap_or_else(|_| "Unknown Error".to_owned())
}
