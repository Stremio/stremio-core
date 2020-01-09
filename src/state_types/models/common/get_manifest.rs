use crate::state_types::messages::{Internal, Msg};
use crate::state_types::{Effect, Environment};
use futures::{future, Future};

pub fn get_manifest<Env: Environment + 'static>(transport_url: &str) -> Effect {
    let transport_url = transport_url.to_owned();
    Box::new(
        Env::addon_transport(&transport_url)
            .manifest()
            .then(move |result| match result {
                Ok(_) => future::ok(Msg::Internal(Internal::ManifestResponse(
                    transport_url,
                    Box::new(result),
                ))),
                Err(_) => future::err(Msg::Internal(Internal::ManifestResponse(
                    transport_url,
                    Box::new(result),
                ))),
            }),
    )
}
