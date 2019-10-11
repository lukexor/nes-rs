//! Memory Map

use crate::console::apu::Apu;
use crate::console::ppu::Ppu;
use crate::input::InputRef;
use crate::mapper::{self, MapperRef};
use crate::serialization::Savable;
use crate::{nes_err, NesResult};
use rand::Rng;
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

pub static mut RANDOMIZE_RAM: bool = false;
const WRAM_SIZE: usize = 2 * 1024;

/// Memory Trait
pub trait Memory {
    fn read(&mut self, addr: u16) -> u8;
    fn peek(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    fn reset(&mut self);
    fn power_cycle(&mut self);
}

impl fmt::Debug for dyn Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

pub struct Ram(Vec<u8>);

impl Ram {
    pub fn init(size: usize) -> Self {
        let randomize = unsafe { RANDOMIZE_RAM };
        let ram = if randomize {
            let mut rng = rand::thread_rng();
            let mut ram = Vec::with_capacity(size);
            for _ in 0..size {
                ram.push(rng.gen_range(0x00, 0xFF));
            }
            ram
        } else {
            vec![0u8; size]
        };
        Self(ram)
    }
    pub fn null() -> Self {
        Self(Vec::new())
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
    pub fn from_vec(v: Vec<u8>) -> Self {
        Self(v)
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl Memory for Ram {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }
    fn peek(&self, addr: u16) -> u8 {
        if self.0.is_empty() {
            return 0;
        }
        let addr = addr as usize % self.0.len();
        self.0[addr]
    }
    fn write(&mut self, addr: u16, val: u8) {
        if self.0.is_empty() {
            return;
        }
        let addr = addr as usize % self.0.len();
        self.0[addr] = val;
    }
    fn reset(&mut self) {}
    fn power_cycle(&mut self) {}
}

impl Savable for Ram {
    fn save(&self, fh: &mut dyn Write) -> NesResult<()> {
        self.0.save(fh)
    }
    fn load(&mut self, fh: &mut dyn Read) -> NesResult<()> {
        self.0.load(fh)
    }
}

impl Bankable for Ram {
    fn chunks(&self, size: usize) -> Vec<Ram> {
        let mut chunks: Vec<Ram> = Vec::new();
        for slice in self.0.chunks(size) {
            chunks.push(Ram::from_bytes(slice));
        }
        chunks
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self::init(0)
    }
}

impl fmt::Debug for Ram {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "Ram {{ len: {} KB }}", self.0.len() / 1024)
    }
}

impl Deref for Ram {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl DerefMut for Ram {
    fn deref_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

pub struct Rom(Vec<u8>);

impl Rom {
    pub fn init(size: usize) -> Self {
        Self(vec![0u8; size as usize])
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
    pub fn from_vec(v: Vec<u8>) -> Self {
        Self(v)
    }
    pub fn to_ram(&self) -> Ram {
        Ram::from_vec(self.0.clone())
    }
}

impl Memory for Rom {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }
    fn peek(&self, addr: u16) -> u8 {
        if self.0.is_empty() {
            return 0;
        }
        let addr = addr as usize % self.0.len();
        self.0[addr]
    }
    fn write(&mut self, _addr: u16, _val: u8) {} // ROM is read-only
    fn reset(&mut self) {}
    fn power_cycle(&mut self) {}
}

impl Savable for Rom {
    fn save(&self, fh: &mut dyn Write) -> NesResult<()> {
        self.0.save(fh)
    }
    fn load(&mut self, fh: &mut dyn Read) -> NesResult<()> {
        self.0.load(fh)
    }
}

impl Bankable for Rom {
    fn chunks(&self, size: usize) -> Vec<Rom> {
        let mut chunks: Vec<Rom> = Vec::new();
        for slice in self.0.chunks(size) {
            chunks.push(Rom::from_bytes(slice));
        }
        chunks
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Rom {
    fn default() -> Self {
        Self::init(0)
    }
}

impl fmt::Debug for Rom {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "Rom {{ len: {} KB }}", self.0.len() / 1024)
    }
}

impl Deref for Rom {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

pub trait Bankable
where
    Self: std::marker::Sized,
{
    fn chunks(&self, size: usize) -> Vec<Self>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub struct Banks<T>
where
    T: Memory + Bankable,
{
    banks: Vec<T>,
    pub size: usize,
}

impl<T> Banks<T>
where
    T: Memory + Bankable,
{
    pub fn new() -> Self {
        Self {
            banks: vec![],
            size: 0usize,
        }
    }

    pub fn init(data: &T, size: usize) -> Self {
        let mut banks: Vec<T> = Vec::with_capacity(data.len());
        if data.len() > 0 {
            for bank in data.chunks(size) {
                banks.push(bank);
            }
        }
        Self { banks, size }
    }
}

impl<T> fmt::Debug for Banks<T>
where
    T: Memory + Bankable,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(
            f,
            "Rom {{ len: {}, size: {} KB  }}",
            self.banks.len(),
            self.size / 1024,
        )
    }
}

impl<T> Deref for Banks<T>
where
    T: Memory + Bankable,
{
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.banks
    }
}

impl<T> DerefMut for Banks<T>
where
    T: Memory + Bankable,
{
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.banks
    }
}

impl<T> Default for Banks<T>
where
    T: Memory + Bankable,
{
    fn default() -> Self {
        Self::new()
    }
}

struct GenieCode {
    code: String,
    data: u8,
    compare: Option<u8>,
}

/// CPU Memory Map
///
/// [http://wiki.nesdev.com/w/index.php/CPU_memory_map]()
pub struct MemoryMap {
    pub wram: Ram,
    open_bus: u8,
    pub ppu: Ppu,
    pub apu: Apu,
    pub mapper: MapperRef,
    input: InputRef,
    genie_codes: HashMap<u16, GenieCode>,
    genie_map: HashMap<char, u8>,
}

impl MemoryMap {
    pub fn init(input: InputRef) -> Self {
        let mut genie_map = HashMap::new();
        genie_map.insert('A', 0x0);
        genie_map.insert('P', 0x1);
        genie_map.insert('Z', 0x2);
        genie_map.insert('L', 0x3);
        genie_map.insert('G', 0x4);
        genie_map.insert('I', 0x5);
        genie_map.insert('T', 0x6);
        genie_map.insert('Y', 0x7);
        genie_map.insert('E', 0x8);
        genie_map.insert('O', 0x9);
        genie_map.insert('X', 0xA);
        genie_map.insert('U', 0xB);
        genie_map.insert('K', 0xC);
        genie_map.insert('S', 0xD);
        genie_map.insert('V', 0xE);
        genie_map.insert('N', 0xF);

        Self {
            wram: Ram::init(WRAM_SIZE),
            open_bus: 0u8,
            ppu: Ppu::new(),
            apu: Apu::new(),
            input,
            mapper: mapper::null(),
            genie_codes: HashMap::new(),
            genie_map,
        }
    }

    pub fn load_mapper(&mut self, mapper: MapperRef) {
        self.mapper = mapper.clone();
        self.ppu.load_mapper(mapper.clone());
        self.apu.load_mapper(mapper);
    }

    pub fn add_genie_code(&mut self, code: &str) -> NesResult<()> {
        if code.len() != 6 && code.len() != 8 {
            return nes_err!("invalid game genie code length");
        }
        let mut hex: Vec<u8> = Vec::with_capacity(code.len());
        for s in code.chars() {
            if let Some(h) = self.genie_map.get(&s) {
                hex.push(*h);
            } else {
                return nes_err!("invalid game genie code");
            }
        }
        let addr = 0x8000
            + (((u16::from(hex[3]) & 7) << 12)
                | ((u16::from(hex[5]) & 7) << 8)
                | ((u16::from(hex[4]) & 8) << 8)
                | ((u16::from(hex[2]) & 7) << 4)
                | ((u16::from(hex[1]) & 8) << 4)
                | (u16::from(hex[4]) & 7)
                | (u16::from(hex[3]) & 8));
        let data = if hex.len() == 6 {
            ((hex[1] & 7) << 4) | ((hex[0] & 8) << 4) | (hex[0] & 7) | (hex[5] & 8)
        } else {
            ((hex[1] & 7) << 4) | ((hex[0] & 8) << 4) | (hex[0] & 7) | (hex[7] & 8)
        };
        let compare = if hex.len() == 8 {
            Some(((hex[7] & 7) << 4) | ((hex[6] & 8) << 4) | (hex[6] & 7) | (hex[5] & 8))
        } else {
            None
        };
        self.genie_codes.insert(
            addr,
            GenieCode {
                code: code.to_string(),
                data,
                compare,
            },
        );
        Ok(())
    }

    pub fn remove_genie_code(&mut self, code: &str) {
        self.genie_codes.retain(|_, gc| gc.code != code);
    }

    fn genie_code(&self, addr: u16) -> Option<&GenieCode> {
        if self.genie_codes.is_empty() {
            None
        } else {
            self.genie_codes.get(&addr)
        }
    }
}

impl Memory for MemoryMap {
    fn read(&mut self, addr: u16) -> u8 {
        // Order of frequently accessed
        let val = match addr {
            // Start..End => Read memory
            0x0000..=0x1FFF => self.wram.read(addr & 0x07FF), // 0x0800..=0x1FFFF are mirrored
            0x4020..=0xFFFF => {
                if let Some(gc) = self.genie_code(addr) {
                    if let Some(compare) = gc.compare {
                        let val = self.mapper.borrow_mut().read(addr);
                        if val == compare {
                            gc.data
                        } else {
                            val
                        }
                    } else {
                        gc.data
                    }
                } else {
                    self.mapper.borrow_mut().read(addr)
                }
            }
            0x4000..=0x4013 | 0x4015 => self.apu.read(addr),
            0x4016..=0x4017 => self.input.borrow_mut().read(addr),
            0x2000..=0x3FFF => self.ppu.read(addr & 0x2007), // 0x2008..=0x3FFF are mirrored
            0x4018..=0x401F => self.open_bus,                // APU/IO Test Mode
            0x4014 => self.open_bus,
        };
        self.open_bus = val;
        val
    }

    fn peek(&self, addr: u16) -> u8 {
        // Order of frequently accessed
        match addr {
            // Start..End => Read memory
            0x0000..=0x1FFF => self.wram.peek(addr & 0x07FF), // 0x0800..=0x1FFFF are mirrored
            0x4020..=0xFFFF => {
                if let Some(gc) = self.genie_code(addr) {
                    if let Some(compare) = gc.compare {
                        let val = self.mapper.borrow_mut().read(addr);
                        if val == compare {
                            gc.data
                        } else {
                            val
                        }
                    } else {
                        gc.data
                    }
                } else {
                    self.mapper.borrow_mut().read(addr)
                }
            }
            0x4000..=0x4013 | 0x4015 => self.apu.peek(addr),
            0x4016..=0x4017 => self.input.borrow().peek(addr),
            0x2000..=0x3FFF => self.ppu.peek(addr & 0x2007), // 0x2008..=0x3FFF are mirrored
            0x4018..=0x401F => self.open_bus,                // APU/IO Test Mode
            0x4014 => self.open_bus,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.open_bus = val;
        // Order of frequently accessed
        match addr {
            // Start..End => Read memory
            0x0000..=0x1FFF => self.wram.write(addr & 0x07FF, val), // 0x8000..=0x1FFFF are mirrored
            0x4020..=0xFFFF => self.mapper.borrow_mut().write(addr, val),
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu.write(addr, val),
            0x4016 => self.input.borrow_mut().write(addr, val),
            0x2000..=0x3FFF => self.ppu.write(addr & 0x2007, val), // 0x2008..=0x3FFF are mirrored
            0x4018..=0x401F => (),                                 // APU/IO Test Mode
            0x4014 => (),                                          // Handled inside the CPU
        }
    }

    fn reset(&mut self) {
        self.apu.reset();
        self.ppu.reset();
        self.mapper.borrow_mut().reset();
    }
    fn power_cycle(&mut self) {
        self.apu.power_cycle();
        self.ppu.power_cycle();
        self.mapper.borrow_mut().power_cycle();
    }
}

impl Savable for MemoryMap {
    fn save(&self, fh: &mut dyn Write) -> NesResult<()> {
        self.wram.save(fh)?;
        self.open_bus.save(fh)?;
        self.ppu.save(fh)?;
        self.apu.save(fh)?;
        self.mapper.borrow().save(fh)
    }
    fn load(&mut self, fh: &mut dyn Read) -> NesResult<()> {
        self.wram.load(fh)?;
        self.open_bus.load(fh)?;
        self.ppu.load(fh)?;
        self.apu.load(fh)?;
        self.mapper.borrow_mut().load(fh)
    }
}

impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryMap {{ }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_offset() {
        // RAM
        let start = 0x0000;
        let end = 0x07FF;

        let mirror_start = 0x0800;
        let mirror_end = 0x1FFF;

        for addr in mirror_start..=mirror_end {
            let addr = addr & end;
            assert!(addr >= start && addr <= end, "Addr within range");
        }

        // PPU
        let start = 0x2000;
        let end = 0x2007;

        let mirror_start = 0x2008;
        let mirror_end = 0x3FFF;

        for addr in mirror_start..=mirror_end {
            let addr = addr & end;
            assert!(addr >= start && addr <= end, "Addr within range");
        }
    }

    #[test]
    fn test_cpu_memory() {
        use crate::input::Input;
        use crate::mapper;
        use std::cell::RefCell;
        use std::path::PathBuf;
        use std::rc::Rc;

        let test_rom = "tests/cpu/nestest.nes";
        let rom = PathBuf::from(test_rom);
        let mapper = mapper::load_rom(rom).expect("loaded mapper");
        let input = Rc::new(RefCell::new(Input::new()));
        let mut mem = MemoryMap::init(input);
        mem.load_mapper(mapper);
        mem.write(0x0005, 0x0015);
        mem.write(0x0015, 0x0050);
        mem.write(0x0016, 0x0025);

        assert_eq!(mem.read(0x0008), 0x00, "read uninitialized byte: 0x00");
        assert_eq!(
            mem.read(0x0005),
            0x15,
            "read initialized byte: 0x{:02X}",
            0x15
        );
        assert_eq!(
            mem.read(0x0808),
            0x00,
            "read uninitialized mirror1 byte: 0x00"
        );
        assert_eq!(
            mem.read(0x0805),
            0x15,
            "read initialized mirror1 byte: 0x{:02X}",
            0x15,
        );
        assert_eq!(
            mem.read(0x1008),
            0x00,
            "read uninitialized mirror2 byte: 0x00"
        );
        assert_eq!(
            mem.read(0x1005),
            0x15,
            "read initialized mirror2 byte: 0x{:02X}",
            0x15,
        );
        // The following are test mode addresses, Not mapped
        assert_eq!(mem.read(0x0418), 0x00, "read unmapped byte: 0x00");
        assert_eq!(mem.read(0x0418), 0x00, "write unmapped byte: 0x00");
    }
}
