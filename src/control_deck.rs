use crate::{common::Clocked, NesResult};
use bus::Bus;
use cpu::Cpu;
use std::io::Read;

mod apu;
mod bus;
mod cpu;
mod filter;
pub mod mapper;
mod ppu;

pub use mapper::MapperType;
pub use ppu::{RENDER_HEIGHT, RENDER_WIDTH};

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
}

impl Clocked for ControlDeck {
    fn clock(&mut self) -> usize {
        self.cpu.clock()
    }
}

impl Default for ControlDeck {
    fn default() -> Self {
        Self::new()
    }
}
