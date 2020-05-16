use super::action::Action;
use pix_engine::event::PixEvent;

/// A mapping of a user event to a UI action
#[derive(Debug, Clone)]
pub struct Keybind {
    pub event: PixEvent,
    pub pressed: bool,
    pub repeat: bool,
    pub modifiers: Vec<PixEvent>,
    pub action: Action,
}

impl Keybind {
    pub fn new(
        event: PixEvent,
        pressed: bool,
        repeat: bool,
        modifiers: Option<&[PixEvent]>,
        action: Action,
    ) -> Self {
        let modifiers = modifiers.unwrap_or(&[][..]).to_vec();
        Self {
            event,
            pressed,
            repeat,
            modifiers,
            action,
        }
    }
}
