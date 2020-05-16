use crate::{
    control_deck::{RENDER_HEIGHT, RENDER_WIDTH},
    NesResult,
};
use include_dir::{include_dir, Dir};
use pix_engine::PixEngine;
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
const _STATIC_DIR: Dir = include_dir!("./static");
const ICON_PATH: &str = "static/tetanes_icon.png";
const DEFAULT_WIDTH: u32 = RENDER_WIDTH;
const DEFAULT_HEIGHT: u32 = RENDER_HEIGHT;
const DEFAULT_VSYNC: bool = true;

pub struct Nes {
    title: String,
    width: u32,
    height: u32,
    should_close: bool,
    has_focus: bool,
    paused: bool,
    bg_paused: bool,
    state: NesState,
    views: Vec<View>,
}

impl Nes {
    pub fn with_prefs(prefs: Preferences) -> NesResult<Self> {
        let width = prefs.scale * DEFAULT_WIDTH;
        let height = prefs.scale * DEFAULT_HEIGHT;
        Ok(Self {
            title: APP_NAME.to_string(),
            width,
            height,
            should_close: false,
            has_focus: false,
            paused: false,
            bg_paused: false,
            state: NesState::with_prefs(prefs)?,
            views: Vec::new(),
        })
    }

    pub fn run(self) -> NesResult<()> {
        let nes_state = self;
        let title = nes_state.title.to_owned();
        let width = nes_state.width;
        let height = nes_state.height;

        let mut engine = PixEngine::new(title, nes_state, width, height, DEFAULT_VSYNC)?;
        let _ = engine.set_icon(ICON_PATH); // Ignore if this fails

        engine.run()?;

        Ok(())
    }
}
