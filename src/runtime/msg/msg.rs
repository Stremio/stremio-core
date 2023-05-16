use crate::runtime::msg::{Action, Event, Internal};

#[derive(Debug)]
pub enum Msg {
    Action(Action),
    Internal(Internal),
    Event(Event),
}
