use super::{view::Viewable, Nes};
use crate::NesResult;
use pix_engine::{event::PixEvent, StateData};

impl Nes {
    pub fn poll_events(&mut self, data: &mut StateData) -> NesResult<()> {
        let events = self.get_events(data);
        for view in &mut self.views {
            if view.handle_event(&mut self.state, data)? {
                break;
            }
        }
        Ok(())
    }

    fn get_events(&self, data: &mut StateData) -> Vec<PixEvent> {
        data.poll()
    }
}
