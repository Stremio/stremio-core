use chrono::DateTime;
use futures::{future, FutureExt, TryFutureExt};

use crate::models::common::{eq_update, Loadable};
use crate::models::ctx::CtxError;
use crate::runtime::msg::{Action, ActionCtx, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::types::api::{
    fetch_api, APIRequest, APIResult, GetModalResponse, GetNotificationResponse,
};
use crate::types::events::Events;

pub fn update_events<E: Env + 'static>(events: &mut Events, msg: &Msg) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::GetEvents)) => {
            let modal_effects = eq_update(&mut events.modal, Loadable::Loading);
            let notification_effects = eq_update(&mut events.notification, Loadable::Loading);
            let requests_effects = Effects::many(vec![get_modal::<E>(), get_notification::<E>()]);

            modal_effects
                .join(notification_effects)
                .join(requests_effects)
        }
        Msg::Internal(Internal::GetModalResult(_, result)) => match result {
            Ok(response) => eq_update(&mut events.modal, Loadable::Ready(response.to_owned())),
            Err(error) => eq_update(&mut events.modal, Loadable::Err(error.to_owned())),
        },
        Msg::Internal(Internal::GetNotificationResult(_, result)) => match result {
            Ok(response) => eq_update(
                &mut events.notification,
                Loadable::Ready(response.to_owned()),
            ),
            Err(error) => eq_update(&mut events.notification, Loadable::Err(error.to_owned())),
        },
        _ => Effects::none().unchanged(),
    }
}

fn get_modal<E: Env + 'static>() -> Effect {
    let request = APIRequest::GetModal {
        date: DateTime::from(E::now()),
    };

    EffectFuture::Concurrent(
        fetch_api::<E, _, _, Option<GetModalResponse>>(&request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::GetModalResult(request, result)))
            .boxed_env(),
    )
    .into()
}

fn get_notification<E: Env + 'static>() -> Effect {
    let request = APIRequest::GetNotification {
        date: DateTime::from(E::now()),
    };

    EffectFuture::Concurrent(
        fetch_api::<E, _, _, Option<GetNotificationResponse>>(&request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::GetNotificationResult(request, result)))
            .boxed_env(),
    )
    .into()
}
