use super::{ViewType, Viewable};
use crate::{
    common::Powered,
    control_deck::{Config, ControlDeck, RENDER_HEIGHT, RENDER_WIDTH},
    input::InputButton,
    map_nes_err,
    nes::{
        action::Action, filesystem, keybinding::Keybind, preferences::Preferences, state::NesState,
    },
    NesResult,
};
use chrono::prelude::{DateTime, Local};
use pix_engine::{
    draw::Rect,
    event::{Key, PixEvent},
    image::Image,
    pixel::ColorType,
    StateData,
};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

const TEXTURE_NAME: &str = "emulation";

pub struct EmulationView {
    // TODO these could be moved to a common struct
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    deck: ControlDeck,
    loaded_title: String,
    running_time: f32,
    paused: bool,
    keybindings: Vec<Keybind>,
}

impl EmulationView {
    pub fn new(width: u32, height: u32, prefs: &Preferences) -> Self {
        let config = Config::from_prefs(prefs);
        Self {
            x: 0,
            y: 0,
            width,
            height,
            deck: ControlDeck::with_config(config),
            loaded_title: String::new(),
            running_time: 0.0,
            paused: false,
            keybindings: Vec::new(),
        }
    }

    fn load_rom<P: AsRef<Path>>(&mut self, path: &P) -> NesResult<()> {
        let path = path.as_ref();
        let rom =
            File::open(path).map_err(|e| map_nes_err!("unable to open file {:?}: {}", path, e))?;
        let mut rom = BufReader::new(rom);
        if let Some(path) = path.file_name().and_then(|s| s.to_str()) {
            self.loaded_title = path.to_string();
            println!("Loading {:?}", self.loaded_title);
        }
        self.deck
            .load_rom(&path.to_string_lossy(), &mut rom)
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

    fn load_keybindings(&mut self) {
        let pressed = true;
        let repeat = false;
        // TODO move this out to a file
        let default_bindings = [
            (Key::Z, pressed, repeat, None, Action::PA(1)),
            (Key::X, pressed, repeat, None, Action::PB(1)),
            (Key::RShift, pressed, repeat, None, Action::PSelect(1)),
            (Key::Return, pressed, repeat, None, Action::PStart(1)),
            (Key::Up, pressed, repeat, None, Action::PUp(1)),
            (Key::Down, pressed, repeat, None, Action::PDown(1)),
            (Key::Left, pressed, repeat, None, Action::PLeft(1)),
            (Key::Right, pressed, repeat, None, Action::PRight(1)),
            (Key::Z, !pressed, repeat, None, Action::PA(1)),
            (Key::X, !pressed, repeat, None, Action::PB(1)),
            (Key::RShift, !pressed, repeat, None, Action::PSelect(1)),
            (Key::Return, !pressed, repeat, None, Action::PStart(1)),
            (Key::Up, !pressed, repeat, None, Action::PUp(1)),
            (Key::Down, !pressed, repeat, None, Action::PDown(1)),
            (Key::Left, !pressed, repeat, None, Action::PLeft(1)),
            (Key::Right, !pressed, repeat, None, Action::PRight(1)),
            (
                Key::O,
                pressed,
                repeat,
                Some(&[PixEvent::KeyPress(Key::Ctrl, true, false)][..]),
                Action::OpenView(ViewType::OpenRom),
            ),
        ];

        for (key, pressed, repeat, modifiers, action) in &default_bindings {
            let keybind = Keybind::new(
                PixEvent::KeyPress(*key, *pressed, *repeat),
                *modifiers,
                action.clone(),
            );
            self.keybindings.push(keybind);
        }
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
}

impl Viewable for EmulationView {
    fn on_start(&mut self, state: &mut NesState, data: &mut StateData) -> NesResult<bool> {
        data.create_texture(
            TEXTURE_NAME,
            ColorType::Rgba,
            Rect::new(0, 0, RENDER_WIDTH, RENDER_HEIGHT), // Trims overscan
            Rect::new(self.x, self.y, self.width, self.height),
        )?;

        self.load_keybindings();
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
    ) -> NesResult<bool> {
        let keybind = self
            .keybindings
            .iter()
            .find(|&keybind| keybind.event == *event);
        if let Some(keybind) = keybind {
            for _ in 0..4 {
                let button = match keybind.action {
                    Action::PA(player) => Some(InputButton::PA(player)),
                    Action::PB(player) => Some(InputButton::PB(player)),
                    Action::PSelect(player) => Some(InputButton::PSelect(player)),
                    Action::PStart(player) => Some(InputButton::PStart(player)),
                    Action::PUp(player) => Some(InputButton::PUp(player)),
                    Action::PDown(player) => Some(InputButton::PDown(player)),
                    Action::PLeft(player) => Some(InputButton::PLeft(player)),
                    Action::PRight(player) => Some(InputButton::PRight(player)),
                    _ => None, // No  input for this action
                };
                if let Some(button) = button {
                    if let PixEvent::KeyPress(_key, pressed, _repeat) = keybind.event {
                        self.deck.input_button(button, pressed);
                        return Ok(true);
                    }
                }
            }

            let mut all_pressed = true;
            for modifier in &keybind.modifiers {
                all_pressed &= state.is_event_pressed(modifier);
            }

            if all_pressed {
                state.queue_action(keybind.action.clone());
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn view_type(&self) -> ViewType {
        ViewType::Emulation
    }
}
