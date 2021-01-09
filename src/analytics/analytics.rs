use crate::env::WebEnv;
use crate::event::WebEvent;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use stremio_core::models::ctx::Ctx;
use stremio_core::runtime::msg::Event;
#[cfg(debug_assertions)]
use stremio_core::runtime::Env;
use stremio_core::types::profile::AuthKey;

#[derive(Clone, Serialize)]
pub struct AnalyticsContext {
    pub url: String,
}

#[derive(Clone, Serialize)]
pub struct AnalyticsEvent {
    pub name: String,
    pub context: AnalyticsContext,
}

pub struct AnalyticsState {
    pub auth_key: Option<AuthKey>,
    pub events: Vec<AnalyticsEvent>,
}

#[derive(Clone)]
pub struct Analytics {
    visit_id: String,
    state: Arc<RwLock<AnalyticsState>>,
}

impl Analytics {
    pub fn new(visit_id: String) -> Self {
        Self {
            visit_id,
            state: Arc::new(RwLock::new(AnalyticsState {
                auth_key: None,
                events: vec![],
            })),
        }
    }
    pub fn emit(&self, event: WebEvent, ctx: &Ctx<WebEnv>) {
        let auth_key = ctx.profile.auth_key();
        if self.auth_changed(auth_key) {
            self.flush();
        };
        let mut state = self.state.write().expect("analytics state write failed");
        state.auth_key = auth_key.cloned();
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
        state.events.push(event);
    }
    fn auth_changed(&self, auth_key: Option<&AuthKey>) -> bool {
        let state = self.state.read().expect("analytics state read failed");
        state.auth_key.as_ref() != auth_key
    }
    fn flush(&self) {
        let mut state = self.state.write().expect("analytics state write failed");
        let events = state.events.drain(..).collect::<Vec<_>>();
        #[cfg(debug_assertions)]
        WebEnv::log(format!(
            "AuthKey: {}, Events: {}",
            serde_json::to_string(&state.auth_key).unwrap(),
            serde_json::to_string(&events).unwrap()
        ));
    }
}
