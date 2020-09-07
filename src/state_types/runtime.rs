use crate::state_types::msg::{Event, Msg};
use crate::state_types::{Effects, Environment, Update};
use core::pin::Pin;
use derivative::Derivative;
use enclose::enclose;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::{future, Future, FutureExt};
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard};

#[derive(Debug, Serialize)]
#[serde(tag = "name", content = "args")]
pub enum RuntimeEvent {
    NewState,
    Event(Event),
}

#[derive(Derivative)]
#[derivative(Debug, Clone(bound = ""))]
pub struct Runtime<Env: Environment, App: Update> {
    app: Arc<RwLock<App>>,
    tx: Sender<RuntimeEvent>,
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static, App: Update + 'static> Runtime<Env, App> {
    pub fn new(app: App, buffer: usize) -> (Self, Receiver<RuntimeEvent>) {
        let (tx, rx) = channel(buffer);
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
    pub fn app(&self) -> LockResult<RwLockReadGuard<App>> {
        self.app.read()
    }
    pub fn dispatch_with<T: FnOnce(&mut App) -> Effects>(
        &self,
        with: T,
    ) -> Pin<Box<dyn Future<Output = ()> + Unpin>> {
        let handle = self.clone();
        let fx = with(&mut *self.app.write().expect("rwlock write failed"));
        if fx.has_changed {
            self.emit_event(RuntimeEvent::NewState);
        };
        // Handle next effects
        let all = fx.into_iter().map(enclose!((handle) move |ft| ft
            .then(enclose!((handle) move |msg| {
                Env::exec(handle.dispatch(&msg));
                future::ready(())
            }))
        ));
        Box::pin(futures::future::join_all(all).map(|_| ()))
    }
    pub fn dispatch(&self, msg: &Msg) -> Pin<Box<dyn Future<Output = ()> + Unpin>> {
        if let Msg::Event(event) = msg {
            self.emit_event(RuntimeEvent::Event(event.to_owned()));
        };
        self.dispatch_with(|model| model.update(msg))
    }
    fn emit_event(&self, event: RuntimeEvent) {
        self.tx.clone().try_send(event).unwrap();
    }
}
