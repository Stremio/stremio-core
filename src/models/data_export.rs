use futures::{future, FutureExt, TryFutureExt};
use percent_encoding::utf8_percent_encode;
use serde::Serialize;
use url::Url;

use crate::{
    constants::URI_COMPONENT_ENCODE_SET,
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvFutureExt, UpdateWithCtx,
    },
    types::{
        api::{fetch_api, APIRequest, APIResult, DataExportResponse},
        profile::AuthKey,
    },
};

use super::{
    common::{eq_update, Loadable},
    ctx::{Ctx, CtxError},
};

#[derive(Serialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataExport {
    /// This is the Loading result of the User data export request.
    pub export_url: Option<(AuthKey, Loadable<Url, CtxError>)>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for DataExport {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::DataExport)) => {
                let auth_key = match ctx.profile.auth_key() {
                    Some(auth_key) => auth_key,
                    None => return Effects::none().unchanged(),
                };

                match &self.export_url {
                    Some((_, Loadable::Loading)) => Effects::none().unchanged(),
                    _ => {
                        let export_url_effects = eq_update(
                            &mut self.export_url,
                            Some((auth_key.to_owned(), Loadable::Loading)),
                        );

                        Effects::one(export_data_from_api::<E>(auth_key.to_owned()))
                            .unchanged()
                            .join(export_url_effects)
                    }
                }
            }
            Msg::Action(Action::Unload) => eq_update(&mut self.export_url, None),
            Msg::Internal(Internal::DataExportResult(auth_key, result)) => match self.export_url {
                Some((ref loading_auth_key, Loadable::Loading)) if loading_auth_key == auth_key => {
                    match result {
                        Ok(result) => {
                            let loaded_export_url = format!(
                                "https://api.strem.io/data-export/{}/export.json",
                                utf8_percent_encode(&result.export_id, URI_COMPONENT_ENCODE_SET)
                            )
                            .parse()
                            .expect("Failed to parse user export url");

                            eq_update(
                                &mut self.export_url,
                                Some((auth_key.to_owned(), Loadable::Ready(loaded_export_url))),
                            )
                        }
                        Err(err) => eq_update(
                            &mut self.export_url,
                            Some((auth_key.to_owned(), Loadable::Err(err.to_owned()))),
                        ),
                    }
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ProfileChanged) => {
                match (self.export_url.as_ref(), ctx.profile.auth_key()) {
                    (Some((export_auth_key, _)), profile_auth_key)
                        if Some(export_auth_key) != profile_auth_key =>
                    {
                        eq_update(&mut self.export_url, None)
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn export_data_from_api<E: Env + 'static>(auth_key: AuthKey) -> Effect {
    let api_request = APIRequest::DataExport {
        auth_key: auth_key.clone(),
    };

    EffectFuture::Concurrent(
        fetch_api::<E, _, _, DataExportResponse>(&api_request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::DataExportResult(auth_key, result)))
            .boxed_env(),
    )
    .into()
}
