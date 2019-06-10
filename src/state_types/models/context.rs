use crate::types::addons::Descriptor;
use crate::types::api::{AuthKey, User};
use crate::state_types::*;
use std::marker::PhantomData;
use futures::future;
use lazy_static::*;
use serde_derive::*;

lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(include_bytes!("../../../stremio-official-addons/index.json"))
            .expect("official addons JSON parse");
}

// These will be stored, so they need to implement both Serialize and Deserilaize
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Auth {
    key: AuthKey,
    pub user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CtxContent {
    pub auth: Option<Auth>,
    pub addons: Vec<Descriptor>,
}
impl Default for CtxContent {
    fn default() -> Self {
        CtxContent {
            auth: None,
            addons: DEFAULT_ADDONS.to_owned(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Ctx<Env: Environment> {
    pub content: CtxContent,
    // Whether it's loaded from storage
    pub is_loaded: bool,
    env: PhantomData<Env>,
}

impl<Env: Environment> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::LoadCtx) => {
                Effects::one(load_storage_effect::<Env>()).unchanged()
            },
            Msg::Internal(Internal::CtxLoaded(Some(content))) => {
                self.is_loaded = true;
                self.content = *content.to_owned();
                Effects::none()
            }
            Msg::Internal(Internal::CtxLoaded(None)) => {
                self.is_loaded = true;
                Effects::none()
            }
            Msg::Action(Action::AddonOp(ActionAddon::Remove{ transport_url })) => {
                if let Some(idx) = self.content.addons.iter().position(|x| x.transport_url == *transport_url) {
                    self.content.addons.remove(idx);
                    // @TODO save
                    Effects::none()
                } else {
                    Effects::none()
                }
            }
            Msg::Action(Action::AddonOp(ActionAddon::Install(descriptor))) => {
                // @TODO should we dedupe?
                self.content.addons.push(*descriptor.to_owned());
                // @TODO save
                Effects::none()
            }
            _ => Effects::none().unchanged()
        }
    }
}

// @TODO move these load/save?
const USER_DATA_KEY: &str = "userData";
fn load_storage_effect<Env: Environment>() -> Effect {
    Box::new(Env::get_storage(USER_DATA_KEY)
        .then(move |res: Result<Option<Box<CtxContent>>, _>| {
            match res {
                Ok(x) => future::ok(Msg::Internal(Internal::CtxLoaded(x))),
                Err(e) => future::err(Msg::Event(Event::ContextMiddlewareFatal(e.into())))
            }
        }))
}
