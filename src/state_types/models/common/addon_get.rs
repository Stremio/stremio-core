use crate::state_types::messages::{Internal, Msg};
use crate::state_types::{Effect, Environment};
use crate::types::addons::ResourceRequest;
use futures::{future, Future};

pub fn addon_get<Env: Environment + 'static>(request: ResourceRequest) -> Effect {
    Box::new(Env::addon_transport(&request.base).get(&request.path).then(
        move |result| match result {
            Ok(_) => future::ok(Msg::Internal(Internal::AddonResponse(
                request,
                Box::new(result),
            ))),
            Err(_) => future::err(Msg::Internal(Internal::AddonResponse(
                request,
                Box::new(result),
            ))),
        },
    ))
}
