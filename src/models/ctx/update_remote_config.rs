use chrono::{DateTime, Duration, Utc};
use futures::{future, FutureExt, TryFutureExt};
use http::Request;
use serde::Deserialize;

use crate::{
    constants::{
        API_URL, REMOTE_CONFIG_DEFAULT_REFRESH_SECONDS, REMOTE_CONFIG_EVENTS_BATCH_SIZE,
        REMOTE_CONFIG_GEO_MAX_AGE, REMOTE_CONFIG_MAX_REFRESH_SECONDS,
        REMOTE_CONFIG_MIN_REFRESH_SECONDS, REMOTE_CONFIG_RETRY_INTERVAL, REMOTE_CONFIG_STORAGE_KEY,
    },
    models::ctx::CtxError,
    runtime::{
        msg::{Action, ActionCtx, ActionRemoteConfig, Event, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvFutureExt,
    },
    types::{
        api::{fetch_api, APIRequest, APIResult, GeoResponse, SuccessResponse},
        profile::Profile,
        remote_config::{
            account_key, evaluate, fresh_config, is_within, patch_settings, set_settings_patches,
            ClientSection, ConfigEvent, ConfigEventKind, Geo, NotificationDef, RemoteConfig,
            RemoteConfigBucket, RemoteConfigResponse,
        },
    },
};

pub fn update_remote_config<E: Env + 'static>(
    remote_config: &mut RemoteConfig,
    bucket: &mut RemoteConfigBucket,
    profile: &Profile,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::RemoteConfig(action))) => {
            let now = E::now();
            match action {
                ActionRemoteConfig::Pull => {
                    let evaluate_effects =
                        evaluate_remote_config::<E>(remote_config, bucket, profile);
                    let fetch_effects = if is_refresh_due(remote_config, bucket, now) {
                        remote_config.last_attempt_at = Some(now);
                        let geo_fresh = bucket.geo.as_ref().is_some_and(|geo| {
                            is_within(geo.fetched_at, REMOTE_CONFIG_GEO_MAX_AGE, now)
                        });
                        let geo_effects = if geo_fresh {
                            Effects::none()
                        } else {
                            Effects::one(fetch_geo::<E>(now))
                        };
                        Effects::one(fetch_remote_config::<E>(now))
                            .join(geo_effects)
                            .unchanged()
                    } else {
                        Effects::none().unchanged()
                    };
                    evaluate_effects
                        .join(fetch_effects)
                        .join(flush_events::<E>(remote_config, bucket))
                }
                ActionRemoteConfig::NotificationShown(id) => {
                    let Some(rev) = notification_rev(remote_config, id, None) else {
                        return Effects::none().unchanged();
                    };
                    let mut queued = bucket.queue_event_once(
                        format!("n:{id}:{rev}:shown"),
                        notification_event(id, rev, "shown"),
                        now,
                    );
                    let experiment_id = notification_def(bucket, id, now)
                        .and_then(|item| item.targeting.experiment)
                        .map(|rule| rule.id);
                    if let Some(experiment_id) = experiment_id {
                        queued |= queue_experiment_event(
                            remote_config,
                            bucket,
                            &experiment_id,
                            "exposure",
                            now,
                        );
                    }
                    queued_effects::<E>(queued, remote_config, bucket)
                }
                ActionRemoteConfig::NotificationAction { id, button } => {
                    let Some(rev) = notification_rev(remote_config, id, Some(button)) else {
                        return Effects::none().unchanged();
                    };
                    let item = notification_def(bucket, id, now);
                    let settings_effects = item
                        .as_ref()
                        .and_then(|item| item.buttons.iter().find(|item| item.id == *button))
                        .and_then(|button| {
                            let mut patches = set_settings_patches(button).peekable();
                            patches.peek()?;
                            patch_settings(&profile.settings, patches)
                        })
                        .filter(|settings| *settings != profile.settings)
                        .map(|settings| {
                            Effects::msg(Msg::Internal(Internal::UpdateSettings(settings)))
                                .unchanged()
                        })
                        .unwrap_or_else(|| Effects::none().unchanged());
                    dismiss::<E>(bucket, profile, item.as_ref(), id, rev, now);
                    bucket.queue_event_once(
                        format!("n:{id}:{rev}:action:{button}"),
                        notification_event(id, rev, &format!("action:{button}")),
                        now,
                    );
                    settings_effects
                        .join(evaluate_remote_config::<E>(remote_config, bucket, profile))
                        .join(bucket_changed_effects::<E>(remote_config, bucket))
                }
                ActionRemoteConfig::DismissNotification(id) => {
                    let Some(rev) = notification_rev(remote_config, id, None) else {
                        return Effects::none().unchanged();
                    };
                    let item = notification_def(bucket, id, now);
                    dismiss::<E>(bucket, profile, item.as_ref(), id, rev, now);
                    bucket.queue_event_once(
                        format!("n:{id}:{rev}:dismissed"),
                        notification_event(id, rev, "dismissed"),
                        now,
                    );
                    evaluate_remote_config::<E>(remote_config, bucket, profile)
                        .join(bucket_changed_effects::<E>(remote_config, bucket))
                }
                ActionRemoteConfig::ExperimentExposed(id) => {
                    let queued = queue_experiment_event(remote_config, bucket, id, "exposure", now);
                    queued_effects::<E>(queued, remote_config, bucket)
                }
                ActionRemoteConfig::ExperimentEvent { id, event } => {
                    let is_goal = fresh_config(bucket, now).is_some_and(|config| {
                        config.experiments.experiments.iter().any(|experiment| {
                            experiment.id == *id && experiment.events.contains(event)
                        })
                    });
                    let queued =
                        is_goal && queue_experiment_event(remote_config, bucket, id, event, now);
                    queued_effects::<E>(queued, remote_config, bucket)
                }
            }
        }
        Msg::Internal(Internal::RemoteConfigResult(started_at, Ok(config)))
            if is_newer(*started_at, remote_config.applied_started_at) =>
        {
            let now = E::now();
            remote_config.applied_started_at = Some(*started_at);
            bucket.config = Some(config.to_owned());
            bucket.fetched_at = Some(now);
            bucket.prune(now);
            evaluate_remote_config::<E>(remote_config, bucket, profile)
                .join(Effects::msg(Msg::Internal(Internal::RemoteConfigBucketChanged)).unchanged())
        }
        Msg::Internal(Internal::GeoResult(started_at, Ok(GeoResponse { country })))
            if is_newer(*started_at, remote_config.applied_geo_started_at) =>
        {
            remote_config.applied_geo_started_at = Some(*started_at);
            bucket.geo = Some(Geo {
                country: country.as_ref().map(|country| country.to_uppercase()),
                fetched_at: E::now(),
            });
            evaluate_remote_config::<E>(remote_config, bucket, profile)
                .join(Effects::msg(Msg::Internal(Internal::RemoteConfigBucketChanged)).unchanged())
        }
        Msg::Internal(Internal::RemoteConfigEventsResult(started_at, last_seq, result)) => {
            if remote_config.flushing_since == Some(*started_at) {
                remote_config.flushing_since = None;
            }
            match result {
                Ok(_) => {
                    bucket.outbox.retain(|queued| queued.seq > *last_seq);
                    bucket_changed_effects::<E>(remote_config, bucket)
                }
                Err(_) => Effects::none().unchanged(),
            }
        }
        Msg::Internal(Internal::RemoteConfigBucketChanged) => {
            Effects::one(push_remote_config_to_storage::<E>(bucket)).unchanged()
        }
        Msg::Internal(Internal::ProfileChanged)
        | Msg::Internal(Internal::CtxAuthResult(..))
        | Msg::Internal(Internal::Logout(_))
        | Msg::Internal(Internal::UserAPIResult { .. }) => {
            evaluate_remote_config::<E>(remote_config, bucket, profile)
        }
        _ => Effects::none().unchanged(),
    }
}

pub fn evaluate_remote_config<E: Env + 'static>(
    remote_config: &mut RemoteConfig,
    bucket: &mut RemoteConfigBucket,
    profile: &Profile,
) -> Effects {
    let assignments = bucket.assignments.to_owned();
    let (notifications, experiments) = evaluate(bucket, &E::client_info(), profile, E::now());
    let bucket_effects = if bucket.assignments != assignments {
        Effects::msg(Msg::Internal(Internal::RemoteConfigBucketChanged)).unchanged()
    } else {
        Effects::none().unchanged()
    };
    let state_effects = if remote_config.notifications != notifications
        || remote_config.experiments != experiments
    {
        remote_config.notifications = notifications;
        remote_config.experiments = experiments;
        Effects::none()
    } else {
        Effects::none().unchanged()
    };
    state_effects.join(bucket_effects)
}

fn is_refresh_due(
    remote_config: &RemoteConfig,
    bucket: &RemoteConfigBucket,
    now: DateTime<Utc>,
) -> bool {
    let min_refresh_seconds = bucket
        .config
        .as_ref()
        .and_then(|config| ClientSection::deserialize(&config.client).ok())
        .and_then(|client| client.min_refresh_seconds)
        .unwrap_or(REMOTE_CONFIG_DEFAULT_REFRESH_SECONDS)
        .clamp(
            REMOTE_CONFIG_MIN_REFRESH_SECONDS,
            REMOTE_CONFIG_MAX_REFRESH_SECONDS,
        );
    let refreshed_recently = bucket.fetched_at.is_some_and(|fetched_at| {
        is_within(
            fetched_at,
            Duration::seconds(min_refresh_seconds as i64),
            now,
        )
    });
    let attempted_recently = remote_config
        .last_attempt_at
        .is_some_and(|last_attempt_at| {
            is_within(last_attempt_at, REMOTE_CONFIG_RETRY_INTERVAL, now)
        });
    !refreshed_recently && !attempted_recently
}

fn is_newer(started_at: DateTime<Utc>, applied_started_at: Option<DateTime<Utc>>) -> bool {
    applied_started_at.map_or(true, |applied_started_at| started_at > applied_started_at)
}

fn notification_rev(remote_config: &RemoteConfig, id: &str, button: Option<&str>) -> Option<u32> {
    remote_config
        .notifications
        .iter()
        .find(|notification| {
            notification.id == id
                && button.map_or(true, |button| {
                    notification.buttons.iter().any(|item| item.id == button)
                })
        })
        .map(|notification| notification.rev)
}

fn notification_def(
    bucket: &RemoteConfigBucket,
    id: &str,
    now: DateTime<Utc>,
) -> Option<NotificationDef> {
    fresh_config(bucket, now)?
        .notifications
        .items
        .into_iter()
        .find(|item| item.id == id)
}

fn notification_event(id: &str, rev: u32, event: &str) -> ConfigEvent {
    ConfigEvent {
        kind: ConfigEventKind::Notification,
        id: id.to_owned(),
        variant: rev.to_string(),
        run: None,
        event: event.to_owned(),
    }
}

fn queue_experiment_event(
    remote_config: &RemoteConfig,
    bucket: &mut RemoteConfigBucket,
    id: &str,
    event: &str,
    now: DateTime<Utc>,
) -> bool {
    let Some(assigned) = remote_config.experiments.get(id) else {
        return false;
    };
    let key = format!("e:{id}:{}:{}:{event}", assigned.unit_key, assigned.salt);
    let event = ConfigEvent {
        kind: ConfigEventKind::Experiment,
        id: id.to_owned(),
        variant: assigned.variant.to_owned(),
        run: Some(assigned.salt.to_owned()),
        event: event.to_owned(),
    };
    bucket.queue_event_once(key, event, now)
}

fn dismiss<E: Env + 'static>(
    bucket: &mut RemoteConfigBucket,
    profile: &Profile,
    item: Option<&NotificationDef>,
    id: &str,
    rev: u32,
    now: DateTime<Utc>,
) {
    bucket
        .dismissed
        .entry(account_key(&E::client_info(), profile))
        .or_default()
        .insert(format!("{id}:{rev}"), now);
    if !item.is_some_and(|item| item.targeting.accounts.is_some()) {
        bucket.last_interaction_at = Some(now);
    }
}

fn queued_effects<E: Env + 'static>(
    queued: bool,
    remote_config: &mut RemoteConfig,
    bucket: &RemoteConfigBucket,
) -> Effects {
    if queued {
        bucket_changed_effects::<E>(remote_config, bucket)
    } else {
        Effects::none().unchanged()
    }
}

fn bucket_changed_effects<E: Env + 'static>(
    remote_config: &mut RemoteConfig,
    bucket: &RemoteConfigBucket,
) -> Effects {
    Effects::msg(Msg::Internal(Internal::RemoteConfigBucketChanged))
        .unchanged()
        .join(flush_events::<E>(remote_config, bucket))
}

fn flush_events<E: Env + 'static>(
    remote_config: &mut RemoteConfig,
    bucket: &RemoteConfigBucket,
) -> Effects {
    let batch = bucket
        .outbox
        .iter()
        .take(REMOTE_CONFIG_EVENTS_BATCH_SIZE)
        .collect::<Vec<_>>();
    let Some(last_seq) = batch.last().map(|queued| queued.seq) else {
        return Effects::none().unchanged();
    };
    let now = E::now();
    let in_flight = remote_config
        .flushing_since
        .is_some_and(|flushing_since| is_within(flushing_since, REMOTE_CONFIG_RETRY_INTERVAL, now));
    if in_flight {
        return Effects::none().unchanged();
    }
    remote_config.flushing_since = Some(now);
    let request = APIRequest::ConfigEvents {
        platform: E::client_info().platform,
        events: batch
            .into_iter()
            .map(|queued| queued.event.to_owned())
            .collect(),
    };
    Effects::one(
        EffectFuture::Concurrent(
            fetch_api::<E, _, _, SuccessResponse>(&request)
                .map_err(CtxError::from)
                .and_then(|result| match result {
                    APIResult::Ok(result) => future::ok(result),
                    APIResult::Err(error) => future::err(CtxError::from(error)),
                })
                .map(move |result| {
                    Msg::Internal(Internal::RemoteConfigEventsResult(now, last_seq, result))
                })
                .boxed_env(),
        )
        .into(),
    )
    .unchanged()
}

fn fetch_remote_config<E: Env + 'static>(started_at: DateTime<Utc>) -> Effect {
    let url = API_URL.join("config").expect("url builder failed");
    let request = Request::get(url.as_str())
        .body(())
        .expect("request builder failed");
    EffectFuture::Concurrent(
        E::fetch::<_, RemoteConfigResponse>(request)
            .map(move |result| Msg::Internal(Internal::RemoteConfigResult(started_at, result)))
            .boxed_env(),
    )
    .into()
}

fn fetch_geo<E: Env + 'static>(started_at: DateTime<Utc>) -> Effect {
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, GeoResponse>(&APIRequest::GetGeo {})
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::GeoResult(started_at, result)))
            .boxed_env(),
    )
    .into()
}

fn push_remote_config_to_storage<E: Env + 'static>(bucket: &RemoteConfigBucket) -> Effect {
    let mut bucket = bucket.to_owned();
    let now = E::now();
    EffectFuture::Sequential(
        E::get_storage::<RemoteConfigBucket>(REMOTE_CONFIG_STORAGE_KEY)
            .then(move |stored| {
                if let Ok(Some(stored)) = stored {
                    bucket.union(stored);
                }
                bucket.prune(now);
                E::set_storage(REMOTE_CONFIG_STORAGE_KEY, Some(&bucket))
            })
            .map(|result| match result {
                Ok(_) => Msg::Event(Event::RemoteConfigPushedToStorage),
                Err(error) => Msg::Event(Event::Error {
                    error: CtxError::from(error),
                    source: Box::new(Event::RemoteConfigPushedToStorage),
                }),
            })
            .boxed_env(),
    )
    .into()
}
