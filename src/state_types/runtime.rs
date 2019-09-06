use crate::state_types::*;
use derivative::*;
use enclose::*;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{future, Future};
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize)]
pub enum RuntimeEv {
    NewModel,
    Event(Event),
}

#[derive(Derivative)]
#[derivative(Debug, Clone(bound = ""))]
pub struct Runtime<Env: Environment, M: Update> {
    pub app: Arc<RwLock<M>>,
    tx: Sender<RuntimeEv>,
    env: PhantomData<Env>,
}
impl<Env: Environment + 'static, M: Update + 'static> Runtime<Env, M> {
    pub fn new(app: M, len: usize) -> (Self, Receiver<RuntimeEv>) {
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
    pub fn dispatch_with<T: FnOnce(&mut M) -> Effects>(&self, with: T) -> Box<dyn Future<Item = (), Error = ()>> {
        let handle = self.clone();
        let fx = with(&mut *self.app.write().expect("rwlock write failed"));
        // Send events
        {
            let mut tx = self.tx.clone();
            if fx.has_changed {
                let _ = tx.try_send(RuntimeEv::NewModel);
            }
        }
        // Handle next effects
        let all = fx.effects.into_iter().map(enclose!((handle) move |ft| ft
            .then(enclose!((handle) move |r| {
                let msg = match r {
                    Ok(msg) => msg,
                    Err(msg) => msg,
                };
                Env::exec(handle.dispatch(&msg));
                future::ok(())
            }))
        ));
        Box::new(futures::future::join_all(all).map(|_| ()))
    }
    pub fn dispatch(&self, msg: &Msg) -> Box<dyn Future<Item = (), Error = ()>> {
        {
            let mut tx = self.tx.clone();
            if let Msg::Event(ev) = msg {
                let _ = tx.try_send(RuntimeEv::Event(ev.to_owned()));
            }
        }
        self.dispatch_with(|model| model.update(msg))
    }
}
