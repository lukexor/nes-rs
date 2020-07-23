use super::{
    action::Action,
    event, filesystem,
    keybinding::Keybind,
    preferences::Preferences,
    view::{ViewType, Viewable},
    Nes,
};
use crate::{common::Powered, NesResult};
use pix_engine::{event::Event, PixResult, State, Stateful};
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
};

#[derive(Debug)]
pub struct NesState {
    pub prefs: Preferences,
    pub loaded_rom: Option<PathBuf>,
    pub keybindings: Vec<Keybind>,
    action_queue: VecDeque<Action>,
    held_keys: HashSet<String>,
    held_buttons: HashSet<String>,
}

impl NesState {
    pub fn with_prefs(mut prefs: Preferences) -> NesResult<Self> {
        let mut roms = filesystem::find_roms(&prefs.current_path)?;
        if prefs.current_path.is_file() {
            prefs.current_path = prefs
                .current_path
                .parent()
                .expect("file path should have a parent")
                .to_path_buf();
        }
        let loaded_rom = if roms.len() == 1 { roms.pop() } else { None };
        let keybindings = filesystem::load_keybindings().unwrap_or_else(event::default_keybindings);
        Ok(Self {
            prefs,
            loaded_rom,
            keybindings,
            action_queue: VecDeque::new(),
            held_keys: HashSet::new(),
            held_buttons: HashSet::new(),
        })
    }

    pub fn set_event_pressed(&mut self, event: &Event) {
        // match event {
        //     Event::KeyPress(key, ..) => {
        //         let key = format!("{}", *key as u8);
        //         self.held_keys.insert(key);
        //     }
        //     Event::GamepadBtn(id, btn, ..) => {
        //         let key = format!("{}{}", id, *btn as u8);
        //         self.held_buttons.insert(key);
        //     }
        //     _ => (),
        // }
    }

    pub fn is_event_pressed(&self, event: &Event) -> bool {
        false
        // match event {
        //     Event::KeyPress(key, ..) => {
        //         let key = format!("{}", *key as u8);
        //         self.held_keys.get(&key).is_some()
        //     }
        //     Event::GamepadBtn(id, btn, ..) => {
        //         let key = format!("{}{}", id, *btn as u8);
        //         self.held_buttons.get(&key).is_some()
        //     }
        //     _ => false,
        // }
    }

    pub fn queue_action(&mut self, action: Action) {
        self.action_queue.push_back(action);
    }
}

impl Nes {
    fn update_title(&mut self, _s: &mut State) -> NesResult<()> {
        // TODO update_title
        Ok(())
    }

    fn check_has_focus(&mut self, s: &mut State) -> NesResult<()> {
        // if self.state.prefs.pause_in_bg {
        //     if !self.has_focus {
        //         // If we don't have focus and not already paused, pause while
        //         // in BG
        //         if !self.paused {
        //             self.bg_paused = true;
        //         }
        //         self.paused = true;
        //         if let Some(view) = self.views.last_mut() {
        //             view.on_pause(&mut self.state, s)?;
        //         }
        //     } else if self.bg_paused {
        //         // Otherwise unpause if we paused due to backgrounding
        //         self.bg_paused = false;
        //         self.paused = false;
        //         if let Some(view) = self.views.last_mut() {
        //             view.on_resume(&mut self.state, s)?;
        //         }
        //     }
        // }
        Ok(())
    }

    fn handle_action(&mut self, s: &mut State) -> NesResult<()> {
        while let Some(action) = self.state.action_queue.pop_front() {
            match action {
                Action::OpenView(view_type) => self.open_view(view_type, s)?,
                Action::CloseView => self.close_view(s)?,
                Action::LoadRom(rom) => self.load_rom(rom, s)?,
                _ => (),
            }
        }
        Ok(())
    }
}

impl Stateful for Nes {
    fn on_start(&mut self, _s: &mut State) -> PixResult<bool> {
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

    fn on_update(&mut self, s: &mut State) -> PixResult<bool> {
        self.poll_events(s)?;
        self.check_has_focus(s)?;
        self.handle_action(s)?;
        self.update_title(s)?;
        if !self.should_close {
            for view in &mut self.views {
                view.on_update(&mut self.state, s)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn on_stop(&mut self, s: &mut State) -> PixResult<bool> {
        for view in &mut self.views {
            view.on_stop(&mut self.state, s)?;
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
