use super::{get_manifest, Loadable};
use crate::constants::OFFICIAL_ADDONS;
use crate::state_types::{Effects, EnvError, Environment};
use crate::types::addons::{Descriptor, Manifest, TransportUrl};
use serde::Serialize;

pub type DescriptorContent = Loadable<Descriptor, String>;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
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
