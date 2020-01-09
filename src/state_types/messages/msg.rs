use super::{Action, Event, Internal};
use derive_more::From;

#[derive(Debug, From)]
pub enum Msg {
    Action(Action),
    Internal(Internal),
    Event(Event),
}
