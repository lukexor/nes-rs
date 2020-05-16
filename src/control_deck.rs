use crate::{
    common::{Clocked, Powered},
    input::InputButton,
    serialization::Savable,
    NesResult,
};
use bus::Bus;
use cpu::Cpu;
use std::io::{Read, Write};

mod apu;
mod bus;
mod cpu;
mod filter;
pub mod mapper;
mod ppu;

pub use mapper::MapperType;
pub use ppu::{RENDER_HEIGHT, RENDER_WIDTH};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NesFormat {
    NTSC,
    PAL,
    DENDY,
}

pub struct ControlDeck {
    cpu: Cpu,
}

impl ControlDeck {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(Bus::new()),
        }
    }

    pub fn load_rom<F: Read>(&mut self, name: &str, rom: &mut F) -> NesResult<()> {
        let mapper = mapper::load_rom(name, rom)?;
        // TODO Refactor this
        self.cpu.bus.load_mapper(mapper);
        Ok(())
    }

    pub fn clock_frame(&mut self) {
        // TODO Refactor this
        while !self.cpu.bus.ppu.frame_complete {
            let _ = self.clock();
        }
        self.cpu.bus.ppu.frame_complete = false;
    }

    pub fn frame(&self) -> &[u8] {
        // TODO Refactor this
        self.cpu.bus.ppu.frame()
    }

    pub fn audio_samples(&self) -> &[f32] {
        // TODO Refactor this
        self.cpu.bus.apu.samples()
    }

    pub fn clear_samples(&mut self) {
        // TODO Refactor this
        self.cpu.bus.apu.clear_samples();
    }

    pub fn input_button(&mut self, button: InputButton, pressed: bool) {
        let mut p1 = &mut self.cpu.bus.input.gamepad1;
        let mut p2 = &mut self.cpu.bus.input.gamepad2;
        match button {
            // TODO Turbo
            InputButton::P1A => p1.a = pressed,
            InputButton::P1B => p1.b = pressed,
            InputButton::P1Select => p1.select = pressed,
            InputButton::P1Start => p1.start = pressed,
            InputButton::P1Up => p1.up = pressed,
            InputButton::P1Down => p1.down = pressed,
            InputButton::P1Left => p1.left = pressed,
            InputButton::P1Right => p1.right = pressed,
            InputButton::P2A => p2.a = pressed,
            InputButton::P2B => p2.b = pressed,
            InputButton::P2Select => p2.select = pressed,
            InputButton::P2Start => p2.start = pressed,
            InputButton::P2Up => p2.up = pressed,
            InputButton::P2Down => p2.down = pressed,
            InputButton::P2Left => p2.left = pressed,
            InputButton::P2Right => p2.right = pressed,
        }
    }
}

impl Clocked for ControlDeck {
    fn clock(&mut self) -> usize {
        self.cpu.clock()
    }
}

impl Powered for ControlDeck {
    fn power_on(&mut self) {
        // TODO
        // load sram
    }
    fn power_off(&mut self) {
        // TODO
        // save sram
    }
}

// TODO impl Savable for ControlDeck
impl Savable for ControlDeck {
    fn save<F: Write>(&self, fh: &mut F) -> NesResult<()> {
        Ok(())
    }
    fn load<F: Read>(&mut self, fh: &mut F) -> NesResult<()> {
        Ok(())
    }
}

impl Default for ControlDeck {
    fn default() -> Self {
        Self::new()
    }
}
