use chrono::DateTime;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};

use crate::constants::DISMISSED_EVENTS_STORAGE_KEY;
use crate::models::common::{eq_update, Loadable};
use crate::models::ctx::CtxError;
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::types::api::{
    fetch_api, APIRequest, APIResult, GetModalResponse, GetNotificationResponse,
};
use crate::types::events::{DismissedEventsBucket, Events};

pub fn update_events<E: Env + 'static>(
    events: &mut Events,
    dismissed_events: &mut DismissedEventsBucket,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Internal(Internal::Logout(_)) => {
            let next_dismissed_events = DismissedEventsBucket::default();
            *dismissed_events = next_dismissed_events;
            Effects::msg(Msg::Internal(Internal::DismissedEventsChanged))
        }
        Msg::Action(Action::Ctx(ActionCtx::GetEvents)) => {
            let modal_effects = eq_update(&mut events.modal, Loadable::Loading);
            let notification_effects = eq_update(&mut events.notification, Loadable::Loading);
            let requests_effects = Effects::many(vec![get_modal::<E>(), get_notification::<E>()]);

            modal_effects
                .join(notification_effects)
                .join(requests_effects)
        }
        Msg::Action(Action::Ctx(ActionCtx::DismissEvent(id))) => {
            dismissed_events
                .items
                .insert(id.to_owned(), DateTime::from(E::now()));

            let message_effects = Effects::one(Effect::Msg(Box::new(Msg::Internal(
                Internal::DismissedEventsChanged,
            ))));

            let events_modal_effects = match events.modal.as_ref() {
                Loadable::Ready(result) => match result {
                    Some(GetModalResponse { id, .. })
                        if dismissed_events.items.contains_key(id) =>
                    {
                        eq_update(&mut events.modal, Loadable::Ready(None))
                    }
                    _ => Effects::none().unchanged(),
                },
                _ => Effects::none().unchanged(),
            };

            let events_notification_effects = match events.notification.as_ref() {
                Loadable::Ready(result) => match result {
                    Some(GetNotificationResponse { id, .. })
                        if dismissed_events.items.contains_key(id) =>
                    {
                        eq_update(&mut events.notification, Loadable::Ready(None))
                    }
                    _ => Effects::none().unchanged(),
                },
                _ => Effects::none().unchanged(),
            };

            message_effects
                .join(events_modal_effects)
                .join(events_notification_effects)
        }
        Msg::Internal(Internal::GetModalResult(_, result)) => match result {
            Ok(response) => match response {
                Some(GetModalResponse { id, .. }) if !dismissed_events.items.contains_key(id) => {
                    eq_update(&mut events.modal, Loadable::Ready(response.to_owned()))
                }
                _ => eq_update(&mut events.modal, Loadable::Ready(None)),
            },
            Err(error) => eq_update(&mut events.modal, Loadable::Err(error.to_owned())),
        },
        Msg::Internal(Internal::GetNotificationResult(_, result)) => match result {
            Ok(response) => match response {
                Some(GetNotificationResponse { id, .. })
                    if !dismissed_events.items.contains_key(id) =>
                {
                    eq_update(
                        &mut events.notification,
                        Loadable::Ready(response.to_owned()),
                    )
                }
                _ => eq_update(&mut events.notification, Loadable::Ready(None)),
            },
            Err(error) => eq_update(&mut events.notification, Loadable::Err(error.to_owned())),
        },
        Msg::Internal(Internal::DismissedEventsChanged) => {
            Effects::one(push_dismissed_events_to_storage::<E>(dismissed_events)).unchanged()
        }
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
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
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
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::GetNotificationResult(request, result)))
            .boxed_env(),
    )
    .into()
}

fn push_dismissed_events_to_storage<E: Env + 'static>(
    dismissed_events: &DismissedEventsBucket,
) -> Effect {
    EffectFuture::Sequential(
        E::set_storage(DISMISSED_EVENTS_STORAGE_KEY, Some(&dismissed_events))
            .map(
                enclose!((dismissed_events.uid => uid) move |result| match result {
                    Ok(_) => Msg::Event(Event::DismissedEventsPushedToStorage { uid }),
                    Err(error) => Msg::Event(Event::Error {
                        error: CtxError::from(error),
                        source: Box::new(Event::DismissedEventsPushedToStorage { uid }),
                    })
                }),
            )
            .boxed_env(),
    )
    .into()
}
