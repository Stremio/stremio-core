use crate::state_types::Event::*;
use crate::state_types::Internal::*;
use crate::state_types::*;
use crate::types::addons::Descriptor;
use crate::types::api::*;
use derivative::*;
use futures::Future;
use lazy_static::*;
use serde_derive::*;
use std::marker::PhantomData;

const USER_DATA_KEY: &str = "userData";
lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> = serde_json::from_slice(include_bytes!(
        "../../../stremio-official-addons/index.json"
    ))
    .expect("official addons JSON parse");
}

// These will be stored, so they need to implement both Serialize and Deserilaize
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Auth {
    pub key: AuthKey,
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

#[derive(Derivative, Serialize)]
#[derivative(Debug, Default, Clone)]
pub struct Ctx<Env: Environment> {
    pub content: CtxContent,
    // Whether it's loaded from storage
    pub is_loaded: bool,
    #[serde(skip)]
    pub library: LibraryLoadable,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            // Loading from storage
            Msg::Action(Action::LoadCtx) => Effects::one(load_storage::<Env>()).unchanged(),
            Msg::Internal(CtxLoaded(opt_content)) => {
                self.content = opt_content
                    .as_ref()
                    .map(|x| *x.to_owned())
                    .unwrap_or(Default::default());
                self.is_loaded = true;
                // @TODO make this update library properly
                self.library = LibraryLoadable::NotLoaded;
                Effects::none()
            }
            // Addon install/remove
            Msg::Action(Action::AddonOp(ActionAddon::Remove { transport_url })) => {
                let pos = self
                    .content
                    .addons
                    .iter()
                    .position(|x| x.transport_url == *transport_url);
                if let Some(idx) = pos {
                    self.content.addons.remove(idx);
                    Effects::one(save_storage::<Env>(&self.content))
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::AddonOp(ActionAddon::Install(descriptor))) => {
                // @TODO should we dedupe?
                self.content.addons.push(*descriptor.to_owned());
                Effects::one(save_storage::<Env>(&self.content))
            }
            // User actions related to API primitives (authentication/addons)
            Msg::Action(Action::UserOp(action)) => match action.to_owned() {
                ActionUser::Logout => {
                    let new_content = Box::new(CtxContent::default());
                    match &self.content.auth {
                        Some(Auth { key, .. }) => {
                            let auth_key = key.to_owned();
                            let action = action.clone();
                            let effect = api_fetch::<Env, SuccessResponse, _>(APIRequest::Logout {
                                auth_key,
                            })
                            .map(|_| CtxUpdate(new_content).into())
                            .map_err(move |e| CtxActionErr(action, e.into()).into());
                            Effects::one(Box::new(effect)).unchanged()
                        }
                        None => Effects::msg(CtxUpdate(new_content).into()).unchanged(),
                    }
                }
                ActionUser::Register { email, password } => Effects::one(authenticate::<Env>(
                    action.to_owned(),
                    APIRequest::Register { email, password },
                ))
                .unchanged(),
                ActionUser::Login { email, password } => Effects::one(authenticate::<Env>(
                    action.to_owned(),
                    APIRequest::Login { email, password },
                ))
                .unchanged(),
                ActionUser::PullAddons => match &self.content.auth {
                    Some(Auth { key, .. }) => {
                        let action = action.to_owned();
                        let key = key.to_owned();
                        let req = APIRequest::AddonCollectionGet {
                            auth_key: key.to_owned(),
                            update: false,
                        };
                        // @TODO: respect last_modified
                        let ft = api_fetch::<Env, CollectionResponse, _>(req)
                            .map(move |r| CtxAddonsPulled(key, r.addons).into())
                            .map_err(move |e| CtxActionErr(action, e.into()).into());
                        Effects::one(Box::new(ft)).unchanged()
                    }
                    None => Effects::none().unchanged(),
                },
                ActionUser::PushAddons => match &self.content.auth {
                    Some(Auth { key, .. }) => {
                        let action = action.to_owned();
                        let req = APIRequest::AddonCollectionSet {
                            auth_key: key.to_owned(),
                            addons: self.content.addons.to_owned(),
                        };
                        let ft = api_fetch::<Env, SuccessResponse, _>(req)
                            .map(|_| CtxAddonsPushed.into())
                            .map_err(move |e| CtxActionErr(action, e.into()).into());
                        Effects::one(Box::new(ft)).unchanged()
                    }
                    None => Effects::none().unchanged(),
                },
                ActionUser::LibSync => match &self.library {
                    LibraryLoadable::Ready(lib) => {
                        let key = lib.key.to_owned();
                        let action = action.to_owned();
                        let ft = lib
                            .sync::<Env>()
                            .map(move |items| LibSyncPulled(key, items).into())
                            .map_err(move |e| CtxActionErr(action, e.into()).into());
                        Effects::one(Box::new(ft)).unchanged()
                    }
                    _ => Effects::none().unchanged(),
                },
            },
            // Handling msgs that result effects
            Msg::Internal(CtxAddonsPulled(key, addons))
                if self.content.auth.as_ref().map(|a| &a.key) == Some(&key)
                    && &self.content.addons != addons =>
            {
                self.content.addons = addons.to_owned();
                Effects::msg(CtxAddonsChangedFromPull.into())
                    .join(Effects::one(save_storage::<Env>(&self.content)))
            }
            Msg::Internal(CtxUpdate(new)) => {
                self.content = *new.to_owned();
                // @TODO update .library
                Effects::msg(CtxChanged.into())
                    .join(Effects::one(save_storage::<Env>(&self.content)))
            }
            Msg::Internal(LibSyncPulled(key, items)) => match &mut self.library {
                LibraryLoadable::Ready(lib) if key == &lib.key => {
                    lib.update(items);
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        }
    }
}

fn load_storage<Env: Environment>() -> Effect {
    Box::new(
        Env::get_storage(USER_DATA_KEY)
            .map(|x| Msg::Internal(CtxLoaded(x)))
            .map_err(|e| Msg::Event(CtxFatal(e.into()))),
    )
}

fn save_storage<Env: Environment>(content: &CtxContent) -> Effect {
    Box::new(
        Env::set_storage(USER_DATA_KEY, Some(content))
            .map(|_| Msg::Event(CtxSaved))
            .map_err(|e| Msg::Event(CtxFatal(e.into()))),
    )
}

fn authenticate<Env: Environment + 'static>(action: ActionUser, req: APIRequest) -> Effect {
    let ft = api_fetch::<Env, AuthResponse, _>(req)
        .and_then(move |AuthResponse { key, user }| {
            let pull_req = APIRequest::AddonCollectionGet {
                auth_key: key.to_owned(),
                update: true,
            };
            api_fetch::<Env, CollectionResponse, _>(pull_req).map(
                move |CollectionResponse { addons, .. }| CtxContent {
                    auth: Some(Auth { key, user }),
                    addons,
                },
            )
        })
        .map(|c| Msg::Internal(CtxUpdate(Box::new(c))))
        .map_err(move |e| Msg::Event(CtxActionErr(action, e.into())));
    Box::new(ft)
}
