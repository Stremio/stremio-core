use crate::env::WebEnv;
use crate::event::WebEvent;
use enclose::enclose;
use futures::FutureExt;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use stremio_core::models::ctx::Ctx;
use stremio_core::runtime::msg::Event;
use stremio_core::runtime::{Env, EnvError, EnvFuture};
use stremio_core::types::api::{fetch_api, APIRequest, APIResult, SuccessResponse};
use stremio_core::types::profile::AuthKey;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[derive(Serialize)]
struct AnalyticsContext {
    url: String,
}

#[derive(Serialize)]
struct AnalyticsEvent {
    name: String,
    context: AnalyticsContext,
}

struct AnalyticsEventsBatch {
    events: Vec<AnalyticsEvent>,
    auth_key: Option<AuthKey>,
}

pub struct Analytics {
    queue: Arc<Mutex<VecDeque<AnalyticsEventsBatch>>>,
    flush_interval_id: i32,
}

impl Analytics {
    pub fn new(_visit_id: String) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        #[allow(clippy::mutex_atomic)]
        let pending = Arc::new(Mutex::new(false));
        let closure = Closure::wrap(Box::new(enclose!((queue, pending) move || {
            flush_events(queue.clone(), pending.clone());
        })) as Box<dyn FnMut()>);
        let flush_interval_id = web_sys::window()
            .expect("window is not available")
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                30 * 1000,
            )
            .expect("set_interval failed");
        closure.forget();
        Self {
            queue,
            flush_interval_id,
        }
    }
    pub fn emit(&self, event: WebEvent, ctx: &Ctx<WebEnv>) {
        let context = AnalyticsContext {
            url: web_sys::window()
                .expect("window is not available")
                .location()
                .href()
                .expect("href is not available"),
        };
        let event = match event {
            WebEvent::CoreEvent(Event::UserAuthenticated { .. }) => AnalyticsEvent {
                name: "login".to_owned(),
                context,
            },
            _ => return,
        };
        let mut queue_value = self.queue.lock().expect("queue lock failed");
        let auth_key = ctx.profile.auth_key().cloned();
        match queue_value.back_mut() {
            Some(events_batch) if events_batch.auth_key == auth_key => {
                events_batch.events.push(event);
            }
            _ => queue_value.push_back(AnalyticsEventsBatch {
                auth_key,
                events: vec![event],
            }),
        };
    }
}

impl Drop for Analytics {
    fn drop(&mut self) {
        web_sys::window()
            .expect("window is not available")
            .clear_interval_with_handle(self.flush_interval_id);
    }
}

fn flush_events(queue: Arc<Mutex<VecDeque<AnalyticsEventsBatch>>>, pending: Arc<Mutex<bool>>) {
    let mut pending_value = pending.lock().expect("pending lock failed");
    if *pending_value {
        return;
    };
    let mut queue_value = queue.lock().expect("queue lock failed");
    if let Some(AnalyticsEventsBatch {
        events,
        auth_key: Some(auth_key),
    }) = queue_value.pop_front()
    {
        if !events.is_empty() {
            *pending_value = true;
            drop(pending_value);
            drop(queue_value);
            WebEnv::exec(
                send_events_to_api(&events, &auth_key)
                    .map(|result| match result {
                        Ok(APIResult::Err { error }) if error.code != 1 => Err(()), // session does not exist
                        Err(EnvError::Fetch(_)) | Err(EnvError::Serde(_)) => Err(()),
                        _ => Ok(()),
                    })
                    .then(enclose!((queue, pending) move |result| async move {
                        let mut pending_value = pending.lock().expect("pending lock failed");
                        *pending_value = false;
                        if result.is_err() {
                            let mut queue_value = queue.lock().expect("queue lock failed");
                            queue_value.push_front(AnalyticsEventsBatch {
                                events,
                                auth_key: Some(auth_key)
                            });
                        };
                    })),
            );
        };
    };
}

fn send_events_to_api(
    events: &[AnalyticsEvent],
    auth_key: &AuthKey,
) -> EnvFuture<APIResult<SuccessResponse>> {
    fetch_api::<WebEnv, _, _>(&APIRequest::Events {
        auth_key: auth_key.to_owned(),
        events: events
            .iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect(),
    })
}
