use crate::{
    common::{Clocked, Powered},
    input::InputButton,
    map_nes_err, nes_err,
    serialization::{validate_save_header, write_save_header, Savable},
    NesResult,
};
use bus::Bus;
use cpu::Cpu;
use mapper::Mapper;
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
    Ntsc,
    Pal,
    Dendy,
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

    pub fn load_sram(&mut self, mut sram: &mut dyn Read) -> NesResult<()> {
        match validate_save_header(&mut sram) {
            Ok(_) => self
                .cpu
                .bus
                .mapper
                .load_sram(&mut sram)
                .map_err(|e| map_nes_err!("failed to load save ram: {}", e)),
            Err(e) => nes_err!("failed to validate save ram header: {}.", e),
        }
    }

    pub fn save_sram(&mut self, mut sram: &mut dyn Write, is_new: bool) -> NesResult<()> {
        if is_new {
            write_save_header(&mut sram)
                .map_err(|e| map_nes_err!("failed to write save ram header: {}", e))?;
        }
        self.cpu
            .bus
            .mapper
            .save_sram(&mut sram)
            .map_err(|e| map_nes_err!("failed to write save ram: {}", e))
    }

    pub fn validate_save(&self, mut sram: &mut dyn Read) -> NesResult<()> {
        validate_save_header(&mut sram)
    }

    pub fn uses_sram(&self) -> bool {
        self.cpu.bus.mapper.battery_backed()
    }

    pub fn clock_frame(&mut self) {
        // TODO Refactor bus interaction
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
        for player in 1u8..=4 {
            let mut input = &mut self.cpu.bus.input.gamepads[player as usize - 1];
            match button {
                InputButton::PA(p) if p == player => input.a = pressed,
                InputButton::PB(p) if p == player => input.b = pressed,
                InputButton::PATurbo(p) if p == player => input.a_turbo = pressed,
                InputButton::PBTurbo(p) if p == player => input.b_turbo = pressed,
                InputButton::PSelect(p) if p == player => input.select = pressed,
                InputButton::PStart(p) if p == player => input.start = pressed,
                InputButton::PUp(p) if p == player => input.up = pressed,
                InputButton::PDown(p) if p == player => input.down = pressed,
                InputButton::PLeft(p) if p == player => input.left = pressed,
                InputButton::PRight(p) if p == player => input.right = pressed,
                _ => (),
            }
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
        self.cpu.power_on();
        // TODO load sram
    }
    fn power_off(&mut self) {
        self.cpu.power_off();
        // TODO save sram
    }
}

impl Savable for ControlDeck {
    fn save<F: Write>(&self, fh: &mut F) -> NesResult<()> {
        self.cpu.save(fh)?;
        Ok(())
    }
    fn load<F: Read>(&mut self, fh: &mut F) -> NesResult<()> {
        self.cpu.load(fh)?;
        Ok(())
    }
}

impl Default for ControlDeck {
    fn default() -> Self {
        Self::new()
    }
}
