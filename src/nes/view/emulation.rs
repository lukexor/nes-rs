use super::{ViewType, Viewable};
use crate::{
    common::Powered,
    control_deck::{Config, ControlDeck, RENDER_HEIGHT, RENDER_WIDTH},
    input::InputButton,
    map_nes_err,
    nes::{
        action::Action, event, filesystem, keybinding::Keybind, preferences::Preferences,
        state::NesState,
    },
    NesResult,
};
use chrono::prelude::{DateTime, Local};
use pix_engine::{draw::Rect, event::PixEvent, image::Image, pixel::ColorType, StateData};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

const TEXTURE_NAME: &str = "emulation";

pub struct EmulationView {
    scale: u32,
    deck: ControlDeck,
    loaded_title: String,
    running_time: f32,
    paused: bool,
}

impl EmulationView {
    pub fn new(prefs: &Preferences) -> Self {
        let scale = prefs.scale;
        let config = Config::from_prefs(prefs);
        Self {
            scale,
            deck: ControlDeck::with_config(config),
            loaded_title: String::new(),
            running_time: 0.0,
            paused: false,
        }
    }

    #[rustfmt::skip]
    pub fn default_keybindings() -> Vec<Keybind> {
        use Action::*;
        use pix_engine::event::{
            Key::*,
            PixEvent::*,
        };
        use ViewType::*;

        let mut binds: Vec<Keybind> = Vec::new();
        let press = true;
        let rpt = true;
        let no_mods = &[][..];
        let ctrl = &[KeyPress(Ctrl, press, !rpt)][..];

        // Player 1 Keyboard
        for pressed in [true, false].iter() {
            binds.push(Keybind::new(KeyPress(Z, *pressed, !rpt), Emulation, no_mods, PA(1)));
            binds.push(Keybind::new(KeyPress(X, *pressed, !rpt), Emulation, no_mods, PB(1)));
            binds.push(Keybind::new(KeyPress(A, *pressed, !rpt), Emulation, no_mods, PATurbo(1)));
            binds.push(Keybind::new(KeyPress(S, *pressed, !rpt), Emulation, no_mods, PBTurbo(1)));
            binds.push(Keybind::new(KeyPress(RShift, *pressed, !rpt), Emulation, no_mods, PSelect(1)));
            binds.push(Keybind::new(KeyPress(Return, *pressed, !rpt), Emulation, no_mods, PStart(1)));
            binds.push(Keybind::new(KeyPress(Up, *pressed, !rpt), Emulation, no_mods, PUp(1)));
            binds.push(Keybind::new(KeyPress(Down, *pressed, !rpt), Emulation, no_mods, PDown(1)));
            binds.push(Keybind::new(KeyPress(Left, *pressed, !rpt), Emulation, no_mods, PLeft(1)));
            binds.push(Keybind::new(KeyPress(Right, *pressed, !rpt), Emulation, no_mods, PRight(1)));
        }

        // Player 1-4 Controller
        // TODO

        // Menu Keyboard
        binds.push(Keybind::new(KeyPress(Escape, press, !rpt), Emulation, no_mods, OpenView(Menu)));
        binds.push(Keybind::new(KeyPress(F1, press, !rpt), Emulation, no_mods, OpenView(Help)));
        binds.push(Keybind::new(KeyPress(O, press, !rpt), Emulation, ctrl, OpenView(OpenRom)));
        binds.push(Keybind::new(KeyPress(Q, press, !rpt), Emulation, ctrl, Action::Quit));
        binds.push(Keybind::new(KeyPress(R, press, !rpt), Emulation, ctrl, Reset));
        binds.push(Keybind::new(KeyPress(P, press, !rpt), Emulation, ctrl, PowerCycle));
        binds.push(Keybind::new(KeyPress(Equals, press, !rpt), Emulation, ctrl, IncSpeed));
        binds.push(Keybind::new(KeyPress(Minus, press, !rpt), Emulation, ctrl, DecSpeed));
        binds.push(Keybind::new(KeyPress(Space, press, !rpt), Emulation, no_mods, FastForward));
        binds.push(Keybind::new(KeyPress(Space, !press, !rpt), Emulation, no_mods, FastForward));
        binds.push(Keybind::new(KeyPress(Num1, press, !rpt), Emulation, no_mods, SetSaveSlot(1)));
        binds.push(Keybind::new(KeyPress(Num2, press, !rpt), Emulation, no_mods, SetSaveSlot(2)));
        binds.push(Keybind::new(KeyPress(Num3, press, !rpt), Emulation, no_mods, SetSaveSlot(3)));
        binds.push(Keybind::new(KeyPress(Num4, press, !rpt), Emulation, no_mods, SetSaveSlot(4)));

        // Menu Controller
        // TODO

        binds
    }

    fn load_rom<P: AsRef<Path>>(&mut self, path: &P) -> NesResult<()> {
        let path = path.as_ref();
        let rom =
            File::open(path).map_err(|e| map_nes_err!("unable to open file {:?}: {}", path, e))?;
        let mut rom = BufReader::new(rom);
        if let Some(path) = path.file_name().and_then(|s| s.to_str()) {
            self.loaded_title = path.to_string();
        }
        self.deck
            .load_rom(&self.loaded_title, &mut rom)
            .map_err(|e| map_nes_err!("failed to load rom {:?}: {}", path, e))?;
        self.load_sram(&path)?;
        self.deck.power_on();
        Ok(())
    }

    fn unload_rom<P: AsRef<Path>>(&mut self, path: &P) -> NesResult<()> {
        let path = path.as_ref();
        self.save_sram(&path)?;
        self.deck.power_off();
        Ok(())
    }

    fn load_sram<P: AsRef<Path>>(&mut self, path: &P) -> NesResult<()> {
        if self.deck.uses_sram() {
            let sram_path = filesystem::sram_path(path)?;
            if sram_path.exists() {
                let sram_file = File::open(&sram_path)
                    .map_err(|e| map_nes_err!("failed to open file {:?}: {}", sram_path, e))?;
                let mut sram = BufReader::new(sram_file);
                self.deck.load_sram(&mut sram)?;
            }
        }
        Ok(())
    }

    fn save_sram<P: AsRef<Path>>(&mut self, path: &P) -> NesResult<()> {
        if self.deck.uses_sram() {
            let sram_path = filesystem::sram_path(path)?;
            let sram_dir = sram_path.parent().expect("sram path shouldn't be root"); // Safe to do because sram_path is never root
            if !sram_dir.exists() {
                std::fs::create_dir_all(sram_dir).map_err(|e| {
                    map_nes_err!("failed to create directory {:?}: {}", sram_dir, e)
                })?;
            }
            let mut sram_opts = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&sram_path)
                .map_err(|e| map_nes_err!("failed to open file {:?}: {}", sram_path, e))?;

            let exists = sram_opts.metadata()?.len() > 0;
            if exists {
                self.deck.validate_save(&mut sram_opts)?;
            }
            let mut sram = BufWriter::new(sram_opts);
            self.deck.save_sram(&mut sram, !exists)?;
        }
        Ok(())
    }

    fn step_emulation(&mut self, elapsed: f32, state: &mut NesState, data: &mut StateData) {
        if !self.paused {
            self.running_time += elapsed;
            self.deck.clock_frame();
            if state.prefs.sound_enabled {
                data.enqueue_audio(&self.deck.audio_samples());
            }
            self.deck.clear_samples();
        }
    }

    fn update_view(&self, data: &mut StateData) -> NesResult<()> {
        // TODO change this to draw_image
        data.copy_texture(TEXTURE_NAME, &self.deck.frame())?;
        Ok(())
    }

    fn configure_deck(&mut self, prefs: &Preferences) {
        let config = Config::from_prefs(&prefs);
        self.deck.set_config(config);
    }

    // TODO Scale screenshot to current width/height
    fn screenshot(&mut self) -> NesResult<()> {
        let datetime: DateTime<Local> = Local::now();
        let filename = datetime
            .format("Screen_Shot_%Y-%m-%d_at_%H_%M_%S")
            .to_string();
        let image = Image::from_bytes(RENDER_WIDTH, RENDER_HEIGHT, &self.deck.frame())?;
        image.save_to_file(&filename)?;
        println!("Saved screenshot: {:?}", filename);
        Ok(())
    }

    fn add_recent_rom<P: AsRef<Path>>(&mut self, path: &P) -> NesResult<()> {
        let image = Image::from_bytes(RENDER_WIDTH, RENDER_HEIGHT, &self.deck.frame())?;
        filesystem::add_recent_rom(path, image)?;
        Ok(())
    }

    fn handle_gamepad_input(&mut self, keybind: &Keybind) -> bool {
        let button = match keybind.action {
            Action::PA(player) => Some(InputButton::PA(player)),
            Action::PB(player) => Some(InputButton::PB(player)),
            Action::PSelect(player) => Some(InputButton::PSelect(player)),
            Action::PStart(player) => Some(InputButton::PStart(player)),
            Action::PUp(player) => Some(InputButton::PUp(player)),
            Action::PDown(player) => Some(InputButton::PDown(player)),
            Action::PLeft(player) => Some(InputButton::PLeft(player)),
            Action::PRight(player) => Some(InputButton::PRight(player)),
            _ => None, // No input for this action
        };
        if let Some(button) = button {
            if let PixEvent::KeyPress(_key, pressed, _repeat) = keybind.event {
                self.deck.input_button(button, pressed);
                return true;
            }
        }
        false
    }
}

impl Viewable for EmulationView {
    fn on_start(&mut self, state: &mut NesState, data: &mut StateData) -> NesResult<bool> {
        data.create_texture(
            TEXTURE_NAME,
            ColorType::Rgba,
            Rect::new(0, 0, RENDER_WIDTH, RENDER_HEIGHT),
            Rect::new(0, 0, self.scale * RENDER_WIDTH, self.scale * RENDER_HEIGHT),
        )?;

        self.configure_deck(&state.prefs);
        if let Some(rom) = &state.loaded_rom {
            self.load_rom(rom)?;
            self.deck.power_on();
        }
        Ok(true)
    }

    fn on_update(
        &mut self,
        elapsed: f32,
        state: &mut NesState,
        data: &mut StateData,
    ) -> NesResult<bool> {
        // TODO ability to adjust emulation speed, separate from frame rate
        self.step_emulation(elapsed, state, data);
        self.update_view(data)?;
        Ok(true)
    }

    fn on_stop(&mut self, state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // TODO save_replay
        self.paused = true;
        if let Some(rom) = &state.loaded_rom {
            self.add_recent_rom(rom)?;
            self.unload_rom(rom)?;
        }
        Ok(true)
    }

    fn on_pause(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        self.paused = true;
        // TODO add message overlay
        Ok(true)
    }

    fn on_resume(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        self.paused = false;
        // TODO remove message overlay
        Ok(true)
    }

    fn handle_event(
        &mut self,
        event: &PixEvent,
        state: &mut NesState,
        _data: &mut StateData,
    ) -> bool {
        if let Some(keybind) = event::match_keybinding(event, self.view_type(), state) {
            if !self.handle_gamepad_input(&keybind) {
                state.queue_action(keybind.action);
            }
            true
        } else {
            false
        }
    }

    fn view_type(&self) -> ViewType {
        ViewType::Emulation
    }
}
