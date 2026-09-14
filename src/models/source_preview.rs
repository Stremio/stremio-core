use futures::FutureExt;
use serde::Serialize;

use crate::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        common::{
            resources_update, resources_update_with_vector_content, Loadable, ResourceError,
            ResourceLoadable, ResourcesAction,
        },
        ctx::Ctx,
        meta_details::{
            meta_items_update, meta_streams_update, selected_guess_stream_update, streams_update,
            Selected,
        },
    },
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvFutureExt, UpdateWithCtx,
    },
    types::{
        addon::ResourcePath,
        resource::{MetaItem, Stream, StreamSource},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SourceStatus {
    Unknown,
    Checking,
    SourcesFound,
    ExternalOnly,
    NoSources,
    CheckFailed,
    NoProvider,
    ChooseVideo,
}

#[derive(Serialize)]
pub struct Preview<'a> {
    pub id: Option<&'a str>,
    pub status: SourceStatus,
}

/// An explicit, single-movie check. It does not modify details, ratings or library state.
#[derive(Default, Clone, Debug, Serialize)]
pub struct SourcePreview {
    pub selected: Option<Selected>,
    pub meta_items: Vec<ResourceLoadable<MetaItem>>,
    pub meta_streams: Vec<ResourceLoadable<Vec<Stream>>>,
    pub streams: Vec<ResourceLoadable<Vec<Stream>>>,
    #[serde(skip)]
    generation: u64,
}

impl SourcePreview {
    pub fn preview(&self) -> Preview<'_> {
        Preview {
            id: self
                .selected
                .as_ref()
                .map(|selected| selected.meta_path.id.as_str()),
            status: self.status(),
        }
    }

    pub fn status(&self) -> SourceStatus {
        if self.selected.is_none() {
            return SourceStatus::Unknown;
        }
        // Match the details serializer: embedded video sources take precedence.
        let streams = if self.meta_streams.is_empty() {
            &self.streams
        } else {
            &self.meta_streams
        };
        let mut external = false;
        for stream in streams
            .iter()
            .filter_map(|resource| resource.content.as_ref()?.ready())
            .flatten()
        {
            if matches!(stream.source, StreamSource::External { .. }) {
                external = true;
            } else {
                return SourceStatus::SourcesFound;
            }
        }
        if self.meta_items.iter().any(pending) || streams.iter().any(pending) {
            return SourceStatus::Checking;
        }
        let has_meta = self
            .meta_items
            .iter()
            .any(|item| matches!(item.content, Some(Loadable::Ready(_))));
        if streams.iter().any(failed) || (!has_meta && self.meta_items.iter().any(failed)) {
            return SourceStatus::CheckFailed;
        }
        if external {
            return SourceStatus::ExternalOnly;
        }
        if self.meta_items.is_empty()
            || (has_meta
                && self
                    .selected
                    .as_ref()
                    .is_some_and(|selected| selected.stream_path.is_some())
                && streams.is_empty())
        {
            return SourceStatus::NoProvider;
        }
        if has_meta
            && self
                .selected
                .as_ref()
                .is_some_and(|selected| selected.stream_path.is_none())
        {
            return SourceStatus::ChooseVideo;
        }
        SourceStatus::NoSources
    }

    fn clear(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.selected = None;
        self.meta_items.clear();
        self.meta_streams.clear();
        self.streams.clear();
    }

    fn isolate(&self, effects: Effects) -> Effects {
        let changed = effects.has_changed;
        let generation = self.generation;
        let effects = Effects::many(
            effects
                .into_iter()
                .map(|effect| match effect {
                    Effect::Future(EffectFuture::Concurrent(future)) => {
                        Effect::Future(EffectFuture::Concurrent(
                            future
                                .map(move |message| isolate_result(generation, message))
                                .boxed_env(),
                        ))
                    }
                    Effect::Future(EffectFuture::Sequential(future)) => {
                        Effect::Future(EffectFuture::Sequential(
                            future
                                .map(move |message| isolate_result(generation, message))
                                .boxed_env(),
                        ))
                    }
                    effect => effect,
                })
                .collect(),
        );
        if changed {
            effects
        } else {
            effects.unchanged()
        }
    }
}

fn isolate_result(generation: u64, message: Msg) -> Msg {
    match message {
        Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
            Msg::Internal(Internal::SourcePreviewResult(generation, request, result))
        }
        message => message,
    }
}

fn pending<T>(resource: &ResourceLoadable<T>) -> bool {
    matches!(resource.content, None | Some(Loadable::Loading))
}

fn failed<T>(resource: &ResourceLoadable<T>) -> bool {
    matches!(resource.content, Some(Loadable::Err(ref error)) if !matches!(error, ResourceError::EmptyContent))
}

impl<E: Env + 'static> UpdateWithCtx<E> for SourcePreview {
    fn update(&mut self, message: &Msg, ctx: &Ctx) -> Effects {
        let effects = match message {
            Msg::Action(Action::Load(ActionLoad::SourcePreview(id))) => {
                self.clear();
                if id.is_empty() {
                    return Effects::none();
                }
                self.selected = Some(Selected {
                    meta_path: ResourcePath::without_extra(META_RESOURCE_NAME, "movie", id),
                    stream_path: None,
                    guess_stream: true,
                });
                meta_items_update::<E>(&mut self.meta_items, &self.selected, &ctx.profile)
            }
            Msg::Action(Action::Unload) | Msg::Internal(Internal::ProfileChanged) => {
                self.clear();
                Effects::none()
            }
            Msg::Internal(Internal::SourcePreviewResult(generation, request, result))
                if *generation == self.generation =>
            {
                if request.path.resource == META_RESOURCE_NAME {
                    let effects = resources_update::<E, _>(
                        &mut self.meta_items,
                        ResourcesAction::ResourceRequestResult { request, result },
                    );
                    let selected_effects =
                        selected_guess_stream_update(&mut self.selected, &self.meta_items);
                    let stream_effects = if selected_effects.has_changed {
                        streams_update::<E>(&mut self.streams, &self.selected, &ctx.profile)
                    } else {
                        Effects::default()
                    };
                    effects
                        .join(selected_effects)
                        .join(stream_effects)
                        .join(meta_streams_update(
                            &mut self.meta_streams,
                            &self.selected,
                            &self.meta_items,
                        ))
                } else if request.path.resource == STREAM_RESOURCE_NAME {
                    resources_update_with_vector_content::<E, _>(
                        &mut self.streams,
                        ResourcesAction::ResourceRequestResult { request, result },
                    )
                } else {
                    Effects::default()
                }
            }
            _ => Effects::default(),
        };
        self.isolate(effects)
    }
}
