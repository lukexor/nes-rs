use super::Viewable;
use crate::{
    control_deck::{ControlDeck, RENDER_HEIGHT, RENDER_WIDTH},
    input::InputButton,
    nes::{
        keybinding::{Action, Keybind},
        state::NesState,
    },
    NesResult,
};
use pix_engine::{
    draw::Rect,
    event::{Key, PixEvent},
    pixel::ColorType,
    StateData,
};
use std::{fs::File, io::BufReader, path::Path};

const TEXTURE_NAME: &str = "emulation";

pub struct EmulationView {
    // TODO these could be moved to a common struct
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    deck: ControlDeck,
    running_time: f32,
    paused: bool,
    keybindings: Vec<Keybind>,
}

impl EmulationView {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
            deck: ControlDeck::new(),
            running_time: 0.0,
            paused: false,
            keybindings: Vec::new(),
        }
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, _path: &P) -> NesResult<()> {
        // TODO convert Read filehandle
        // self.deck.load_rom(path)
        Ok(())
    }

    pub fn step_emulation(&mut self, elapsed: f32, state: &mut NesState, data: &mut StateData) {
        if !self.paused {
            self.running_time += elapsed;
            self.deck.clock_frame();
            if state.preferences.sound_enabled {
                data.enqueue_audio(&self.deck.audio_samples());
            }
            self.deck.clear_samples();
        }
    }

    pub fn update_view(&self, data: &mut StateData) -> NesResult<()> {
        // TODO change this to draw_image
        data.copy_texture(TEXTURE_NAME, &self.deck.frame())?;
        Ok(())
    }

    fn load_keybindings(&mut self) {
        let pressed = true;
        let repeat = false;
        // TODO move this out to a file
        let default_bindings = [
            (Key::Z, pressed, repeat, Action::P1A),
            (Key::X, pressed, repeat, Action::P1B),
            (Key::RShift, pressed, repeat, Action::P1Select),
            (Key::Return, pressed, repeat, Action::P1Start),
            (Key::Up, pressed, repeat, Action::P1Up),
            (Key::Down, pressed, repeat, Action::P1Down),
            (Key::Left, pressed, repeat, Action::P1Left),
            (Key::Right, pressed, repeat, Action::P1Right),
            (Key::Z, !pressed, repeat, Action::P1A),
            (Key::X, !pressed, repeat, Action::P1B),
            (Key::RShift, !pressed, repeat, Action::P1Select),
            (Key::Return, !pressed, repeat, Action::P1Start),
            (Key::Up, !pressed, repeat, Action::P1Up),
            (Key::Down, !pressed, repeat, Action::P1Down),
            (Key::Left, !pressed, repeat, Action::P1Left),
            (Key::Right, !pressed, repeat, Action::P1Right),
        ];

        for (key, pressed, repeat, action) in &default_bindings {
            let keybind = Keybind::new(
                PixEvent::KeyPress(*key, *pressed, false),
                *pressed,
                *repeat,
                *action,
            );
            self.keybindings.push(keybind);
        }
    }
}

impl Viewable for EmulationView {
    fn on_start(&mut self, _state: &mut NesState, data: &mut StateData) -> NesResult<bool> {
        // TODO
        let rom_name = "roms/castlevania_iii_draculas_curse.nes";
        let file = File::open(rom_name).expect("valid path");
        let mut buffer = BufReader::new(file);
        self.deck.load_rom(rom_name, &mut buffer)?;
        // TODO self.control_deck.power_on()
        // TODO move the src Rect dimensions to video settings for trim top
        data.create_texture(
            TEXTURE_NAME,
            ColorType::Rgba,
            Rect::new(0, 0, RENDER_WIDTH, RENDER_HEIGHT), // Trims overscan
            Rect::new(self.x, self.y, self.width, self.height),
        )?;

        self.load_keybindings();

        self.paused = false;

        Ok(true)
    }

    fn on_update(
        &mut self,
        elapsed: f32,
        state: &mut NesState,
        data: &mut StateData,
    ) -> NesResult<bool> {
        // TODO check if window is focused for pause in BG preference
        // TODO ability to adjust emulation speed, separate from frame rate
        self.step_emulation(elapsed, state, data);
        self.update_view(data)?;
        Ok(true)
    }

    fn on_stop(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // TODO save_replay
        // self.control_deck.power_off()
        self.paused = true;
        Ok(true)
    }

    fn on_pause(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        self.paused = !self.paused;
        // TODO add message overlay
        Ok(true)
    }

    fn on_resume(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        self.paused = !self.paused;
        // TODO remove message overlay
        Ok(true)
    }

    fn handle_event(
        &mut self,
        event: &PixEvent,
        state: &mut NesState,
        data: &mut StateData,
    ) -> NesResult<bool> {
        let keybind = self
            .keybindings
            .iter()
            .find(|&keybind| keybind.event == *event);
        if let Some(keybind) = keybind {
            let button = match keybind.action {
                Action::P1A => Some(InputButton::P1A),
                Action::P1B => Some(InputButton::P1B),
                Action::P1Select => Some(InputButton::P1Select),
                Action::P1Start => Some(InputButton::P1Start),
                Action::P1Up => Some(InputButton::P1Up),
                Action::P1Down => Some(InputButton::P1Down),
                Action::P1Left => Some(InputButton::P1Left),
                Action::P1Right => Some(InputButton::P1Right),
                Action::P2A => Some(InputButton::P2A),
                Action::P2B => Some(InputButton::P2B),
                Action::P2Select => Some(InputButton::P2Select),
                Action::P2Start => Some(InputButton::P2Start),
                Action::P2Up => Some(InputButton::P2Up),
                Action::P2Down => Some(InputButton::P2Down),
                Action::P2Left => Some(InputButton::P2Left),
                Action::P2Right => Some(InputButton::P2Right),
                _ => None, // No input for this action
            };
            if let Some(button) = button {
                self.deck.input_button(button, keybind.pressed);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
