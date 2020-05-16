use super::{view::Viewable, Nes};
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
                self.handle_event(&event, data)?;
            }
        }
        Ok(())
    }

    fn get_events(&self, data: &mut StateData) -> Vec<PixEvent> {
        data.poll()
    }

    fn handle_event(&mut self, event: &PixEvent, data: &mut StateData) -> NesResult<()> {
        for view in &mut self.views {
            if view.handle_event(event, &mut self.state, data)? {
                break;
            }
        }
        Ok(())
    }
}
