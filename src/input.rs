//! NES Controller Inputs

use crate::{
    common::Powered,
    memory::{MemRead, MemWrite},
};

// The "strobe state": the order in which the NES reads the buttons.
const STROBE_A: u8 = 0;
const STROBE_B: u8 = 1;
const STROBE_SELECT: u8 = 2;
const STROBE_START: u8 = 3;
const STROBE_UP: u8 = 4;
const STROBE_DOWN: u8 = 5;
const STROBE_LEFT: u8 = 6;
const STROBE_RIGHT: u8 = 7;

/// Player inputs (1-4)
#[rustfmt::skip]
#[derive(Debug, PartialEq, Eq)]
pub enum InputButton {
    PA(u8), PATurbo(u8), PB(u8), PBTurbo(u8), PSelect(u8), PStart(u8),
    PUp(u8), PDown(u8), PLeft(u8), PRight(u8),
}

/// Represents an NES Joypad
#[derive(Default, Debug, Copy, Clone)]
pub struct Gamepad {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub a_turbo: bool,
    pub b: bool,
    pub b_turbo: bool,
    pub select: bool,
    pub start: bool,
    pub strobe_state: u8,
}

#[derive(Default, Debug, Clone)]
pub struct Zapper {
    pub light_sense: bool,
    pub triggered: bool,
}

impl Gamepad {
    fn next_state(&mut self) -> u8 {
        let state = match self.strobe_state {
            STROBE_A => self.a,
            STROBE_B => self.b,
            STROBE_SELECT => self.select,
            STROBE_START => self.start,
            STROBE_UP => self.up,
            STROBE_DOWN => self.down,
            STROBE_LEFT => self.left,
            STROBE_RIGHT => self.right,
            _ => panic!("invalid state {}", self.strobe_state),
        };
        self.strobe_state = (self.strobe_state + 1) & 7;
        state as u8
    }
    fn peek_state(&self) -> u8 {
        let state = match self.strobe_state {
            STROBE_A => self.a,
            STROBE_B => self.b,
            STROBE_SELECT => self.select,
            STROBE_START => self.start,
            STROBE_UP => self.up,
            STROBE_DOWN => self.down,
            STROBE_LEFT => self.left,
            STROBE_RIGHT => self.right,
            _ => panic!("invalid state {}", self.strobe_state),
        };
        state as u8
    }
}

impl Powered for Gamepad {
    fn reset(&mut self) {
        self.strobe_state = STROBE_A;
    }
}

/// Input containing gamepad input state
#[derive(Default, Debug, Clone)]
pub struct Input {
    pub gamepads: [Gamepad; 4],
    pub zapper: Zapper,
    open_bus: u8,
}

impl Input {
    /// Returns an empty Input instance with no event pump
    pub fn new() -> Self {
        Self {
            gamepads: [Gamepad::default(); 4],
            zapper: Zapper::default(),
            open_bus: 0u8,
        }
    }
}

impl MemRead for Input {
    fn read(&mut self, addr: u16) -> u8 {
        let val = match addr {
            0x4016 => self.gamepads[0].next_state() | 0x40,
            0x4017 => self.gamepads[1].next_state() | 0x40,
            _ => self.open_bus,
        };
        self.open_bus = val;
        val
    }

    fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x4016 => self.gamepads[0].peek_state() | 0x40,
            0x4017 => self.gamepads[1].peek_state() | 0x40,
            _ => self.open_bus,
        }
    }
}

impl MemWrite for Input {
    fn write(&mut self, addr: u16, val: u8) {
        self.open_bus = val;
        if addr == 0x4016 && val == 0 {
            self.gamepads[0].reset();
            self.gamepads[1].reset();
        }
    }
}

impl Powered for Input {
    fn reset(&mut self) {
        self.gamepads[0].reset();
        self.gamepads[1].reset();
    }
}
