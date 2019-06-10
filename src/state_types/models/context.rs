use crate::state_types::Event::*;
use crate::state_types::*;
use crate::types::addons::Descriptor;
use crate::types::api::*;
use lazy_static::*;
use serde_derive::*;
use std::marker::PhantomData;

lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> = serde_json::from_slice(include_bytes!(
        "../../../stremio-official-addons/index.json"
    ))
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

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::LoadCtx) => Effects::one(load_storage::<Env>()).unchanged(),
            Msg::Internal(CtxLoaded(Some(content))) => {
                self.is_loaded = true;
                self.content = *content.to_owned();
                Effects::none()
            }
            Msg::Internal(CtxLoaded(None)) => {
                self.is_loaded = true;
                Effects::none()
            }
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
            // logout -> either emit CtxUpdate immediately or first remove session, then do it
            // login/register -> Internal::CtxUpdate -> Event::CtxChanged
            //   CtxUpdate - should call save_storage
            // push addons - Event::CtxPushedAddons
            // pull addons - the result of that, Internal::CtxPulledAddons, should be handled and produces Event::CtxPulledAddons(is_new)
            Msg::Action(Action::UserOp(user_op @ ActionUser::Logout)) => {
                let new_content = Box::new(CtxContent::default());
                match &self.content.auth {
                    Some(Auth { key, .. }) => {
                        let auth_key = key.to_string();
                        let user_op = user_op.clone();
                        let effect = api_fetch::<Env, SuccessResponse>(
                            "https://api.strem.io",
                            APIRequest::Logout { auth_key },
                        )
                        .map(|_| CtxUpdate(new_content).into())
                        .map_err(move |e| UserOpError(user_op, e.into()).into());
                        Effects::one(Box::new(effect)).unchanged()
                    }
                    None => Effects::msg(CtxUpdate(new_content).into()).unchanged(),
                }
            }
            Msg::Internal(CtxUpdate(new)) => {
                // @TODO this is the place to check for changed add-ons/auth and emit the
                // corresponding events
                self.content = *new.to_owned();
                Effects::msg(CtxChanged.into())
                    .join(Effects::one(save_storage::<Env>(&self.content)))
            }
            _ => Effects::none().unchanged(),
        }
    }
}

// @TODO move these load/save?
const USER_DATA_KEY: &str = "userData";
fn load_storage<Env: Environment>() -> Effect {
    Box::new(
        Env::get_storage(USER_DATA_KEY)
            .map(|x| Msg::Internal(CtxLoaded(x)))
            .map_err(|e| Msg::Event(ContextMiddlewareFatal(e.into()))),
    )
}

fn save_storage<Env: Environment>(content: &CtxContent) -> Effect {
    Box::new(
        Env::set_storage(USER_DATA_KEY, Some(content))
            .map(|_| Msg::Event(CtxSaved))
            .map_err(|e| Msg::Event(ContextMiddlewareFatal(e.into()))),
    )
}

fn api_fetch<Env: Environment + 'static, OUT: 'static>(
    base_url: &str,
    api_req: APIRequest,
) -> impl Future<Item = OUT, Error = MiddlewareError>
where
    OUT: serde::de::DeserializeOwned,
{
    let url = format!("{}/api/{}", base_url, api_req.method_name());
    let req = Request::post(url)
        .body(api_req)
        .expect("builder cannot fail");
    Env::fetch_serde::<_, APIResult<OUT>>(req)
        .map_err(Into::into)
        .and_then(|res| match res {
            APIResult::Err { error } => future::err(error.into()),
            APIResult::Ok { result } => future::ok(result),
        })
}
