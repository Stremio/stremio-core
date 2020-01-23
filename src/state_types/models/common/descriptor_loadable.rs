use super::{get_manifest, Loadable};
use crate::constants::OFFICIAL_ADDONS;
use crate::state_types::messages::{Internal, Msg, MsgError};
use crate::state_types::{Effects, Environment};
use crate::types::addons::{Descriptor, Manifest, TransportUrl};
use futures::{future, Future};
use serde::Serialize;

pub type DescriptorError = String;

pub type DescriptorContent = Loadable<Descriptor, DescriptorError>;

#[derive(Debug, Clone, Serialize)]
pub struct DescriptorLoadable {
    pub transport_url: TransportUrl,
    pub content: DescriptorContent,
}

impl PartialEq for DescriptorLoadable {
    fn eq(&self, other: &Self) -> bool {
        self.transport_url.eq(&other.transport_url)
    }
}

pub enum DescriptorAction<'a> {
    DescriptorRequested {
        transport_url: &'a TransportUrl,
    },
    ManifestRequestResult {
        transport_url: &'a TransportUrl,
        result: &'a Result<Manifest, MsgError>,
    },
}

pub fn descriptor_update<Env: Environment + 'static>(
    descriptor: &mut Option<DescriptorLoadable>,
    action: DescriptorAction,
) -> Effects {
    match action {
        DescriptorAction::DescriptorRequested { transport_url } => {
            if Some(transport_url).ne(&descriptor
                .as_ref()
                .map(|descriptor| &descriptor.transport_url))
            {
                let transport_url = transport_url.to_owned();
                *descriptor = Some(DescriptorLoadable {
                    transport_url: transport_url.to_owned(),
                    content: DescriptorContent::Loading,
                });
                Effects::one(Box::new(get_manifest::<Env>(&transport_url).then(
                    move |result| {
                        let msg = Msg::Internal(Internal::ManifestRequestResult(
                            transport_url,
                            Box::new(result),
                        ));
                        match result {
                            Ok(_) => future::ok(msg),
                            Err(_) => future::err(msg),
                        }
                    },
                )))
            } else {
                Effects::none().unchanged()
            }
        }
        DescriptorAction::ManifestRequestResult {
            transport_url,
            result,
        } => match descriptor {
            Some(descriptor) if transport_url.eq(&descriptor.transport_url) => {
                descriptor.content = match result {
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
