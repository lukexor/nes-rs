use crate::{
    common::{Clocked, NesStandard, Powered},
    input::InputButton,
    map_nes_err,
    nes::preferences::Preferences,
    nes_err,
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

pub const ALL_AUDIO_CHANNELS: u16 = AudioChannel::Pulse1 as u16
    | AudioChannel::Pulse2 as u16
    | AudioChannel::Triangle as u16
    | AudioChannel::Noise as u16
    | AudioChannel::Dmc as u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoFilter {
    Standard,
    Pixellate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EmulationSpeed {
    S10,
    S50,
    S100,
    S150,
    S200,
    S300,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioChannel {
    Pulse1 = 1,
    Pulse2 = (1 << 1),
    Triangle = (1 << 2),
    Noise = (1 << 3),
    Dmc = (1 << 4),
}

#[derive(Debug)]
pub struct Config {
    emulation_speed: EmulationSpeed,
    randomize_start_ram: bool,
    unlimited_sprites: bool,
    concurrent_dpad: bool,
    cheats: Vec<String>,
    nes_standard: NesStandard,
    video_filter: VideoFilter,
    wide_nes: bool,
    audio_channels: u16, // Bitflag of AudioChannel
}

#[derive(Debug)]
pub struct ControlDeck {
    cpu: Cpu,
    config: Config,
}

impl ControlDeck {
    pub fn new() -> Self {
        let config = Config::default();
        Self {
            cpu: Cpu::new(Bus::new(config.randomize_start_ram)),
            config,
        }
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            cpu: Cpu::new(Bus::new(config.randomize_start_ram)),
            config,
        }
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
        // TODO self.cpu.bus.apu.set_speed(self.config.emulation_speed);
    }

    pub fn load_rom<F: Read>(&mut self, name: &str, rom: &mut F) -> NesResult<()> {
        let mapper = mapper::load_rom(name, rom, self.config.randomize_start_ram)?;
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
        let mut input = &mut self.cpu.bus.input.gamepads;
        match button {
            InputButton::PA(p) => input[p as usize - 1].a = pressed,
            InputButton::PB(p) => input[p as usize - 1].b = pressed,
            InputButton::PATurbo(p) => input[p as usize - 1].a_turbo = pressed,
            InputButton::PBTurbo(p) => input[p as usize - 1].b_turbo = pressed,
            InputButton::PSelect(p) => input[p as usize - 1].select = pressed,
            InputButton::PStart(p) => input[p as usize - 1].start = pressed,
            InputButton::PUp(p) => input[p as usize - 1].up = pressed,
            InputButton::PDown(p) => input[p as usize - 1].down = pressed,
            InputButton::PLeft(p) => input[p as usize - 1].left = pressed,
            InputButton::PRight(p) => input[p as usize - 1].right = pressed,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            emulation_speed: EmulationSpeed::S100,
            randomize_start_ram: false,
            unlimited_sprites: false,
            concurrent_dpad: false,
            cheats: Vec::new(),
            nes_standard: NesStandard::Ntsc,
            video_filter: VideoFilter::Standard,
            wide_nes: false,
            audio_channels: ALL_AUDIO_CHANNELS,
        }
    }
    pub fn from_prefs(prefs: &Preferences) -> Self {
        Self {
            emulation_speed: prefs.emulation_speed,
            randomize_start_ram: prefs.randomize_start_ram,
            unlimited_sprites: prefs.unlimited_sprites,
            concurrent_dpad: prefs.concurrent_dpad,
            cheats: prefs.cheats.clone(),
            nes_standard: prefs.nes_standard,
            video_filter: prefs.video_filter,
            wide_nes: prefs.wide_nes,
            audio_channels: prefs.audio_channels,
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

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
