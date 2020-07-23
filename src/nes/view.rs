use crate::{nes::state::NesState, NesResult};
use enum_dispatch::enum_dispatch;
use pix_engine::{event::Event, State};
use views::*;

mod emulation;
mod open_rom;

pub mod views {
    pub use super::emulation::EmulationView;
    pub use super::open_rom::OpenRomView;
}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch]
#[derive(Debug)]
pub enum View {
    EmulationView,
    OpenRomView,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ViewType {
    Menu,
    Emulation,
    Help,
    OpenRom,
}

#[enum_dispatch(View)]
pub trait Viewable {
    fn on_start(&mut self, _state: &mut NesState, _s: &mut State) -> NesResult<bool> {
        Ok(true)
    }
    fn on_update(&mut self, _state: &mut NesState, _s: &mut State) -> NesResult<bool> {
        Ok(true)
    }
    fn on_stop(&mut self, _state: &mut NesState, _s: &mut State) -> NesResult<bool> {
        Ok(true)
    }
    fn on_pause(&mut self, _state: &mut NesState, _s: &mut State) -> NesResult<bool> {
        Ok(true)
    }
    fn on_resume(&mut self, _state: &mut NesState, _s: &mut State) -> NesResult<bool> {
        Ok(true)
    }
    fn handle_event(&mut self, _event: &Event, _state: &mut NesState, _s: &mut State) -> bool {
        false
    }
    fn is_active(&self) -> bool {
        false
    }
    fn view_type(&self) -> ViewType;
}
