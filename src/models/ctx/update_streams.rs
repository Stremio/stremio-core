use enclose::enclose;
use futures::FutureExt;

use crate::{
    constants::STREAMS_STORAGE_KEY,
    models::{
        common::{Loadable, ResourceLoadable},
        ctx::{CtxError, CtxStatus},
    },
    runtime::{
        msg::{CtxAuthResponse, Event, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvFutureExt,
    },
    types::{
        addon::ResourceRequest,
        resource::Stream,
        streams::{StreamItemState, StreamsBucket, StreamsItem, StreamsItemKey},
    },
};

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
            let exact_item = streams.items.get(&key);
            let last_stream_item = match meta_content {
                Loadable::Ready(meta_item) => exact_item
                    .filter(|item| item.stream.is_source_match(stream))
                    .or_else(|| streams.last_stream_item(video_id, meta_item)),
                _ => exact_item,
            };
            let last_stream_state = last_stream_item.and_then(|item| item.adjusted_state(stream));

            streams.items.insert(
                key,
                streams_item::<E>(stream, stream_request, meta_request, last_stream_state),
            );
            Effects::msg(Msg::Internal(Internal::StreamsChanged(false)))
        }
        Msg::Internal(Internal::StreamStateChanged {
            state,
            stream,
            stream_request,
            meta_request,
        }) => {
            let meta_id = &meta_request.path.id;
            let video_id = &stream_request.path.id;

            let key = StreamsItemKey {
                meta_id: meta_id.to_owned(),
                video_id: video_id.to_owned(),
            };
            match streams.items.get_mut(&key) {
                Some(item) if item.stream.is_source_match(stream) => {
                    item.state = Some(state.to_owned());
                }
                _ => {
                    streams.items.insert(
                        key,
                        streams_item::<E>(
                            stream,
                            stream_request,
                            meta_request,
                            Some(state.to_owned()),
                        ),
                    );
                }
            }
            Effects::msg(Msg::Internal(Internal::StreamsChanged(false)))
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

fn streams_item<E: Env>(
    stream: &Stream,
    stream_request: &ResourceRequest,
    meta_request: &ResourceRequest,
    state: Option<StreamItemState>,
) -> StreamsItem {
    StreamsItem {
        stream: stream.to_owned(),
        r#type: meta_request.path.r#type.to_owned(),
        meta_id: meta_request.path.id.to_owned(),
        video_id: stream_request.path.id.to_owned(),
        meta_transport_url: meta_request.base.to_owned(),
        stream_transport_url: stream_request.base.to_owned(),
        state,
        mtime: E::now(),
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
