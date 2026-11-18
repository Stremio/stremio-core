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

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(tag = "name", content = "args")]
pub enum RuntimeEvent<E: Env, M: Model<E>> {
    NewState(Vec<M::Field>, #[cfg(test)] M),
    CoreEvent(Event),
}

#[derive(Debug)]
pub struct RuntimeAction<E: Env, M: Model<E>> {
    pub field: Option<M::Field>,
    pub action: Action,
}

/// Runs a model and publishes state changes and core events.
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
    /// Creates a runtime, starts the initial effects, and returns its event receiver.
    ///
    /// Keep polling the receiver while the runtime can emit events. The stream
    /// ends once all runtime handles have been dropped and buffered events have
    /// been read. Pending effects hold their own runtime clones, so dropping the
    /// caller's handle does not cancel those effects or immediately end the stream.
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
    /// Applies the action synchronously and sends any state change notification.
    ///
    /// Immediate effects are handled inline. Future effects are scheduled through
    /// [`Env::exec_sequential`] or [`Env::exec_concurrent`]. This method returns
    /// `()`.
    ///
    /// # Panics
    ///
    /// Panics if an event emitted during dispatch cannot be sent because the
    /// receiver is closed or the event channel is full.
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
