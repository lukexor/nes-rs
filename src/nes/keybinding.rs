use super::{action::Action, view::ViewType};
use pix_engine::event::Event;

/// A mapping of a user event to a UI action
#[derive(Debug, Clone)]
pub struct Keybind {
    pub event: Event,
    pub view_type: ViewType,
    pub modifiers: Vec<Event>,
    pub action: Action,
}

impl Keybind {
    pub fn new(event: Event, view_type: ViewType, modifiers: &[Event], action: Action) -> Self {
        Self {
            event,
            view_type,
            modifiers: modifiers.to_vec(),
            action,
        }
    }
}
