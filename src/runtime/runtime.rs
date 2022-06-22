use crate::runtime::msg::{Action, Event, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, Model};
use derivative::Derivative;
use enclose::enclose;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::FutureExt;
#[cfg(test)]
use futures::SinkExt;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard};

#[derive(Serialize)]
#[serde(tag = "name", content = "args")]
pub enum RuntimeEvent {
    NewState,
    CoreEvent(Event),
}

pub struct RuntimeAction<E: Env, M: Model<E>> {
    pub field: Option<M::Field>,
    pub action: Action,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct Runtime<E: Env, M: Model<E>> {
    model: Arc<RwLock<M>>,
    tx: Sender<RuntimeEvent>,
    env: PhantomData<E>,
}

impl<E, M> Runtime<E, M>
where
    E: Env + Send + 'static,
    M: Model<E> + Send + Sync + 'static,
{
    pub fn new(model: M, effects: Effects, buffer: usize) -> (Self, Receiver<RuntimeEvent>) {
        let (tx, rx) = channel(buffer);
        let model = Arc::new(RwLock::new(model));
        let runtime = Runtime {
            model,
            tx,
            env: PhantomData,
        };
        runtime.handle_effects(effects);
        (runtime, rx)
    }
    pub fn model(&self) -> LockResult<RwLockReadGuard<M>> {
        self.model.read()
    }
    pub fn dispatch(&self, action: RuntimeAction<E, M>) {
        let effects = {
            let mut model = self.model.write().expect("model write failed");
            match action {
                RuntimeAction {
                    field: Some(field),
                    action,
                } => model.update_field(&Msg::Action(action), &field),
                RuntimeAction { action, .. } => model.update(&Msg::Action(action)),
            }
        };
        self.handle_effects(effects);
    }
    #[cfg(test)]
    pub async fn close(&mut self) -> Result<(), anyhow::Error> {
        self.tx.flush().await?;
        self.tx.close_channel();
        Ok(())
    }
    fn emit(&self, event: RuntimeEvent) {
        self.tx.clone().try_send(event).expect("emit event failed");
    }
    fn handle_effects(&self, effects: Effects) {
        if effects.has_changed {
            self.emit(RuntimeEvent::NewState);
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
                let effects = self.model.write().expect("model write failed").update(&msg);
                self.handle_effects(effects);
            }
            Msg::Action(_) => {
                panic!("effects are not allowed to resolve with action");
            }
        }
    }
}
