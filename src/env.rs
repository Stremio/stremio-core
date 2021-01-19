use crate::event::WebEvent;
use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
use futures::future::Either;
use futures::{future, Future, FutureExt, TryFutureExt};
use http::{Method, Request};
use lazy_static::lazy_static;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::RwLock;
use stremio_analytics::Analytics;
use stremio_core::models::ctx::Ctx;
use stremio_core::runtime::msg::{Action, ActionCtx, Event};
use stremio_core::runtime::{Env, EnvError, EnvFuture, TryEnvFuture};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};

const INSTALLATION_ID_STORAGE_KEY: &str = "installation_id";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "core_imports"])]
    static app_version: String;
    #[wasm_bindgen(catch, js_namespace = ["window", "core_imports"])]
    fn sanitize_location_path(path: &str) -> Result<String, JsValue>;
}

lazy_static! {
    static ref INSTALLATION_ID: RwLock<Option<String>> = Default::default();
    static ref VISIT_ID: String = hex::encode(WebEnv::random_buffer(10));
    static ref ANALYTICS: Analytics<WebEnv> = Default::default();
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsContext {
    app_type: String,
    app_version: Version,
    server_version: Option<Version>,
    shell_version: Option<Version>,
    system_language: String,
    app_language: String,
    #[serde(rename = "installationID")]
    installation_id: String,
    #[serde(rename = "visitID")]
    visit_id: String,
    url: String,
}

pub enum WebEnv {}

impl WebEnv {
    pub fn init() -> TryEnvFuture<()> {
        WebEnv::migrate_storage_schema()
            .and_then(|_| WebEnv::get_storage::<String>(INSTALLATION_ID_STORAGE_KEY))
            .map_ok(|installation_id| {
                installation_id.or_else(|| Some(hex::encode(WebEnv::random_buffer(10))))
            })
            .and_then(|installation_id| {
                *INSTALLATION_ID
                    .write()
                    .expect("installation id write failed") = installation_id;
                WebEnv::set_storage(
                    INSTALLATION_ID_STORAGE_KEY,
                    Some(&*INSTALLATION_ID.read().expect("installation id read failed")),
                )
            })
            .boxed_local()
    }
    pub fn emit_to_analytics(event: WebEvent, ctx: &Ctx) {
        let (name, data) = match event {
            WebEvent::CoreEvent(Event::UserAuthenticated { .. }) => (
                "login".to_owned(),
                json!({
                    "data": "data",
                }),
            ),
            WebEvent::CoreAction(Action::Ctx(ActionCtx::Logout)) => (
                "logout".to_owned(),
                json!({
                    "data": "data",
                }),
            ),
            _ => return,
        };
        ANALYTICS.emit(name, data, ctx);
    }
    pub fn send_next_analytics_batch() -> impl Future<Output = ()> {
        ANALYTICS.send_next_batch()
    }
    pub fn set_interval<F: FnMut() + 'static>(func: F, timeout: i32) -> i32 {
        let closure = Closure::wrap(Box::new(func) as Box<dyn FnMut()>);
        let interval_id = web_sys::window()
            .expect("window is not available")
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                timeout,
            )
            .expect("set interval failed");
        closure.forget();
        interval_id
    }
    #[allow(dead_code)]
    pub fn clear_interval(id: i32) {
        web_sys::window()
            .expect("window is not available")
            .clear_interval_with_handle(id);
    }
    pub fn random_buffer(len: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; len];
        getrandom::getrandom(buffer.as_mut_slice()).expect("generate random buffer failed");
        buffer
    }
}

impl Env for WebEnv {
    fn fetch<IN, OUT>(request: Request<IN>) -> TryEnvFuture<OUT>
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
    fn get_storage<T>(key: &str) -> TryEnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        future::ready(get_storage_sync(key)).boxed_local()
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()> {
        future::ready(set_storage_sync(key, value)).boxed_local()
    }
    fn exec<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future)
    }
    fn now() -> DateTime<Utc> {
        let msecs = js_sys::Date::now() as i64;
        let (secs, nsecs) = (msecs / 1000, msecs % 1000 * 1_000_000);
        Utc.timestamp(secs, nsecs as u32)
    }
    fn flush_analytics() -> EnvFuture<()> {
        ANALYTICS.flush().boxed_local()
    }
    fn analytics_context() -> serde_json::Value {
        let location_hash = web_sys::window()
            .expect("window is not available")
            .location()
            .hash()
            .expect("location hash is not available");
        let hash_path = location_hash.split('#').last().unwrap_or_default();
        let url = sanitize_location_path(hash_path).expect("sanitize location path failed");
        json!({
            "url": url,
            "visit_id": &*VISIT_ID,
            "app_version": &*app_version
        })
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
