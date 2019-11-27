use crate::state_types::messages::action::Action;
use crate::state_types::messages::event::Event;
use crate::state_types::messages::internal::Internal;
use derive_more::From;

#[derive(Debug, From)]
pub enum Msg {
    Action(Action),
    Internal(Internal),
    Event(Event),
}
