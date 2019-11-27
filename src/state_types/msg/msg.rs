use crate::state_types::msg::action::Action;
use crate::state_types::msg::event::Event;
use crate::state_types::msg::internal::Internal;
use derive_more::From;

#[derive(Debug, From)]
pub enum Msg {
    Action(Action),
    Internal(Internal),
    Event(Event),
}
