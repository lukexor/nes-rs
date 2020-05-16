use super::{
    action::Action,
    filesystem,
    preferences::Preferences,
    view::{ViewType, Viewable},
    Nes,
};
use crate::{common::Powered, NesResult};
use pix_engine::{event::PixEvent, PixEngineResult, State, StateData};
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
};

pub struct NesState {
    pub prefs: Preferences,
    pub loaded_rom: Option<PathBuf>,
    action_queue: VecDeque<Action>,
    held_keys: HashSet<String>,
    held_buttons: HashSet<String>,
}

impl NesState {
    pub fn with_prefs(prefs: Preferences) -> NesResult<Self> {
        let mut roms = filesystem::find_roms(&prefs.current_path)?;
        let loaded_rom = if roms.len() == 1 { roms.pop() } else { None };
        Ok(Self {
            prefs,
            loaded_rom,
            action_queue: VecDeque::new(),
            held_keys: HashSet::new(),
            held_buttons: HashSet::new(),
        })
    }

    pub fn set_event_pressed(&mut self, event: &PixEvent) {
        match event {
            PixEvent::KeyPress(key, ..) => {
                let key = format!("{}", *key as u8);
                self.held_keys.insert(key);
            }
            PixEvent::GamepadBtn(id, btn, ..) => {
                let key = format!("{}{}", id, *btn as u8);
                self.held_buttons.insert(key);
            }
            _ => (),
        }
    }

    pub fn is_event_pressed(&self, event: &PixEvent) -> bool {
        match event {
            PixEvent::KeyPress(key, ..) => {
                let key = format!("{}", *key as u8);
                self.held_keys.get(&key).is_some()
            }
            PixEvent::GamepadBtn(id, btn, ..) => {
                let key = format!("{}{}", id, *btn as u8);
                self.held_buttons.get(&key).is_some()
            }
            _ => false,
        }
    }

    pub fn queue_action(&mut self, action: Action) {
        self.action_queue.push_back(action);
    }
}

impl Nes {
    fn update_title(&mut self, _data: &mut StateData) -> NesResult<()> {
        // TODO update_title
        Ok(())
    }

    fn check_has_focus(&mut self, data: &mut StateData) -> NesResult<()> {
        if self.state.prefs.pause_in_bg {
            if !self.has_focus {
                // If we don't have focus and not already paused, pause while
                // in BG
                if !self.paused {
                    self.bg_paused = true;
                }
                self.paused = true;
                if let Some(view) = self.views.last_mut() {
                    view.on_pause(&mut self.state, data)?;
                }
            } else if self.bg_paused {
                // Otherwise unpause if we paused due to backgrounding
                self.bg_paused = false;
                self.paused = false;
                if let Some(view) = self.views.last_mut() {
                    view.on_resume(&mut self.state, data)?;
                }
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, data: &mut StateData) -> NesResult<()> {
        while let Some(action) = self.state.action_queue.pop_front() {
            match action {
                Action::OpenView(view_type) => self.open_view(view_type, data)?,
                Action::CloseView => self.close_view(data)?,
                _ => (),
            }
        }
        Ok(())
    }
}

impl State for Nes {
    fn on_start(&mut self, data: &mut StateData) -> PixEngineResult<bool> {
        self.power_on();
        // Initial view is emulation only if a given rom is passed in on the command line
        // Queuing this will call on_start
        if self.state.loaded_rom.is_some() {
            self.state
                .queue_action(Action::OpenView(ViewType::Emulation));
        } else {
            self.state.queue_action(Action::OpenView(ViewType::OpenRom));
        }
        Ok(true)
    }

    fn on_update(&mut self, elapsed: f32, data: &mut StateData) -> PixEngineResult<bool> {
        self.poll_events(data)?;
        self.check_has_focus(data)?;
        self.handle_action(data)?;
        self.update_title(data)?;
        if !self.should_close {
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
