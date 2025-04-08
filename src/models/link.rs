use crate::models::common::{eq_update, Loadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLink, ActionLoad, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvError, EnvFutureExt, UpdateWithCtx};
use crate::types::api::{
    fetch_api, APIError, APIResult, LinkCodeResponse, LinkDataResponse, LinkRequest,
};
use derivative::Derivative;
use derive_more::From;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};
use serde::Serialize;
use std::fmt;

#[derive(Clone, PartialEq, From, Serialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum LinkError {
    API(APIError),
    Env(EnvError),
    UnexpectedResponse(String),
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            LinkError::API(error) => write!(f, "API: {}", error.message),
            LinkError::Env(error) => write!(f, "Env: {}", error.message()),
            LinkError::UnexpectedResponse(message) => write!(f, "UnexpectedResponse: {message}"),
        }
    }
}

#[derive(Derivative, Serialize, Clone, Debug)]
#[derivative(Default(bound = ""))]
#[serde(rename_all = "camelCase")]
pub struct Link<T> {
    pub code: Option<Loadable<LinkCodeResponse, LinkError>>,
    pub data: Option<Loadable<T, LinkError>>,
}

impl<T, E> UpdateWithCtx<E> for Link<T>
where
    T: PartialEq + TryFrom<LinkDataResponse, Error = derive_more::TryIntoError<LinkDataResponse>>,
    E: Env + 'static,
{
    fn update(&mut self, msg: &Msg, _: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Link)) => {
                let code_effects = eq_update(&mut self.code, Some(Loadable::Loading));
                let data_effects = eq_update(&mut self.data, None);
                Effects::one(create_code::<E>())
                    .unchanged()
                    .join(code_effects)
                    .join(data_effects)
            }
            Msg::Action(Action::Link(ActionLink::ReadData)) => match &self.code {
                Some(Loadable::Ready(LinkCodeResponse { code, .. })) => {
                    let data_effects = eq_update(&mut self.data, Some(Loadable::Loading));
                    Effects::one(read_data::<E>(code))
                        .unchanged()
                        .join(data_effects)
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Action(Action::Unload) => {
                let code_effects = eq_update(&mut self.code, None);
                let data_effects = eq_update(&mut self.data, None);
                code_effects.join(data_effects)
            }
            Msg::Internal(Internal::LinkCodeResult(result))
                if self
                    .code
                    .as_ref()
                    .map(|code| code.is_loading())
                    .unwrap_or_default() =>
            {
                let code_effects = match result {
                    Ok(resp) => eq_update(&mut self.code, Some(Loadable::Ready(resp.to_owned()))),
                    Err(error) => eq_update(&mut self.code, Some(Loadable::Err(error.to_owned()))),
                };
                let data_effects = eq_update(&mut self.data, None);
                code_effects.join(data_effects)
            }
            Msg::Internal(Internal::LinkDataResult(request_code, result))
                if self
                    .data
                    .as_ref()
                    .map(|data| data.is_loading())
                    .unwrap_or_default() =>
            {
                match &self.code {
                    Some(Loadable::Ready(LinkCodeResponse { code, .. }))
                        if code == request_code =>
                    {
                        let next_data = match result {
                            Ok(data) => match T::try_from(data.to_owned()) {
                                Ok(data) => Loadable::Ready(data),
                                Err(error) => {
                                    Loadable::Err(LinkError::UnexpectedResponse(error.to_string()))
                                }
                            },
                            Err(error) => Loadable::Err(error.to_owned()),
                        };
                        eq_update(&mut self.data, Some(next_data))
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn create_code<E: Env + 'static>() -> Effect {
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, LinkCodeResponse>(&LinkRequest::Create)
            .map_err(LinkError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(LinkError::from(error)),
            })
            .map(|result| Msg::Internal(Internal::LinkCodeResult(result)))
            .boxed_env(),
    )
    .into()
}

fn read_data<E: Env + 'static>(code: &str) -> Effect {
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, LinkDataResponse>(&LinkRequest::Read {
            code: code.to_owned(),
        })
        .map_err(LinkError::from)
        .and_then(|result| match result {
            APIResult::Ok(result) => future::ok(result),
            APIResult::Err(error) => future::err(LinkError::from(error)),
        })
        .map(enclose!((code.to_owned() => code) move |result| {
            Msg::Internal(Internal::LinkDataResult(code, result))
        }))
        .boxed_env(),
    )
    .into()
}
