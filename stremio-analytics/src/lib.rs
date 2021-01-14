use derivative::Derivative;
use enclose::enclose;
use futures::{future, FutureExt};
use serde::Serialize;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use stremio_core::models::ctx::Ctx;
use stremio_core::runtime::{Env, EnvError, EnvFuture};
use stremio_core::types::api::{fetch_api, APIRequest, APIResult, SuccessResponse};
use stremio_core::types::profile::AuthKey;

#[derive(Clone, PartialEq, Serialize)]
struct Event {
    #[serde(flatten)]
    data: serde_json::Value,
    #[serde(rename = "eventName")]
    name: String,
    #[serde(rename = "eventTime")]
    time: i64,
    #[serde(rename = "eventNumber")]
    number: u64,
    #[serde(rename = "app")]
    context: serde_json::Value,
}

#[derive(Clone, PartialEq)]
struct EventsBatch {
    auth_key: AuthKey,
    events: Vec<Event>,
}

#[derive(Default)]
struct State {
    number: u64,
    queue: VecDeque<EventsBatch>,
    pending: Option<EventsBatch>,
}

impl State {
    fn next_number(&mut self) -> u64 {
        self.number = self.number.wrapping_add(1);
        self.number
    }
    fn pop_batch(&mut self) -> Option<EventsBatch> {
        self.queue.pop_front()
    }
    fn push_event(&mut self, event: Event, auth_key: AuthKey) {
        match self.queue.back_mut() {
            Some(batch) if batch.auth_key == auth_key => {
                batch.events.push(event);
            }
            _ => self.queue.push_back(EventsBatch {
                auth_key,
                events: vec![event],
            }),
        };
    }
    fn revert_pending(&mut self) {
        if let Some(batch) = self.pending.take() {
            self.queue.push_front(batch);
        };
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct Analytics<E: Env> {
    state: Arc<Mutex<State>>,
    env: PhantomData<E>,
}

impl<E: Env + 'static> Analytics<E> {
    pub fn emit(&self, name: String, data: serde_json::Value, ctx: &Ctx) {
        let mut state = self.state.lock().expect("analytics state lock failed");
        let auth_key = match ctx.profile.auth_key() {
            Some(auth_key) => auth_key.to_owned(),
            _ => return,
        };
        let event = Event {
            name,
            data,
            number: state.next_number(),
            time: E::now().timestamp_millis(),
            context: E::analytics_context(),
        };
        state.push_event(event, auth_key);
    }
    pub async fn flush_next(&self) {
        let mut state = self.state.lock().expect("analytics state lock failed");
        if state.pending.is_some() {
            return;
        };
        let batch = state.pop_batch();
        if let Some(batch) = batch {
            state.pending = Some(batch.to_owned());
            drop(state);
            send_events_batch_to_api::<E>(&batch)
                .map(|result| match result {
                    Ok(APIResult::Err { error }) if error.code != 1 => Err(()),
                    Err(EnvError::Fetch(_)) | Err(EnvError::Serde(_)) => Err(()),
                    _ => Ok(()),
                })
                .then(enclose!((self.state => state) move |result| async move {
                    let mut state = state.lock().expect("analytics state lock failed");
                    if state.pending == Some(batch) {
                        match result {
                            Ok(_) => {
                                state.pending = None;
                            }
                            Err(_) => {
                                state.revert_pending();
                            }
                        };
                    };
                }))
                .await;
        };
    }
    pub async fn flush_all(&self) {
        let mut state = self.state.lock().expect("analytics state lock failed");
        state.pending = None;
        let batches = state.queue.drain(..).collect::<Vec<_>>();
        drop(state);
        future::join_all(
            batches
                .iter()
                .map(|batch| send_events_batch_to_api::<E>(&batch)),
        )
        .await;
    }
}

fn send_events_batch_to_api<E: Env>(batch: &EventsBatch) -> EnvFuture<APIResult<SuccessResponse>> {
    fetch_api::<E, _, _>(&APIRequest::Events {
        auth_key: batch.auth_key.to_owned(),
        events: batch
            .events
            .iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect(),
    })
}