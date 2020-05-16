use super::{ViewType, Viewable};
use crate::{
    control_deck::{RENDER_HEIGHT, RENDER_WIDTH},
    nes::{action::Action, filesystem, keybinding::Keybind, state::NesState},
    NesResult,
};
use pix_engine::{
    draw::Rect,
    event::{Key, PixEvent},
    image::{Image, ImageRef},
    pixel::{self, ColorType, Pixel},
    StateData,
};
use std::{ffi::OsStr, path::PathBuf};

const TEXTURE_NAME: &str = "open_rom";
const WIDTH: u32 = 3 * RENDER_WIDTH;
const HEIGHT: u32 = 3 * RENDER_HEIGHT;
const MAX_ROWS: usize = (HEIGHT as usize - 63) / 30 - 1;

pub struct OpenRomView {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    active: bool,
    selected: usize,
    scroll: usize,
    scroll_max: usize,
    paths: Vec<PathBuf>,
    image: ImageRef,
    keybindings: Vec<Keybind>,
}

impl OpenRomView {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
            active: false,
            selected: 0,
            scroll: 0,
            scroll_max: 1,
            paths: Vec::new(),
            image: Image::new_ref(WIDTH, HEIGHT),
            keybindings: Vec::new(),
        }
    }

    fn load_keybindings(&mut self) {
        let pressed = true;
        let repeat = true;
        let default_bindings = [
            (Key::Escape, pressed, !repeat, None, Action::CloseView),
            (
                Key::O,
                pressed,
                !repeat,
                Some(&[PixEvent::KeyPress(Key::Ctrl, true, false)][..]),
                Action::CloseView,
            ),
            (Key::Up, pressed, !repeat, None, Action::SelectUp),
            (Key::Up, pressed, repeat, None, Action::SelectUp),
            (Key::Down, pressed, !repeat, None, Action::SelectDown),
            (Key::Down, pressed, repeat, None, Action::SelectDown),
            (Key::Return, pressed, !repeat, None, Action::SelectPath),
        ];
        for (key, pressed, repeat, modifiers, action) in &default_bindings {
            let keybind = Keybind::new(
                PixEvent::KeyPress(*key, *pressed, *repeat),
                *modifiers,
                action.clone(),
            );
            self.keybindings.push(keybind);
        }
    }

    fn load_files(&mut self, state: &mut NesState) -> NesResult<()> {
        self.paths = Vec::new();
        if state.prefs.current_path.parent().is_some() {
            self.paths.push(PathBuf::from("../"));
        }
        // TODO: catch errors to display message
        self.paths
            .extend(filesystem::list_dirs(&state.prefs.current_path)?);
        self.paths
            .extend(filesystem::find_roms(&state.prefs.current_path)?);
        self.scroll_max = self.paths.len() + 1;
        Ok(())
    }

    fn render_view(&mut self, data: &mut StateData) -> NesResult<()> {
        data.set_draw_target(self.image.clone());
        data.fill(Pixel([0, 0, 0, 128]));
        let (mut x, mut y) = (0, 0);
        data.fill_rect(x, y, WIDTH, HEIGHT, pixel::VERY_DARK_GRAY);
        x += 3;
        y += 3;
        data.fill_rect(x, y, WIDTH, HEIGHT, pixel::DARK_GRAY);
        x += 10;
        y += 10;
        data.set_draw_scale(3);
        data.draw_string(x, y, "Open File", pixel::WHITE);
        y += 50;
        data.set_draw_scale(2);
        for (i, rom) in self.paths[self.scroll..].iter().enumerate() {
            if y > HEIGHT - 30 {
                break;
            }
            let color = if i == self.selected - self.scroll {
                pixel::BLUE
            } else {
                pixel::WHITE
            };
            if rom == &PathBuf::from("../") {
                data.draw_string(x, y, "../", color);
                y += 30;
            } else if let Some(path) = rom.file_name().and_then(|file| file.to_str()) {
                data.draw_string(x, y, &path.to_string(), color);
                y += 30;
            }
        }
        Ok(())
    }
}

impl Viewable for OpenRomView {
    fn on_start(&mut self, state: &mut NesState, data: &mut StateData) -> NesResult<bool> {
        data.create_texture(
            TEXTURE_NAME,
            ColorType::Rgba,
            Rect::new(0, 0, WIDTH, HEIGHT),
            Rect::new(self.x, self.y, self.width, self.height),
        )?;
        self.load_keybindings();
        self.load_files(state)?;
        Ok(true)
    }

    fn on_update(
        &mut self,
        _elapsed: f32,
        _state: &mut NesState,
        data: &mut StateData,
    ) -> NesResult<bool> {
        // Update view
        if self.active {
            self.render_view(data)?;
            data.copy_draw_target(TEXTURE_NAME)?;
        }
        Ok(true)
    }

    fn on_pause(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        // There's nothing to pause
        Ok(true)
    }

    fn on_resume(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        self.active = true;
        Ok(true)
    }

    fn on_stop(&mut self, _state: &mut NesState, _data: &mut StateData) -> NesResult<bool> {
        self.active = false;
        // TODO clean up resources created with on_start
        Ok(true)
    }

    fn handle_event(
        &mut self,
        event: &PixEvent,
        state: &mut NesState,
        _data: &mut StateData,
    ) -> NesResult<bool> {
        let keybind = self
            .keybindings
            .iter()
            .find(|&keybind| keybind.event == *event);
        if let Some(keybind) = keybind {
            let mut all_pressed = true;
            for modifier in &keybind.modifiers {
                all_pressed &= state.is_event_pressed(modifier);
            }

            if all_pressed {
                match keybind.action {
                    Action::SelectUp => {
                        if self.selected > 0 {
                            self.selected -= 1;
                            if self.scroll > 0 && self.selected < self.scroll + 1 {
                                self.scroll -= 1;
                            }
                        }
                    }
                    Action::SelectDown => {
                        if self.selected < self.paths.len() - 1 {
                            self.selected += 1;
                            if self.selected - self.scroll >= MAX_ROWS
                                && self.scroll < self.selected - MAX_ROWS
                            {
                                self.scroll += 1;
                            }
                        }
                    }
                    Action::SelectPath => {
                        let path = &self.paths[self.selected];
                        if path.is_dir() {
                            if path == &PathBuf::from("../") {
                                let current_path = &state.prefs.current_path;
                                if let Some(path) = current_path.parent() {
                                    state.prefs.current_path = path.to_path_buf();
                                }
                            } else {
                                state.prefs.current_path = path.clone();
                            }
                            self.load_files(state)?;
                            self.selected = 0;
                            self.scroll = 0;
                        } else if path.extension() == Some(OsStr::new("nes")) {
                            state.queue_action(Action::LoadRom(path.clone()));
                        }
                    }
                    _ => state.queue_action(keybind.action.clone()),
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn view_type(&self) -> ViewType {
        ViewType::OpenRom
    }
}
