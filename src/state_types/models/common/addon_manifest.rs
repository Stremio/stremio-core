use crate::state_types::messages::Internal;
use crate::state_types::{Effect, Environment};
use crate::types::addons::TransportUrl;
use futures::{future, Future};

pub fn addon_manifest<Env: Environment + 'static>(transport_url: TransportUrl) -> Effect {
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
