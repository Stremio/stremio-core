use crate::constants::OFFICIAL_ADDONS;
use crate::models::common::Loadable;
use crate::runtime::msg::{Internal, Msg};
use crate::runtime::{Effects, EnvError, Environment};
use crate::types::addon::{Descriptor, Manifest};
use futures::FutureExt;
use serde::Serialize;
use url::Url;

#[derive(Serialize)]
pub struct DescriptorLoadable {
    pub transport_url: Url,
    pub content: Loadable<Descriptor, EnvError>,
}

impl PartialEq for DescriptorLoadable {
    fn eq(&self, other: &Self) -> bool {
        self.transport_url == other.transport_url
    }
}

pub enum DescriptorAction<'a> {
    DescriptorRequested {
        transport_url: &'a Url,
    },
    ManifestRequestResult {
        transport_url: &'a Url,
        result: &'a Result<Manifest, EnvError>,
    },
}

pub fn descriptor_update<Env: Environment + 'static>(
    descriptor: &mut Option<DescriptorLoadable>,
    action: DescriptorAction,
) -> Effects {
    match action {
        DescriptorAction::DescriptorRequested { transport_url } => {
            if descriptor
                .as_ref()
                .map(|descriptor| &descriptor.transport_url)
                != Some(transport_url)
            {
                let transport_url = transport_url.to_owned();
                *descriptor = Some(DescriptorLoadable {
                    transport_url: transport_url.to_owned(),
                    content: Loadable::Loading,
                });
                Effects::one(
                    Env::addon_transport(&transport_url)
                        .manifest()
                        .map(move |result| {
                            Msg::Internal(Internal::ManifestRequestResult(transport_url, result))
                        })
                        .boxed_local(),
                )
            } else {
                Effects::none().unchanged()
            }
        }
        DescriptorAction::ManifestRequestResult {
            transport_url,
            result,
        } => match descriptor {
            Some(descriptor) if descriptor.transport_url == *transport_url => {
                descriptor.content = match result {
                    Ok(manifest) => Loadable::Ready(Descriptor {
                        transport_url: transport_url.to_owned(),
                        manifest: manifest.to_owned(),
                        flags: OFFICIAL_ADDONS
                            .iter()
                            .find(|descriptor| descriptor.transport_url == *transport_url)
                            .map(|descriptor| descriptor.flags.to_owned())
                            .unwrap_or_default(),
                    }),
                    Err(error) => Loadable::Err(error.to_owned()),
                };
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        },
    }
}
