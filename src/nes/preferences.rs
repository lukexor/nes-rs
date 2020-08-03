use crate::{
    common::NesStandard,
    control_deck::{EmulationSpeed, VideoFilter, ALL_AUDIO_CHANNELS},
    NesResult,
};
use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Preferences {
    // General
    pub pause_in_bg: bool,
    pub emulation_speed: EmulationSpeed,
    pub randomize_start_ram: bool,
    pub unlimited_sprites: bool,
    pub concurrent_dpad: bool,
    pub savestates_enabled: bool,
    pub active_save_slot: SaveSlot,
    pub sound_enabled: bool,
    // TODO Input/Keybindings?

    // Gameplay
    pub current_path: PathBuf,
    pub recent_games: Vec<PathBuf>,
    pub action_recording: bool,
    pub action_playback: bool,
    pub cheats: Vec<String>,
    pub nes_standard: NesStandard,

    // TODO Multiplayer
    // pub client_ip
    // pub remote_ip

    // Video
    pub video_recording: bool,
    pub fullscreen: bool,
    pub start_fullscreen: bool,
    pub scale: f32,
    pub video_filter: VideoFilter,
    pub vsync: bool,
    pub show_fps: bool,
    trim_borders: u8,   // Bitflag of TrimBorder
    pub wide_nes: bool, // TODO wide_nes

    // Audio
    pub audio_recording: bool,
    pub audio_channels: u16, // Bitflag of AudioChannel

    // Debug
    pub debug_cpu: bool,
    pub debug_ppu: bool,
}

#[derive(Debug)]
pub enum SaveSlot {
    S1 = 1,
    S2 = 2,
    S3 = 3,
    S4 = 4,
}

#[derive(Debug)]
pub enum Scale {
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
}

#[derive(Debug)]
pub enum TrimBorder {
    TopBot = 1,
    LeftRight = (1 << 1),
}

impl Preferences {
    pub fn new(path: Option<PathBuf>) -> NesResult<Self> {
        Ok(Self {
            // General
            pause_in_bg: true,
            emulation_speed: EmulationSpeed::S100, // 100%
            randomize_start_ram: true,
            unlimited_sprites: false,
            concurrent_dpad: false,
            savestates_enabled: true,
            active_save_slot: SaveSlot::S1,
            sound_enabled: true,
            // TODO Input/Keybindings?

            // Gameplay
            current_path: path
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("./")))
                .canonicalize()?,
            recent_games: Vec::new(),
            action_recording: false,
            action_playback: false,
            cheats: Vec::new(),
            nes_standard: NesStandard::Ntsc,

            // TODO Multiplayer
            // client_ip
            // remote_ip

            // Video
            video_recording: false,
            fullscreen: false,
            start_fullscreen: false,
            scale: (Scale::X3 as u32) as f32,
            video_filter: VideoFilter::Standard,
            vsync: true,
            show_fps: false,
            trim_borders: TrimBorder::TopBot as u8,
            wide_nes: false,

            // Audio
            audio_recording: false,
            audio_channels: ALL_AUDIO_CHANNELS,

            // Debug
            debug_cpu: false,
            debug_ppu: false,
        })
    }
}