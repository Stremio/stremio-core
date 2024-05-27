use std::{collections::HashMap, sync::RwLock};

use chrono::{offset::TimeZone, DateTime, Utc};
use futures::{future, Future, FutureExt, TryFutureExt};
use gloo_utils::format::JsValueSerdeExt;
use http::{Method, Request};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use tracing::trace;
use url::Url;

use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::WorkerGlobalScope;

use stremio_core::{
    analytics::Analytics,
    models::{ctx::Ctx, streaming_server::StreamingServer},
    runtime::{
        msg::{Action, ActionCtx, Event},
        Env, EnvError, EnvFuture, EnvFutureExt, TryEnvFuture,
    },
    types::{api::AuthRequest, resource::StreamSource},
};

use crate::{
    event::{UIEvent, WebEvent},
    model::WebModel,
};

const UNKNOWN_ERROR: &str = "Unknown Error";
const INSTALLATION_ID_STORAGE_KEY: &str = "installation_id";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["self"], js_name = app_version)]
    static APP_VERSION: String;
    #[wasm_bindgen(js_namespace = ["self"], js_name = shell_version)]
    static SHELL_VERSION: Option<String>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn get_location_hash() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn local_storage_get_item(key: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn local_storage_set_item(key: String, value: String) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn local_storage_remove_item(key: String) -> Result<(), JsValue>;
}

lazy_static! {
    static ref INSTALLATION_ID: RwLock<Option<String>> = Default::default();
    static ref VISIT_ID: String = hex::encode(WebEnv::random_buffer(10));
    static ref ANALYTICS: Analytics<WebEnv> = Default::default();
    static ref PLAYER_REGEX: Regex =
        Regex::new(r"^/player/([^/]*)(?:/([^/]*)/([^/]*)/([^/]*)/([^/]*)/([^/]*))?$")
            .expect("Player Regex failed to build");
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsContext {
    app_type: String,
    app_version: String,
    server_version: Option<String>,
    shell_version: Option<String>,
    system_language: Option<String>,
    app_language: String,
    #[serde(rename = "installationID")]
    installation_id: String,
    #[serde(rename = "visitID")]
    visit_id: String,
    #[serde(rename = "url")]
    path: String,
}

pub enum WebEnv {}

impl WebEnv {
    /// Sets panic hook, enables logging
    pub fn init() -> TryEnvFuture<()> {
        WebEnv::migrate_storage_schema()
            .inspect(|migration_result| trace!("Migration result: {migration_result:?}",))
            .and_then(|_| WebEnv::get_storage::<String>(INSTALLATION_ID_STORAGE_KEY))
            .inspect(|installation_id_result| trace!("Migration: {installation_id_result:?}"))
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
            .inspect_ok(|_| {
                WebEnv::set_interval(
                    || WebEnv::exec_concurrent(WebEnv::send_next_analytics_batch()),
                    30 * 1000,
                );
            })
            .boxed_local()
    }
    pub fn get_location_hash() -> EnvFuture<'static, String> {
        get_location_hash()
            .map(|location_hash| {
                location_hash
                    .ok()
                    .and_then(|location_hash| location_hash.as_string())
                    .unwrap_or_default()
            })
            .boxed_env()
    }
    pub fn emit_to_analytics(event: &WebEvent, model: &WebModel, path: &str) {
        let (name, data) = match event {
            WebEvent::UIEvent(UIEvent::LocationPathChanged { prev_path }) => (
                "stateChange".to_owned(),
                json!({ "previousURL": sanitize_location_path(prev_path) }),
            ),
            WebEvent::UIEvent(UIEvent::Search {
                query,
                responses_count,
            }) => (
                "search".to_owned(),
                json!({ "query": query, "rows": responses_count }),
            ),
            WebEvent::UIEvent(UIEvent::Share { url }) => {
                ("share".to_owned(), json!({ "url": url }))
            }
            WebEvent::UIEvent(UIEvent::StreamClicked { stream }) => (
                "streamClicked".to_owned(),
                json!({
                    "type": match &stream.source {
                        StreamSource::Url { .. } => "Url",
                        StreamSource::YouTube { .. } => "YouTube",
                        StreamSource::Torrent { .. } => "Torrent",
                        StreamSource::Rar { .. } => "Rar",
                        StreamSource::Zip { .. } => "Zip",
                        StreamSource::External { .. } => "External",
                        StreamSource::PlayerFrame { .. } => "PlayerFrame"
                    }
                }),
            ),
            WebEvent::CoreEvent(core_event) => match core_event.as_ref() {
                Event::UserAuthenticated { auth_request } => (
                    "login".to_owned(),
                    json!({
                        "type": match auth_request {
                            AuthRequest::Login { facebook, .. } if *facebook => "facebook",
                            AuthRequest::Login { .. } => "login",
                            AuthRequest::LoginWithToken { .. } => "loginWithToken",
                            AuthRequest::Register { .. } => "register",
                        },
                    }),
                ),
                Event::AddonInstalled { transport_url, id } => (
                    "installAddon".to_owned(),
                    json!({
                        "addonTransportUrl": transport_url,
                        "addonID": id
                    }),
                ),
                Event::AddonUninstalled { transport_url, id } => (
                    "removeAddon".to_owned(),
                    json!({
                        "addonTransportUrl": transport_url,
                        "addonID": id
                    }),
                ),
                Event::PlayerPlaying { load_time, context } => (
                    "playerPlaying".to_owned(),
                    json!({
                        "loadTime": load_time,
                        "player": context
                    }),
                ),
                Event::PlayerStopped { context } => {
                    ("playerStopped".to_owned(), json!({ "player": context }))
                }
                Event::PlayerEnded {
                    context,
                    is_binge_enabled,
                    is_playing_next_video,
                } => (
                    "playerEnded".to_owned(),
                    json!({
                       "player": context,
                       "isBingeEnabled": is_binge_enabled,
                       "isPlayingNextVideo": is_playing_next_video
                    }),
                ),
                Event::TraktPlaying { context } => {
                    ("traktPlaying".to_owned(), json!({ "player": context }))
                }
                Event::TraktPaused { context } => {
                    ("traktPaused".to_owned(), json!({ "player": context }))
                }
                _ => return,
            },
            WebEvent::CoreAction(core_action) => match core_action.as_ref() {
                Action::Ctx(ActionCtx::AddToLibrary(meta_preview)) => {
                    let library_item = model.ctx.library.items.get(&meta_preview.id);
                    (
                        "addToLib".to_owned(),
                        json!({
                            "libItemID":  &meta_preview.id,
                            "libItemType": &meta_preview.r#type,
                            "libItemName": &meta_preview.name,
                            "wasTemp": library_item.map(|library_item| library_item.temp).unwrap_or_default(),
                            "isReadded": library_item.map(|library_item| library_item.removed).unwrap_or_default(),
                        }),
                    )
                }
                Action::Ctx(ActionCtx::RemoveFromLibrary(id)) => {
                    match model.ctx.library.items.get(id) {
                        Some(library_item) => (
                            "removeFromLib".to_owned(),
                            json!({
                                "libItemID":  &library_item.id,
                                "libItemType": &library_item.r#type,
                                "libItemName": &library_item.name,
                            }),
                        ),
                        _ => return,
                    }
                }
                Action::Ctx(ActionCtx::Logout) => ("logout".to_owned(), serde_json::Value::Null),
                _ => return,
            },
        };
        ANALYTICS.emit(name, data, &model.ctx, &model.streaming_server, path);
    }
    pub fn send_next_analytics_batch() -> impl Future<Output = ()> {
        ANALYTICS.send_next_batch()
    }
    pub fn set_interval<F: FnMut() + 'static>(func: F, timeout: i32) -> i32 {
        let func = Closure::wrap(Box::new(func) as Box<dyn FnMut()>);
        let interval_id = global()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                func.as_ref().unchecked_ref(),
                timeout,
            )
            .expect("set interval failed");
        func.forget();
        interval_id
    }
    #[allow(dead_code)]
    pub fn clear_interval(id: i32) {
        global().clear_interval_with_handle(id);
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
            <JsValue as JsValueSerdeExt>::from_serde(&headers)
                .expect("WebEnv::fetch: JsValue from Headers failed to be built")
        };
        let body = match serde_json::to_string(&body) {
            Ok(ref body) if body != "null" && parts.method != Method::GET => {
                Some(JsValue::from_str(body))
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
        let promise = global().fetch_with_request(&request);
        async {
            let resp = JsFuture::from(promise).await.map_err(|error| {
                EnvError::Fetch(
                    error
                        .dyn_into::<js_sys::Error>()
                        .map(|error| String::from(error.message()))
                        .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                )
            })?;

            let resp = resp
                .dyn_into::<web_sys::Response>()
                .expect("WebEnv::fetch: Response into web_sys::Response failed to be built");
            // status check and JSON extraction from response.
            let resp = if resp.status() != 200 {
                return Err(EnvError::Fetch(format!(
                    "Unexpected HTTP status code {}",
                    resp.status(),
                )));
            } else {
                // Response.json() to JSON::Stringify

                JsFuture::from(
                    resp.text()
                        .expect("WebEnv::fetch: Response text failed to be retrieved"),
                )
                .map_err(|error| {
                    EnvError::Fetch(
                        error
                            .dyn_into::<js_sys::Error>()
                            .map(|error| String::from(error.message()))
                            .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                    )
                })
                .await
                .and_then(|js_value| {
                    js_value.dyn_into::<js_sys::JsString>().map_err(|error| {
                        EnvError::Fetch(
                            error
                                .dyn_into::<js_sys::Error>()
                                .map(|error| String::from(error.message()))
                                .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                        )
                    })
                })?
            };

            response_deserialize(resp)
        }
        .boxed_local()
    }

    fn get_storage<T>(key: &str) -> TryEnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        local_storage_get_item(key.to_owned())
            .map_err(|error| {
                EnvError::StorageReadError(
                    error
                        .dyn_into::<js_sys::Error>()
                        .map(|error| String::from(error.message()))
                        .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                )
            })
            .and_then(|value| async move {
                value
                    .as_string()
                    .map(|value| serde_json::from_str(&value))
                    .transpose()
                    .map_err(EnvError::from)
            })
            .boxed_local()
    }

    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()> {
        let key = key.to_owned();
        match value {
            Some(value) => future::ready(serde_json::to_string(value))
                .map_err(EnvError::from)
                .and_then(|value| {
                    local_storage_set_item(key, value).map_err(|error| {
                        EnvError::StorageWriteError(
                            error
                                .dyn_into::<js_sys::Error>()
                                .map(|error| String::from(error.message()))
                                .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                        )
                    })
                })
                .boxed_local(),
            None => local_storage_remove_item(key)
                .map_err(|error| {
                    EnvError::StorageWriteError(
                        error
                            .dyn_into::<js_sys::Error>()
                            .map(|error| String::from(error.message()))
                            .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                    )
                })
                .boxed_local(),
        }
    }

    fn exec_concurrent<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future)
    }

    fn exec_sequential<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future)
    }

    fn now() -> DateTime<Utc> {
        let msecs = js_sys::Date::now() as i64;
        let (secs, nsecs) = (msecs / 1000, msecs % 1000 * 1_000_000);
        Utc.timestamp_opt(secs, nsecs as u32)
            .single()
            .expect("Invalid timestamp")
    }

    fn flush_analytics() -> EnvFuture<'static, ()> {
        ANALYTICS.flush().boxed_local()
    }

    fn analytics_context(
        ctx: &Ctx,
        streaming_server: &StreamingServer,
        path: &str,
    ) -> serde_json::Value {
        serde_json::to_value(AnalyticsContext {
            app_type: "stremio-web".to_owned(),
            app_version: APP_VERSION.to_owned(),
            server_version: streaming_server
                .settings
                .as_ref()
                .ready()
                .map(|settings| settings.server_version.to_owned()),
            shell_version: SHELL_VERSION.to_owned(),
            system_language: global()
                .navigator()
                .language()
                .map(|language| language.to_lowercase()),
            app_language: ctx.profile.settings.interface_language.to_owned(),
            installation_id: INSTALLATION_ID
                .read()
                .expect("installation id read failed")
                .as_ref()
                .expect("installation id not available")
                .to_owned(),
            visit_id: VISIT_ID.to_owned(),
            path: sanitize_location_path(path),
        })
        .expect("AnalyticsContext to JSON")
    }

    #[cfg(debug_assertions)]
    fn log(message: String) {
        web_sys::console::log_1(&JsValue::from(message));
    }
}

fn sanitize_location_path(path: &str) -> String {
    match Url::parse(&format!("stremio://{}", path)) {
        Ok(url) => {
            let query = url
                .query()
                .map(|query| format!("?{}", query))
                .unwrap_or_default();
            let path = match PLAYER_REGEX.captures(url.path()) {
                Some(captures) => {
                    match (
                        captures.get(3),
                        captures.get(4),
                        captures.get(5),
                        captures.get(6),
                    ) {
                        (Some(match_3), Some(match_4), Some(match_5), Some(match_6)) => {
                            format!(
                                "/player/***/***/{cap_3}/{cap_4}/{cap_5}/{cap_6}",
                                cap_3 = match_3.as_str(),
                                cap_4 = match_4.as_str(),
                                cap_5 = match_5.as_str(),
                                cap_6 = match_6.as_str(),
                            )
                        }
                        _ => "/player/***".to_owned(),
                    }
                }
                _ => url.path().to_owned(),
            };
            format!("{}{}", path, query)
        }
        _ => path.to_owned(),
    }
}

fn global() -> WorkerGlobalScope {
    js_sys::global()
        .dyn_into::<WorkerGlobalScope>()
        .expect("worker global scope is not available")
}

fn response_deserialize<OUT>(response: js_sys::JsString) -> Result<OUT, EnvError>
where
    for<'de> OUT: Deserialize<'de> + 'static,
{
    let response = Into::<String>::into(response);
    let mut deserializer = serde_json::Deserializer::from_str(response.as_str());

    // deserialize into the final OUT struct

    serde_path_to_error::deserialize::<_, OUT>(&mut deserializer)
        .map_err(|error| EnvError::Fetch(error.to_string()))
}

/// > One other difference is that the tests must be in the root of the crate,
/// > or within a pub mod. Putting them inside a private module will not work.
#[cfg(test)]
pub mod tests {
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use wasm_bindgen_test::wasm_bindgen_test;

    use stremio_core::{
        runtime::EnvError,
        types::{
            addon::ResourceResponse,
            api::{APIResult, CollectionResponse},
        },
    };

    use super::response_deserialize;

    #[wasm_bindgen_test]
    fn test_deserialization_path_error() {
        let json_string = serde_json::json!({
            "result": []
        })
        .to_string();
        let result = response_deserialize::<APIResult<Vec<String>>>(json_string.into());
        assert!(result.is_ok());

        // Bad ApiResult response, non-existing variant
        {
            let json_string = serde_json::json!({
                "unknown_variant": {"test": 1}
            })
            .to_string();
            let result = response_deserialize::<APIResult<CollectionResponse>>(json_string.into());

            assert_eq!(
                result.expect_err("Should be an error"),
                EnvError::Fetch("unknown variant `unknown_variant`, expected `error` or `result` at line 1 column 18".to_string()),
                "Message does not include the text 'unknown variant `unknown_variant`, expected `error` or `result` at line 1 column 18'"
            );
        }

        // Addon ResourceResponse error, bad variant values
        {
            let json_string = serde_json::json!({
                "metas": {"object_key": "value"}
            })
            .to_string();
            let result = response_deserialize::<ResourceResponse>(json_string.into());

            assert_eq!(
                result.expect_err("Should be an error"),
                EnvError::Fetch("invalid type: map, expected a sequence".to_string()),
                "Message does not include the text 'Cannot deserialize as ResourceResponse'"
            );
        }
    }
}
