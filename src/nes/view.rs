use crate::{nes::state::NesState, NesResult};
use enum_dispatch::enum_dispatch;
use pix_engine::StateData;
use views::*;

mod emulation;

pub mod views {
    pub use super::emulation::EmulationView;
}

#[enum_dispatch]
pub enum View {
    EmulationView,
    // TODO
    // Open,
    // Preferences,
    // HelpAbout,
}

#[enum_dispatch(View)]
pub trait Viewable {
    fn on_start(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // TODO
        Ok(true)
    }
    fn on_update(
        &mut self,
        _elapsed: f32,
        _state: &mut NesState,
        _data: &mut StateData,
    ) -> NesResult<bool> {
        // TODO
        Ok(true)
    }
    fn on_stop(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // TODO
        Ok(true)
    }
    fn on_pause(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // TODO
        Ok(true)
    }
    fn on_resume(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // TODO
        Ok(true)
    }
    fn handle_event(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // TODO
        Ok(false)
    }
}
