use enclose::enclose;
use futures::FutureExt;
use std::collections::hash_map::Entry;

use crate::constants::STREAMS_STORAGE_KEY;
use crate::models::common::{Loadable, ResourceLoadable};
use crate::models::ctx::{CtxError, CtxStatus};
use crate::runtime::msg::{CtxAuthResponse, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::types::streams::{StreamsBucket, StreamsItem, StreamsItemKey};

pub fn update_streams<E: Env + 'static>(
    streams: &mut StreamsBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Internal(Internal::Logout(_)) => {
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
            meta_item:
                ResourceLoadable {
                    request: meta_request,
                    content: Some(meta_content),
                },
        }) if !meta_content.is_loading() => {
            let meta_id = &meta_request.path.id;
            let video_id = &stream_request.path.id;

            let key = StreamsItemKey {
                meta_id: meta_id.to_owned(),
                video_id: video_id.to_owned(),
            };
            let last_stream_item = match meta_content {
                Loadable::Ready(meta_item) => streams.last_stream_item(video_id, meta_item),
                _ => streams.items.get(&key),
            };
            let last_stream_state = last_stream_item.and_then(|item| item.adjusted_state(stream));

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
            let entry = streams
                .items
                .entry(key)
                .and_modify(|item| item.state = Some(state.to_owned()));
            match entry {
                Entry::Occupied(_) => Effects::msg(Msg::Internal(Internal::StreamsChanged(false))),
                _ => Effects::none().unchanged(),
            }
        }
        Msg::Internal(Internal::StreamsChanged(persisted)) if !persisted => {
            Effects::one(push_streams_to_storage::<E>(streams)).unchanged()
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok(CtxAuthResponse { auth, .. }))
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
