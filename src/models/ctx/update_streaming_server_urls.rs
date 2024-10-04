use crate::constants::STREAMING_SERVER_URLS_STORAGE_KEY;
use crate::runtime::msg::{Action, ActionStreamingServer};
use crate::runtime::EnvFutureExt;
use crate::{
    runtime::{
        msg::{ActionServerUrlsBucket, Event, Internal, Msg},
        Effect, EffectFuture, Effects, Env,
    },
    types::{profile::Profile, streaming_server::ServerUrlsBucket},
};
use futures::FutureExt;

use super::CtxError;

pub fn update_streaming_server_urls<E: Env + 'static>(
    server_urls_bucket: &mut ServerUrlsBucket,
    profile: &Profile,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::StreamingServer(ActionStreamingServer::ServerUrlsBucket(action))) => {
            match action {
                ActionServerUrlsBucket::AddServerUrl(url) => {
                    server_urls_bucket.add_url(url.clone());
                    Effects::msg(Msg::Internal(Internal::StreamingServerUrlsBucketChanged))
                }
            }
        }
        Msg::Internal(Internal::StreamingServerUrlsBucketChanged) => {
            Effects::one(push_server_urls_to_storage::<E>(server_urls_bucket)).unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

fn push_server_urls_to_storage<E: Env + 'static>(server_urls_bucket: &ServerUrlsBucket) -> Effect {
    let uid = server_urls_bucket.uid.clone();

    EffectFuture::Sequential(
        E::set_storage(STREAMING_SERVER_URLS_STORAGE_KEY, Some(server_urls_bucket))
            .map(move |result| match result {
                Ok(_) => Msg::Event(Event::StreamingServerUrlsPushedToStorage { uid: uid.clone() }),
                Err(error) => Msg::Event(Event::Error {
                    error: CtxError::from(error),
                    source: Box::new(Event::StreamingServerUrlsPushedToStorage {
                        uid: uid.clone(),
                    }),
                }),
            })
            .boxed_env(),
    )
    .into()
}
