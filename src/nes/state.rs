use super::{preferences::Preferences, view::Viewable, Nes};
use crate::common::Powered;
use pix_engine::{PixEngineResult, State, StateData};

pub struct NesState {
    // TODO
    pub preferences: Preferences,
}

impl NesState {
    pub fn new() -> Self {
        Self {
            preferences: Preferences::default(),
        }
    }
}

impl State for Nes {
    fn on_start(&mut self, data: &mut StateData) -> PixEngineResult<bool> {
        self.power_on();
        for view in &mut self.views {
            view.on_start(&mut self.state, data)?;
        }
        Ok(true)
    }

    fn on_update(&mut self, elapsed: f32, data: &mut StateData) -> PixEngineResult<bool> {
        self.poll_events(data)?;
        if !self.should_close {
            self.update_title(data);
            for view in &mut self.views {
                view.on_update(elapsed, &mut self.state, data)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn on_stop(&mut self, data: &mut StateData) -> PixEngineResult<bool> {
        for view in &mut self.views {
            view.on_stop(&mut self.state, data)?;
        }
        self.power_off();
        Ok(true)
    }
}

impl Nes {
    pub fn update_title(&mut self, _data: &mut StateData) {
        // TODO
    }
}

impl Powered for Nes {
    fn power_on(&mut self) {
        // TODO
    }

    fn power_off(&mut self) {
        // TODO
    }

    fn reset(&mut self) {
        // TODO
    }

    fn power_cycle(&mut self) {
        // TODO
    }
}
