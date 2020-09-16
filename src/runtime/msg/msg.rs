use crate::runtime::msg::{Action, Event, Internal};

pub enum Msg {
    Action(Action),
    Internal(Internal),
    Event(Event),
}
