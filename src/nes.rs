use crate::{
    control_deck::{RENDER_HEIGHT, RENDER_WIDTH},
    NesResult,
};
use include_dir::include_dir;
use pix_engine::prelude::*;
use preferences::Preferences;
use state::NesState;
use view::View;

pub mod preferences;

mod action;
mod event;
mod filesystem;
mod keybinding;
mod state;
mod view;

const APP_NAME: &str = "TetaNES";
// This includes static assets as a binary during installation
const ICON_PATH: &str = "static/tetanes_icon.png";
const DEFAULT_WIDTH: u32 = RENDER_WIDTH;
const DEFAULT_HEIGHT: u32 = RENDER_HEIGHT;

#[derive(Debug)]
pub struct Nes {
    title: String,
    width: u32,
    height: u32,
    scale: f32,
    should_close: bool,
    has_focus: bool,
    paused: bool,
    bg_paused: bool,
    state: NesState,
    views: Vec<View>,
}

impl Nes {
    pub fn with_prefs(prefs: Preferences) -> NesResult<Self> {
        let _ = include_dir!("./static");
        Ok(Self {
            title: APP_NAME.to_string(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            scale: prefs.scale,
            should_close: false,
            has_focus: false,
            paused: false,
            bg_paused: false,
            state: NesState::with_prefs(prefs)?,
            views: Vec::new(),
        })
    }

    pub fn run(&mut self) -> NesResult<()> {
        let mut engine = PixEngine::create(&self.title, self.width, self.height)
            .position_centered()
            .resizable()
            .scale(self.scale, self.scale)
            .vsync_enabled()
            .icon(ICON_PATH)
            .build()?;
        engine.run(self)?;
        Ok(())
    }
}
