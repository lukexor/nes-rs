use crate::{
    control_deck::{RENDER_HEIGHT, RENDER_WIDTH},
    NesResult,
};
use include_dir::{include_dir, Dir};
use pix_engine::PixEngine;
use state::NesState;
use view::{views::*, View};

mod event;
mod filesystem;
mod keybinding;
mod preferences;
mod state;
mod view;

const APP_NAME: &str = "TetaNES";
// This includes static assets as a binary during installation
const _STATIC_DIR: Dir = include_dir!("./static");
const ICON_PATH: &str = "static/tetanes_icon.png";
const DEFAULT_SCALE: u32 = 3;
const DEFAULT_WIDTH: u32 = DEFAULT_SCALE * RENDER_WIDTH;
const DEFAULT_HEIGHT: u32 = DEFAULT_SCALE * RENDER_HEIGHT;
const DEFAULT_VSYNC: bool = true;

pub struct Nes {
    title: String,
    width: u32,
    height: u32,
    should_close: bool,
    has_focus: bool,
    state: NesState,
    views: Vec<View>,
}

impl Nes {
    pub fn new() -> Self {
        let width = DEFAULT_WIDTH;
        let height = DEFAULT_HEIGHT;
        Self {
            title: APP_NAME.to_string(),
            width,
            height,
            should_close: false,
            has_focus: false,
            state: NesState::new(),
            views: vec![EmulationView::new(width, height).into()],
        }
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

impl Default for Nes {
    fn default() -> Self {
        Self::new()
    }
}
