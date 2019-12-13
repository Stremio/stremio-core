use crate::constants::OFFICIAL_ADDONS;
use crate::state_types::messages::Internal;
use crate::state_types::models::common::Loadable;
use crate::state_types::{Effect, Effects, EnvError, Environment};
use crate::types::addons::{Descriptor, Manifest, TransportUrl};
use futures::{future, Future};
use serde::Serialize;

pub type DescriptorContent = Loadable<Descriptor, String>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DescriptorLoadable {
    pub transport_url: TransportUrl,
    pub content: DescriptorContent,
}

pub enum DescriptorAction<'a> {
    DescriptorRequested {
        transport_url: &'a TransportUrl,
    },
    DescriptorReplaced {
        descriptor: Option<DescriptorLoadable>,
    },
    ManifestResponseReceived {
        transport_url: &'a TransportUrl,
        response: &'a Result<Manifest, EnvError>,
    },
}

pub fn descriptor_update<Env>(
    descriptor: &mut Option<DescriptorLoadable>,
    action: DescriptorAction,
) -> Effects
where
    Env: Environment + 'static,
{
    match action {
        DescriptorAction::DescriptorRequested { transport_url } => {
            if Some(transport_url).ne(&descriptor
                .as_ref()
                .map(|descriptor| &descriptor.transport_url))
            {
                *descriptor = Some(DescriptorLoadable {
                    transport_url: transport_url.to_owned(),
                    content: DescriptorContent::Loading,
                });
                Effects::one(get_manifest::<Env>(transport_url))
            } else {
                Effects::none().unchanged()
            }
        }
        DescriptorAction::DescriptorReplaced {
            descriptor: next_descriptor,
        } => {
            if next_descriptor.ne(descriptor) {
                *descriptor = next_descriptor;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        DescriptorAction::ManifestResponseReceived {
            transport_url,
            response,
        } => match descriptor {
            Some(descriptor) if transport_url.eq(&descriptor.transport_url) => {
                descriptor.content = match response {
                    Ok(manifest) => DescriptorContent::Ready(Descriptor {
                        transport_url: transport_url.to_owned(),
                        manifest: manifest.to_owned(),
                        flags: OFFICIAL_ADDONS
                            .iter()
                            .find(|descriptor| descriptor.transport_url.eq(transport_url))
                            .map(|descriptor| descriptor.flags.to_owned())
                            .unwrap_or_default(),
                    }),
                    Err(error) => DescriptorContent::Err(error.to_string()),
                };
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        },
    }
}

fn get_manifest<Env: Environment + 'static>(transport_url: &str) -> Effect {
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
