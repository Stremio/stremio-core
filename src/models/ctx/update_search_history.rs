use enclose::enclose;
use futures::FutureExt;

use crate::constants::SEARCH_HISTORY_STORAGE_KEY;
use crate::models::ctx::{CtxError, CtxStatus};
use crate::runtime::msg::{Action, ActionCtx, CtxAuthResponse, Event, Internal};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::{runtime::msg::Msg, types::search_history::SearchHistoryBucket};

pub fn update_search_history<E: Env + 'static>(
    search_history: &mut SearchHistoryBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Internal(Internal::Logout(_)) => {
            let next_search_history = SearchHistoryBucket::default();
            *search_history = next_search_history;
            Effects::msg(Msg::Internal(Internal::SearchHistoryChanged))
        }
        Msg::Action(Action::Ctx(ActionCtx::ClearSearchHistory)) => {
            search_history.items.clear();
            Effects::msg(Msg::Internal(Internal::SearchHistoryChanged))
        }
        Msg::Internal(Internal::CatalogsWithExtraSearch { query }) => {
            search_history.items.insert(query.to_owned(), E::now());
            Effects::msg(Msg::Internal(Internal::SearchHistoryChanged))
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok(CtxAuthResponse { auth, .. }))
                if loading_auth_request == auth_request =>
            {
                let next_search_history = SearchHistoryBucket::new(Some(auth.user.id.to_owned()));
                *search_history = next_search_history;
                Effects::msg(Msg::Internal(Internal::SearchHistoryChanged))
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::SearchHistoryChanged) => {
            Effects::one(push_search_history_to_storage::<E>(search_history)).unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

fn push_search_history_to_storage<E: Env + 'static>(
    search_history: &SearchHistoryBucket,
) -> Effect {
    EffectFuture::Sequential(
        E::set_storage(SEARCH_HISTORY_STORAGE_KEY, Some(&search_history))
            .map(
                enclose!((search_history.uid => uid) move |result| match result {
                    Ok(_) => Msg::Event(Event::SearchHistoryPushedToStorage { uid }),
                    Err(error) => Msg::Event(Event::Error {
                        error: CtxError::from(error),
                        source: Box::new(Event::SearchHistoryPushedToStorage { uid }),
                    })
                }),
            )
            .boxed_env(),
    )
    .into()
}
