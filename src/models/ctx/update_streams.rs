use enclose::enclose;
use futures::FutureExt;

use crate::constants::STREAMS_STORAGE_KEY;
use crate::models::ctx::{CtxError, CtxStatus};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::types::streams::{StreamsBucket, StreamsItem, StreamsItemKey};

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
            stream_request: Some(stream_request),
            meta_request: Some(meta_request),
        }) => {
            let meta_id = &meta_request.path.id;
            let video_id = &stream_request.path.id;

            let key = StreamsItemKey {
                meta_id: meta_id.to_owned(),
                video_id: video_id.to_owned(),
            };
            let last_stream_state = streams
                .items
                .get(&key)
                .and_then(|item| item.adjusted_state(stream));

            let streams_item = StreamsItem {
                stream: stream.to_owned(),
                r#type: meta_request.path.r#type.to_owned(),
                meta_id: meta_id.to_owned(),
                video_id: video_id.to_owned(),
                meta_transport_url: meta_request.base.to_owned(),
                stream_transport_url: stream_request.base.to_owned(),
                state: last_stream_state,
                mtime: E::now(),
            };

            streams.items.insert(key, streams_item);
            Effects::msg(Msg::Internal(Internal::StreamsChanged(false)))
        }
        Msg::Internal(Internal::StreamStateChanged {
            state,
            stream_request: Some(stream_request),
            meta_request: Some(meta_request),
        }) => {
            let meta_id = &meta_request.path.id;
            let video_id = &stream_request.path.id;

            let key = StreamsItemKey {
                meta_id: meta_id.to_owned(),
                video_id: video_id.to_owned(),
            };
            let steam_item = streams.items.get(&key).cloned();
            match steam_item {
                Some(item) => {
                    let new_stream_item = StreamsItem {
                        state: state.clone(),
                        ..item
                    };
                    streams.items.insert(key, new_stream_item);
                    Effects::msg(Msg::Internal(Internal::StreamsChanged(false)))
                }
                None => Effects::none().unchanged(),
            }
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
