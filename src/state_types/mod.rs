use serde_derive::*;

mod state_container;
pub use self::state_container::*;

mod catalogs;
pub use self::catalogs::*;

mod actions;
pub use self::actions::*;

#[derive(Debug, Serialize)]
pub enum Loadable<T, M> {
    NotLoaded,
    Loading,
    Ready(T),
    Message(M),
}

// trait Middleware {
//  pub dispatch
//  @TODO better name? or the passthrough / opaque semantics should be in the construct phrase
//  pub connect_passthrough
//  pub connect_opaque
//  emit: fn
//  reactor: fn
//  }

