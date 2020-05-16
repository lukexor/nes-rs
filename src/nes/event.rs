use super::{
    keybinding::Keybind,
    state::NesState,
    view::{views::*, ViewType, Viewable},
    Nes,
};
use crate::NesResult;
use pix_engine::{event::PixEvent, StateData};

impl Nes {
    pub fn poll_events(&mut self, data: &mut StateData) -> NesResult<()> {
        let events = self.get_events(data);
        for event in events {
            // Only process events if focused
            if let PixEvent::Focus(_, focused) = event {
                self.has_focus = focused;
            }
            if self.has_focus {
                self.state.set_event_pressed(&event);
                self.handle_event(&event, data)?;
            }
        }
        Ok(())
    }

    fn get_events(&self, data: &mut StateData) -> Vec<PixEvent> {
        data.poll()
    }

    fn handle_event(&mut self, event: &PixEvent, data: &mut StateData) -> NesResult<()> {
        for view in self.views.iter_mut().rev() {
            if view.handle_event(event, &mut self.state, data) {
                break;
            }
        }
        Ok(())
    }
}

pub fn match_keybinding(
    event: &PixEvent,
    view_type: ViewType,
    state: &NesState,
) -> Option<Keybind> {
    let keybind = state
        .keybindings
        .iter()
        .find(|&keybind| keybind.event == *event && keybind.view_type == view_type);
    if let Some(keybind) = keybind {
        let mut all_pressed = true;
        for modifier in &keybind.modifiers {
            all_pressed &= state.is_event_pressed(modifier);
        }

        if all_pressed {
            return Some(keybind.clone());
        }
    }
    None
}

#[rustfmt::skip]
pub fn default_keybindings() -> Vec<Keybind> {
    let mut binds: Vec<Keybind> = Vec::new();
    binds.extend(EmulationView::default_keybindings());
    binds.extend(OpenRomView::default_keybindings());
    binds
}
