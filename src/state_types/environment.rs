use std::error::Error;
use futures::Future;
pub trait Environment {
    fn fetch(url: String) -> Box<Future<Item=Box<Vec<u8>>, Error=Box<Error>>>;
    // @TODO: get_storage
    // @TODO: set_storage
}
