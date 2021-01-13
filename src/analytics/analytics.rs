use derivative::Derivative;
use enclose::enclose;
use futures::FutureExt;
use serde::Serialize;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use stremio_core::models::ctx::Ctx;
use stremio_core::runtime::{Env, EnvError, EnvFuture};
use stremio_core::types::api::{fetch_api, APIRequest, APIResult, SuccessResponse};
use stremio_core::types::profile::AuthKey;

#[derive(Clone, PartialEq, Serialize)]
struct AnalyticsEvent {
    #[serde(flatten)]
    event: serde_json::Value,
    context: serde_json::Value,
}

#[derive(Clone, PartialEq)]
struct AnalyticsEventsBatch {
    auth_key: AuthKey,
    events: Vec<AnalyticsEvent>,
}

#[derive(Derivative)]
#[derivative(Default, Clone(bound = ""))]
pub struct Analytics<E: Env> {
    queue: Arc<Mutex<VecDeque<AnalyticsEventsBatch>>>,
    pending: Arc<Mutex<Option<AnalyticsEventsBatch>>>,
    env: PhantomData<E>,
}

impl<E: Env + 'static> Analytics<E> {
    pub fn emit(&self, event: serde_json::Value, ctx: &Ctx) {
        let auth_key = match ctx.profile.auth_key() {
            Some(auth_key) => auth_key.to_owned(),
            _ => return,
        };
        let event = AnalyticsEvent {
            event,
            context: E::analytics_context(),
        };
        let mut queue = self.queue.lock().expect("queue lock failed");
        match queue.back_mut() {
            Some(batch) if batch.auth_key == auth_key => {
                batch.events.push(event);
            }
            _ => queue.push_back(AnalyticsEventsBatch {
                auth_key,
                events: vec![event],
            }),
        };
    }
    pub fn flush_next_batch(&self) {
        let mut pending = self.pending.lock().expect("pending lock failed");
        if pending.is_some() {
            return;
        };
        let mut queue = self.queue.lock().expect("queue lock failed");
        if let Some(batch) = queue.pop_front() {
            *pending = Some(batch.to_owned());
            E::exec(
                send_events_batch_to_api::<E>(&batch)
                    .map(|result| match result {
                        Ok(APIResult::Err { error }) if error.code != 1 => Err(()),
                        Err(EnvError::Fetch(_)) | Err(EnvError::Serde(_)) => Err(()),
                        _ => Ok(()),
                    })
                    .then(enclose!((self.clone() => analytics) move |result| async move {
                        let mut pending = analytics.pending.lock().expect("pending lock failed");
                        match &*pending {
                            Some(pending_batch) if *pending_batch == batch => {
                                *pending = None;
                            },
                            _ => {
                                return;
                            }
                        };
                        if result.is_err() {
                            let mut queue = analytics.queue.lock().expect("queue lock failed");
                            queue.push_front(batch);
                        };
                    })),
            );
        };
    }
    pub fn flush_all(&self) {
        loop {
            {
                let mut pending = self.pending.lock().expect("pending lock failed");
                *pending = None;
            }
            self.flush_next_batch();
            {
                let queue = self.queue.lock().expect("queue lock failed");
                if queue.is_empty() {
                    break;
                }
            }
        }
    }
}

fn send_events_batch_to_api<E: Env>(
    batch: &AnalyticsEventsBatch,
) -> EnvFuture<APIResult<SuccessResponse>> {
    fetch_api::<E, _, _>(&APIRequest::Events {
        auth_key: batch.auth_key.to_owned(),
        events: batch
            .events
            .iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect(),
    })
}
