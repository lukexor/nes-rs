use super::Viewable;
use crate::{
    control_deck::{ControlDeck, RENDER_HEIGHT, RENDER_WIDTH},
    nes::state::NesState,
    NesResult,
};
use pix_engine::{draw::Rect, pixel::ColorType, StateData};
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
        }
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, _path: &P) -> NesResult<()> {
        // TODO convert to string
        // self.deck.load_rom(path)
        Ok(())
    }

    pub fn step_emulation(&mut self, elapsed: f32, state: &mut NesState, data: &mut StateData) {
        if !self.paused {
            self.running_time += elapsed;
            // TODO
            // // Frames that aren't multiples of the default render 1 more/less frames
            // // every other frame
            // let mut frames_to_run = 0;
            // self.speed_counter += (100.0 * self.config.speed) as i32;
            // while self.speed_counter > 0 {
            //     self.speed_counter -= 100;
            //     frames_to_run += 1;
            // }
            // Clock NES
            let frames_to_run = 1;
            for _ in 0..frames_to_run as usize {
                self.deck.clock_frame();
            }

            if state.preferences.sound_enabled {
                let audio_samples = self.deck.audio_samples();
                data.enqueue_audio(&audio_samples);
            }
            self.deck.clear_samples();
        }
    }

    pub fn update_image(&self, data: &mut StateData) -> NesResult<()> {
        // TODO change this to draw_image
        data.copy_texture(TEXTURE_NAME, &self.deck.frame())?;
        Ok(())
    }
}

impl Viewable for EmulationView {
    fn on_start(&mut self, _state: &mut NesState, data: &mut StateData) -> NesResult<bool> {
        // TODO
        let rom_name = "roms/castlevania_iii_draculas_curse.nes";
        let file = File::open(rom_name).expect("valid path");
        let mut buffer = BufReader::new(file);
        self.deck.load_rom(rom_name, &mut buffer)?;
        // TODO move the src Rect dimensions to video settings for trim top
        data.create_texture(
            TEXTURE_NAME,
            ColorType::Rgba,
            Rect::new(0, 8, RENDER_WIDTH, RENDER_HEIGHT - 8), // Trims overscan
            Rect::new(0, 0, self.width, self.height),
        )?;
        Ok(true)
    }

    fn on_update(
        &mut self,
        elapsed: f32,
        state: &mut NesState,
        data: &mut StateData,
    ) -> NesResult<bool> {
        self.step_emulation(elapsed, state, data);
        self.update_image(data)?;
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
