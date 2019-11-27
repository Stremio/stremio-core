use crate::state_types::messages::Internal;
use crate::state_types::{Effect, Environment};
use crate::types::addons::ResourceRequest;
use futures::{future, Future};

pub fn addon_get<Env: Environment + 'static>(request: ResourceRequest) -> Effect {
    Box::new(Env::addon_transport(&request.base).get(&request.path).then(
        move |result| match result {
            Ok(_) => future::ok(Internal::AddonResponse(request, Box::new(result)).into()),
            Err(_) => future::err(Internal::AddonResponse(request, Box::new(result)).into()),
        },
    ))
}
