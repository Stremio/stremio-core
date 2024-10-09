use crate::constants::STREAMING_SERVER_URLS_STORAGE_KEY;
use crate::runtime::msg::{Action, ActionStreamingServer, CtxAuthResponse};
use crate::runtime::EnvFutureExt;
use crate::runtime::{
    msg::{ActionServerUrlsBucket, Event, Internal, Msg},
    Effect, EffectFuture, Effects, Env,
};
use crate::types::server_urls::ServerUrlsBucket;
use futures::FutureExt;

use super::{CtxError, CtxStatus};

pub fn update_streaming_server_urls<E: Env + 'static>(
    streaming_server_urls: &mut ServerUrlsBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::StreamingServer(ActionStreamingServer::ServerUrlsBucket(action))) => {
            match action {
                ActionServerUrlsBucket::AddServerUrl(url) => {
                    streaming_server_urls.add_url(url.clone());
                    Effects::msg(Msg::Internal(Internal::StreamingServerUrlsBucketChanged))
                }
                ActionServerUrlsBucket::EditServerUrl { id, new_url } => {
                    streaming_server_urls.edit_item(id, new_url.clone());
                    Effects::msg(Msg::Internal(Internal::StreamingServerUrlsBucketChanged))
                }
                ActionServerUrlsBucket::DeleteServerUrl(id) => {
                    streaming_server_urls.delete_item(id);
                    Effects::msg(Msg::Internal(Internal::StreamingServerUrlsBucketChanged))
                }
            }
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok(CtxAuthResponse { auth, .. }))
                if loading_auth_request == auth_request =>
            {
                let next_server_urls = ServerUrlsBucket::new(Some(auth.user.id.to_owned()));
                *streaming_server_urls = next_server_urls;
                Effects::msg(Msg::Internal(Internal::StreamingServerUrlsBucketChanged))
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::StreamingServerUrlsBucketChanged) => {
            Effects::one(push_server_urls_to_storage::<E>(streaming_server_urls)).unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

fn push_server_urls_to_storage<E: Env + 'static>(
    streaming_server_urls: &ServerUrlsBucket,
) -> Effect {
    let uid: Option<String> = streaming_server_urls.uid.clone();

    EffectFuture::Sequential(
        E::set_storage(
            STREAMING_SERVER_URLS_STORAGE_KEY,
            Some(streaming_server_urls),
        )
        .map(move |result| match result {
            Ok(_) => Msg::Event(Event::StreamingServerUrlsPushedToStorage { uid: uid.clone() }),
            Err(error) => Msg::Event(Event::Error {
                error: CtxError::from(error),
                source: Box::new(Event::StreamingServerUrlsPushedToStorage { uid: uid.clone() }),
            }),
        })
        .boxed_env(),
    )
    .into()
}
