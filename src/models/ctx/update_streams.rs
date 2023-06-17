use crate::constants::STREAMS_STORAGE_KEY;
use crate::models::ctx::{CtxError, CtxStatus};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::types::streams::{StreamsBucket, StreamsItem};
use enclose::enclose;
use futures::FutureExt;

pub fn update_streams<E: Env + 'static>(
    streams: &mut StreamsBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::Logout)) | Msg::Internal(Internal::Logout) => {
            let next_streams = StreamsBucket::default();
            if *streams != next_streams {
                *streams = next_streams;
                Effects::msg(Msg::Internal(Internal::StreamsChanged(false)))
            } else {
                Effects::none().unchanged()
            }
        }
        Msg::Internal(Internal::StreamLoaded {
            stream,
            meta_id: Some(meta_id),
            video_id: Some(video_id),
            transport_url: Some(transport_url),
        }) => {
            let streams_item = StreamsItem {
                stream: stream.to_owned(),
                transport_url: transport_url.to_owned(),
                mtime: E::now(),
            };
            streams
                .items
                .insert((meta_id.to_owned(), video_id.to_owned()), streams_item);
            Effects::msg(Msg::Internal(Internal::StreamsChanged(false)))
        }
        Msg::Internal(Internal::StreamsChanged(persisted)) if !persisted => {
            Effects::one(push_streams_to_storage::<E>(streams)).unchanged()
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok((auth, _, _)))
                if loading_auth_request == auth_request =>
            {
                let next_streams = StreamsBucket::new(Some(auth.user.id.to_owned()));
                if *streams != next_streams {
                    *streams = next_streams;
                    Effects::msg(Msg::Internal(Internal::StreamsChanged(false)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        _ => Effects::none().unchanged(),
    }
}

fn push_streams_to_storage<E: Env + 'static>(streams: &StreamsBucket) -> Effect {
    EffectFuture::Sequential(
        E::set_storage(STREAMS_STORAGE_KEY, Some(&streams))
            .map(enclose!((streams.uid => uid) move |result| match result {
                Ok(_) => Msg::Event(Event::StreamsPushedToStorage { uid }),
                Err(error) => Msg::Event(Event::Error {
                    error: CtxError::from(error),
                    source: Box::new(Event::StreamsPushedToStorage { uid }),
                })
            }))
            .boxed_env(),
    )
    .into()
}
