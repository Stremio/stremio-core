use crate::state_types::msg::{Event, Msg};
use crate::state_types::{Effects, Environment, Update};
use core::pin::Pin;
use derivative::Derivative;
use enclose::enclose;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::{future, Future, FutureExt};
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize)]
#[serde(tag = "name", content = "args")]
pub enum RuntimeEvent {
    NewState,
    Event(Event),
}

#[derive(Derivative)]
#[derivative(Debug, Clone(bound = ""))]
pub struct Runtime<Env: Environment, AppModel: Update> {
    pub app: Arc<RwLock<AppModel>>,
    tx: Sender<RuntimeEvent>,
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static, AppModel: Update + 'static> Runtime<Env, AppModel> {
    pub fn new(app: AppModel, len: usize) -> (Self, Receiver<RuntimeEvent>) {
        let (tx, rx) = channel(len);
        let app = Arc::new(RwLock::new(app));
        (
            Runtime {
                app,
                tx,
                env: PhantomData,
            },
            rx,
        )
    }
    pub fn dispatch_with<T: FnOnce(&mut AppModel) -> Effects>(
        &self,
        with: T,
    ) -> Pin<Box<dyn Future<Output = ()> + Unpin>> {
        let handle = self.clone();
        let fx = with(&mut *self.app.write().expect("rwlock write failed"));
        // Send events
        {
            let mut tx = self.tx.clone();
            if fx.has_changed {
                let _ = tx.try_send(RuntimeEvent::NewState);
            }
        }
        // Handle next effects
        let all = fx.effects.into_iter().map(enclose!((handle) move |ft| ft
            .then(enclose!((handle) move |msg| {
                Env::exec(handle.dispatch(&msg));
                future::ready(())
            }))
        ));
        Pin::new(Box::new(futures::future::join_all(all).map(|_| ())))
    }
    pub fn dispatch(&self, msg: &Msg) -> Pin<Box<dyn Future<Output = ()> + Unpin>> {
        {
            let mut tx = self.tx.clone();
            if let Msg::Event(ev) = msg {
                let _ = tx.try_send(RuntimeEvent::Event(ev.to_owned()));
            }
        }
        self.dispatch_with(|model| model.update(msg))
    }
}
