//! # Runtime - The Core Event Loop
//!
//! The [`Runtime`] is the central coordinator for stremio-core's architecture.
//! It manages the application state ([`Model`]) and processes user actions through
//! a message-passing system.
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────┐     dispatch()     ┌─────────────────┐
//! │   UI / Client   │ ─────────────────▶ │     Runtime     │
//! └─────────────────┘                    └────────┬────────┘
//!         ▲                                       │
//!         │  RuntimeEvent::NewState               │ update()
//!         │                                       ▼
//! ┌───────┴─────────┐                    ┌─────────────────┐
//! │   Rx (Receiver) │ ◀───────────────── │      Model      │
//! └─────────────────┘                    └─────────────────┘
//! ```
//!
//! ## Important Gotchas
//!
//! 1. **The `dispatch()` method executes effects immediately**: When you call
//!    [`Runtime::dispatch`], it synchronously updates the model and spawns any
//!    resulting effect futures. Make sure your [`Env`] is properly configured
//!    to handle async execution.
//!
//! 2. **The receiver never closes automatically**: The [`Receiver`] from
//!    [`Runtime::new`] stays open as long as the Runtime exists. Drop the
//!    Runtime to close the channel.
//!
//! 3. **Effects must not resolve to Actions**: Effects can only produce
//!    [`Event`] or [`Internal`](crate::runtime::msg::Internal) messages.
//!    Attempting to dispatch an Action from an effect will panic.
//!
//! ## Usage Example
//!
//! ```ignore
//! // Create runtime with initial model
//! let (runtime, rx) = Runtime::<MyEnv, MyModel>::new(model, effects, 1000);
//!
//! // Dispatch user actions
//! runtime.dispatch(RuntimeAction { field: None, action: my_action });
//!
//! // Listen for state changes
//! while let Some(event) = rx.recv().await {
//!     match event {
//!         RuntimeEvent::NewState(fields, ..) => { /* re-render UI */ }
//!         RuntimeEvent::CoreEvent(event) => { /* handle event */ }
//!     }
//! }
//! ```
//!
//! [`Model`]: crate::runtime::Model
//! [`Env`]: crate::runtime::Env

use crate::runtime::msg::{Action, Event, Msg};
use crate::runtime::{Effect, EffectFuture, Env, Model};
use derivative::Derivative;
use enclose::enclose;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::FutureExt;
#[cfg(test)]
use futures::SinkExt;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard};

/// Events emitted by the [`Runtime`] to notify clients of state changes.
///
/// These events are sent through the channel returned by [`Runtime::new`].
#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(tag = "name", content = "args")]
pub enum RuntimeEvent<E: Env, M: Model<E>> {
    /// The model state has changed. Contains the list of fields that were modified.
    NewState(Vec<M::Field>, #[cfg(test)] M),
    /// A core event was emitted (e.g., error, notification, or other internal event).
    CoreEvent(Event),
}

/// An action to be dispatched to the [`Runtime`].
///
/// # Fields
/// - `field`: Optional field to target for partial model updates
/// - `action`: The user action to process
#[derive(Debug)]
pub struct RuntimeAction<E: Env, M: Model<E>> {
    /// Optional field for targeted updates. If `None`, the entire model is updated.
    pub field: Option<M::Field>,
    /// The action to dispatch (e.g., load content, play media, update settings).
    pub action: Action,
}

/// The core application runtime that manages state and processes actions.
///
/// Generic over:
/// - `E`: The environment providing async execution and storage capabilities
/// - `M`: The application model containing all state
///
/// See the [module-level documentation](self) for usage details and gotchas.
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct Runtime<E: Env, M: Model<E>> {
    model: Arc<RwLock<M>>,
    tx: Sender<RuntimeEvent<E, M>>,
    env: PhantomData<E>,
}

impl<E, M> Runtime<E, M>
where
    E: Env + Send + 'static,
    M: Model<E> + Send + Sync + 'static,
{
    pub fn new(
        model: M,
        effects: Vec<Effect>,
        buffer: usize,
    ) -> (Self, Receiver<RuntimeEvent<E, M>>) {
        let (tx, rx) = channel(buffer);
        let model = Arc::new(RwLock::new(model));
        let runtime = Runtime {
            model,
            tx,
            env: PhantomData,
        };
        runtime.handle_effects(effects, vec![]);
        (runtime, rx)
    }
    pub fn model(&self) -> LockResult<RwLockReadGuard<'_, M>> {
        self.model.read()
    }
    pub fn dispatch(&self, action: RuntimeAction<E, M>) {
        let (effects, fields) = {
            let mut model = self.model.write().expect("model write failed");
            match action {
                RuntimeAction {
                    field: Some(field),
                    action,
                } => model.update_field(&Msg::Action(action), &field),
                RuntimeAction { action, .. } => model.update(&Msg::Action(action)),
            }
        };
        self.handle_effects(effects, fields);
    }
    #[cfg(test)]
    pub async fn close(&mut self) -> Result<(), anyhow::Error> {
        self.tx.flush().await?;
        self.tx.close_channel();
        Ok(())
    }
    fn emit(&self, event: RuntimeEvent<E, M>) {
        self.tx.clone().try_send(event).expect("emit event failed");
    }
    fn handle_effects(&self, effects: Vec<Effect>, fields: Vec<M::Field>) {
        if !fields.is_empty() {
            #[cfg(test)]
            let model = self.model.read().expect("model read failed");
            self.emit(RuntimeEvent::<E, M>::NewState(
                fields,
                #[cfg(test)]
                model.to_owned(),
            ));
        };
        effects
            .into_iter()
            .for_each(enclose!((self.clone() => runtime) move |effect| {
                match effect {
                    Effect::Msg(msg) => {
                        runtime.handle_effect_output(*msg);
                    }
                    Effect::Future(EffectFuture::Sequential(future)) => {
                        E::exec_sequential(future.then(enclose!((runtime) move |msg| async move {
                            runtime.handle_effect_output(msg);
                        })))
                    },
                    Effect::Future(EffectFuture::Concurrent(future)) => {
                        E::exec_concurrent(future.then(enclose!((runtime) move |msg| async move {
                            runtime.handle_effect_output(msg);
                        })))
                    }
                }
            }));
    }
    fn handle_effect_output(&self, msg: Msg) {
        match msg {
            Msg::Event(event) => {
                self.emit(RuntimeEvent::CoreEvent(event));
            }
            Msg::Internal(_) => {
                let (effects, fields) =
                    self.model.write().expect("model write failed").update(&msg);
                self.handle_effects(effects, fields);
            }
            Msg::Action(_) => {
                panic!("effects are not allowed to resolve with action");
            }
        }
    }
}
