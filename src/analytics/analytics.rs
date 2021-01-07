use crate::env::WebEnv;
use crate::ui_event::UIEvent;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use stremio_core::models::ctx::Ctx;
use stremio_core::runtime::msg::Event;
#[cfg(debug_assertions)]
use stremio_core::runtime::Env;
use stremio_core::types::profile::AuthKey;

pub enum StremioEvent {
    CoreEvent(Event),
    UIEvent(UIEvent),
}

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
    pub visit_id: String,
    pub auth_key: Option<AuthKey>,
    pub events: Vec<AnalyticsEvent>,
}

#[derive(Clone)]
pub struct Analytics {
    state: Arc<RwLock<AnalyticsState>>,
}

impl Analytics {
    pub fn new(visit_id: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(AnalyticsState {
                visit_id,
                auth_key: None,
                events: vec![],
            })),
        }
    }
    pub fn emit(&mut self, event: StremioEvent, _ctx: &Ctx<WebEnv>) {
        let event = match event {
            StremioEvent::CoreEvent(Event::UserAuthenticated { .. }) => AnalyticsEvent {
                name: "login".to_owned(),
                context: AnalyticsContext {
                    url: web_sys::window()
                        .expect("window is not available")
                        .location()
                        .href()
                        .unwrap(),
                },
            },
            _ => return,
        };
        #[cfg(debug_assertions)]
        WebEnv::log(serde_json::to_string(&event).unwrap());
    }
}
