use super::action::Action;
use pix_engine::event::PixEvent;

/// A mapping of a user event to a UI action
#[derive(Debug, Clone)]
pub struct Keybind {
    pub event: PixEvent,
    pub modifiers: Vec<PixEvent>,
    pub action: Action,
}

impl Keybind {
    pub fn new(event: PixEvent, modifiers: Option<&[PixEvent]>, action: Action) -> Self {
        let modifiers = modifiers.unwrap_or(&[][..]).to_vec();
        Self {
            event,
            modifiers,
            action,
        }
    }
}
