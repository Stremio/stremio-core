use crate::state_types::messages::Internal;
use crate::state_types::models::common::Loadable;
use crate::state_types::{Effect, Effects, EnvError, Environment};
use crate::types::addons::{Manifest, TransportUrl};
use futures::{future, Future};
use serde::Serialize;

pub type ManifestContent = Loadable<Manifest, String>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ManifestLoadable {
    pub transport_url: TransportUrl,
    pub content: ManifestContent,
}

pub enum ManifestAction<'a> {
    ManifestRequested {
        transport_url: &'a TransportUrl,
    },
    ManifestReplaced {
        manifest: Option<ManifestLoadable>,
    },
    ManifestResponseReceived {
        transport_url: &'a TransportUrl,
        response: &'a Result<Manifest, EnvError>,
    },
}

pub fn manifest_update<Env>(
    manifest: &mut Option<ManifestLoadable>,
    action: ManifestAction,
) -> Effects
where
    Env: Environment + 'static,
{
    match action {
        ManifestAction::ManifestRequested { transport_url } => {
            if Some(transport_url).ne(&manifest.as_ref().map(|manifest| &manifest.transport_url)) {
                *manifest = Some(ManifestLoadable {
                    transport_url: transport_url.to_owned(),
                    content: ManifestContent::Loading,
                });
                Effects::one(get_manifest::<Env>(transport_url))
            } else {
                Effects::none().unchanged()
            }
        }
        ManifestAction::ManifestReplaced {
            manifest: next_manifest,
        } => {
            if next_manifest.ne(manifest) {
                *manifest = next_manifest;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        ManifestAction::ManifestResponseReceived {
            transport_url,
            response,
        } => match manifest {
            Some(manifest) if transport_url.eq(&manifest.transport_url) => {
                manifest.content = match response {
                    Ok(manifest) => ManifestContent::Ready(manifest.to_owned()),
                    Err(error) => ManifestContent::Err(error.to_string()),
                };
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        },
    }
}

fn get_manifest<Env: Environment + 'static>(transport_url: &TransportUrl) -> Effect {
    let transport_url = transport_url.to_owned();
    Box::new(
        Env::addon_transport(&transport_url)
            .manifest()
            .then(move |result| match result {
                Ok(_) => {
                    future::ok(Internal::ManifestResponse(transport_url, Box::new(result)).into())
                }
                Err(_) => {
                    future::err(Internal::ManifestResponse(transport_url, Box::new(result)).into())
                }
            }),
    )
}
