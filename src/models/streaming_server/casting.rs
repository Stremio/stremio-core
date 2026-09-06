use futures::FutureExt;
use http::Request;
use serde::Serialize;
use serde_json::{json, Value};
use url::Url;

use crate::{
    models::{ctx::CtxError, streaming_server::StreamingServer},
    runtime::{
        msg::{CastingSubtitles, Event, Internal, Msg, PlayOnDeviceArgs},
        Effect, EffectFuture, Effects, Env, EnvError, EnvFutureExt,
    },
};

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum CastingStatus {
    Starting,
    Playing,
    Stopping,
    Err(EnvError),
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CastingSession {
    pub id: u64,
    pub device: String,
    pub status: CastingStatus,
    #[serde(skip)]
    transport_url: Url,
}

#[derive(Clone, Debug)]
enum Command {
    Play(PlayOnDeviceArgs),
    Subtitles(CastingSubtitles),
    Stop,
}

#[derive(Clone, Debug)]
pub(super) struct CastingRequest {
    session: CastingSession,
    command: Command,
}

impl StreamingServer {
    pub(super) fn start_casting<E: Env + 'static>(&mut self, args: &PlayOnDeviceArgs) -> Effects {
        if Url::parse(&args.source).is_err()
            || !self.playback_devices.ready().is_some_and(|devices| {
                devices.iter().any(|device| {
                    device.id == args.device
                        && matches!(device.r#type.as_str(), "chromecast" | "tv")
                })
            })
        {
            return cast_error(
                args.device.clone(),
                Command::Play(args.clone()),
                EnvError::Other("Casting device or source is unavailable".to_owned()),
            )
            .unchanged()
            .join(if self.casting.is_none() {
                Effects::msg(Msg::Event(Event::StoppedCasting {
                    device: args.device.clone(),
                }))
                .unchanged()
            } else {
                Effects::none().unchanged()
            });
        }
        let was_idle = self.casting_requests.is_empty();
        self.queue_cast_stop();
        self.casting_generation += 1;
        let session = CastingSession {
            id: self.casting_generation,
            device: args.device.clone(),
            status: CastingStatus::Starting,
            transport_url: self.selected.transport_url.clone(),
        };
        self.casting = Some(session.clone());
        self.casting_requests.push_back(CastingRequest {
            session,
            command: Command::Play(args.clone()),
        });
        if was_idle {
            self.next_cast_request::<E>()
        } else {
            Effects::none()
        }
    }

    // Keep the in-flight request, discard obsolete queued work, then stop the
    // session that may already have reached the device. Its original URL is retained.
    fn queue_cast_stop(&mut self) {
        let mut first = true;
        self.casting_requests.retain(|request| {
            let keep = first || matches!(request.command, Command::Stop);
            first = false;
            keep
        });
        if let Some(session) = &mut self.casting {
            let in_flight = self
                .casting_requests
                .front()
                .filter(|request| request.session.id == session.id);
            if matches!(session.status, CastingStatus::Starting) && in_flight.is_none() {
                self.casting = self.casting_requests.back().map(|request| CastingSession {
                    status: CastingStatus::Stopping,
                    ..request.session.clone()
                });
                return;
            }
            session.status = CastingStatus::Stopping;
            if !self.casting_requests.iter().any(|request| {
                request.session.id == session.id && matches!(request.command, Command::Stop)
            }) {
                self.casting_requests.push_back(CastingRequest {
                    session: session.clone(),
                    command: Command::Stop,
                });
            }
        }
    }

    pub(super) fn stop_casting<E: Env + 'static>(&mut self) -> Effects {
        if self.casting.is_none() {
            return Effects::none().unchanged();
        }
        let was_idle = self.casting_requests.is_empty();
        self.queue_cast_stop();
        if was_idle {
            self.next_cast_request::<E>()
        } else {
            Effects::none()
        }
    }

    pub(super) fn set_casting_subtitles<E: Env + 'static>(
        &mut self,
        id: u64,
        subtitles: &CastingSubtitles,
    ) -> Effects {
        let Some(session) = self.casting.as_ref().filter(|session| {
            session.id == id
                && matches!(
                    session.status,
                    CastingStatus::Starting | CastingStatus::Playing
                )
        }) else {
            return Effects::none().unchanged();
        };
        let was_idle = self.casting_requests.is_empty();
        // Only the latest queued subtitle selection matters, including explicit Off.
        let mut first = true;
        self.casting_requests.retain(|request| {
            let keep = first
                || request.session.id != id
                || !matches!(request.command, Command::Subtitles(_));
            first = false;
            keep
        });
        self.casting_requests.push_back(CastingRequest {
            session: session.clone(),
            command: Command::Subtitles(subtitles.clone()),
        });
        if was_idle {
            self.next_cast_request::<E>().unchanged()
        } else {
            Effects::none().unchanged()
        }
    }

    pub(super) fn casting_result<E: Env + 'static>(
        &mut self,
        id: u64,
        result: &Result<(), EnvError>,
    ) -> Effects {
        if !self
            .casting_requests
            .front()
            .is_some_and(|request| request.session.id == id)
        {
            return Effects::none().unchanged();
        }
        let request = self
            .casting_requests
            .pop_front()
            .expect("cast request exists");
        let is_current = self
            .casting
            .as_ref()
            .is_some_and(|session| session.id == id);
        let mut effects = Effects::none().unchanged();
        match (&request.command, result) {
            (Command::Play(_), Ok(())) if is_current => {
                if let Some(session) = &mut self.casting {
                    if matches!(session.status, CastingStatus::Starting) {
                        session.status = CastingStatus::Playing;
                        effects = Effects::msg(Msg::Event(Event::PlayingOnDevice {
                            device: session.device.clone(),
                        }));
                    }
                }
            }
            (Command::Stop, Ok(())) if is_current => {
                self.casting = None;
                effects = Effects::msg(Msg::Event(Event::StoppedCasting {
                    device: request.session.device.clone(),
                }));
            }
            (Command::Stop, Err(error)) => {
                // Do not start another device or resume locally while the old one
                // may still be playing. Keep its identity so Stop can be retried.
                self.casting_requests.clear();
                self.casting = Some(CastingSession {
                    status: CastingStatus::Err(error.clone()),
                    ..request.session.clone()
                });
                effects = Effects::none();
            }
            (Command::Play(_), Err(_)) if is_current => {
                self.casting_requests.clear();
                if let Some(session) = &mut self.casting {
                    session.status = CastingStatus::Stopping;
                    self.casting_requests.push_back(CastingRequest {
                        session: session.clone(),
                        command: Command::Stop,
                    });
                }
                effects = Effects::none();
            }
            _ => {}
        }
        if let Err(error) = result {
            effects = effects.join(
                cast_error(request.session.device, request.command, error.clone()).unchanged(),
            );
        }
        effects.join(self.next_cast_request::<E>().unchanged())
    }

    fn next_cast_request<E: Env + 'static>(&self) -> Effects {
        self.casting_requests
            .front()
            .map_or_else(Effects::none, |request| {
                Effects::one(cast_request::<E>(request.clone()))
            })
    }
}

fn cast_error(device: String, command: Command, error: EnvError) -> Effects {
    let source = match command {
        Command::Play(_) => Event::PlayingOnDevice { device },
        Command::Subtitles(_) => Event::CastingSubtitlesChanged { device },
        Command::Stop => Event::StoppedCasting { device },
    };
    Effects::msg(Msg::Event(Event::Error {
        error: CtxError::Env(error),
        source: Box::new(source),
    }))
}

fn cast_request<E: Env + 'static>(request: CastingRequest) -> Effect {
    let mut endpoint = request.session.transport_url.clone();
    endpoint
        .path_segments_mut()
        .expect("streaming server URL has a path")
        .pop_if_empty()
        .extend(["casting", &request.session.device, "player"]);
    let id = request.session.id;
    EffectFuture::Concurrent(
        async move {
            match request.command {
                Command::Play(args) => {
                    send::<E>(
                        &endpoint,
                        json!({ "source": args.source, "time": args.time.unwrap_or(0) }),
                    )
                    .await?;
                    // Source initialization resets subtitle state in existing servers.
                    if let Some(subtitles) = args.subtitles {
                        send::<E>(&endpoint, json!(subtitles)).await?;
                    }
                    Ok(())
                }
                Command::Subtitles(subtitles) => send::<E>(&endpoint, json!(subtitles)).await,
                Command::Stop => send::<E>(&endpoint, json!({ "source": null })).await,
            }
        }
        .map(move |result| Msg::Internal(Internal::StreamingServerCastingResult(id, result)))
        .boxed_env(),
    )
    .into()
}

async fn send<E: Env + 'static>(endpoint: &Url, body: Value) -> Result<(), EnvError> {
    let request = Request::post(endpoint.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .expect("request builder failed");
    let response = E::fetch::<_, Value>(request).await?;
    // Older servers return cast failures as HTTP 200 strings or error objects.
    match response
        .as_str()
        .or_else(|| response.get("error").and_then(Value::as_str))
    {
        Some(error) => Err(EnvError::Other(error.to_owned())),
        None => Ok(()),
    }
}
