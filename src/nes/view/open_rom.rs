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
const MAX_ROWS: usize = 19;
const MAX_RECENTS: usize = 2;

#[derive(PartialEq, Eq)]
enum OpenMode {
    NewFile,
    Recent,
}

pub struct OpenRomView {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    active: bool,
    selected: usize,
    scroll: usize,
    paths: Vec<PathBuf>,
    recent_roms: Vec<(PathBuf, PathBuf)>,
    mode: OpenMode,
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
            paths: Vec::new(),
            recent_roms: Vec::new(),
            mode: OpenMode::Recent,
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
            (Key::Right, pressed, !repeat, None, Action::SelectRight),
            (Key::Left, pressed, !repeat, None, Action::SelectLeft),
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

    fn load_paths(&mut self, state: &mut NesState) -> NesResult<()> {
        // Look up directories and .nes files in current_path
        self.paths = Vec::new();
        if state.prefs.current_path.parent().is_some() {
            self.paths.push(PathBuf::from("../"));
        }
        // TODO: catch errors to display message instead of propagating
        self.paths
            .extend(filesystem::list_dirs(&state.prefs.current_path)?);
        self.paths
            .extend(filesystem::find_roms(&state.prefs.current_path)?);

        // Load recently played games
        self.recent_roms = filesystem::get_recent_roms()?;
        if self.recent_roms.is_empty() {
            self.mode = OpenMode::NewFile;
        }
        Ok(())
    }

    fn render_view(&mut self, data: &mut StateData) -> NesResult<()> {
        data.set_draw_target(self.image.clone());
        let (mut x, mut y) = (20, 20);
        data.fill_rect(x, y, WIDTH - 2 * x, HEIGHT - 2 * y, pixel::DARK_GRAY);
        let border = 20;
        x += border;
        y += border;
        data.set_draw_scale(3);
        let file_w = 9 * 8 * 3 - 3;
        let file_color = if self.mode == OpenMode::NewFile {
            pixel::WHITE
        } else {
            pixel::GRAY
        };
        data.draw_string(x, y, "Open File", file_color);
        let pad = 2 * 8 * 3;
        let recent_color = if self.mode == OpenMode::Recent {
            pixel::WHITE
        } else {
            pixel::GRAY
        };
        data.draw_string(x + file_w + pad, y, "Open Recent", recent_color);
        let hr_color = pixel::VERY_DARK_GRAY;
        let hr_w = 3;
        let y_pad = 40;
        data.fill_rect(
            x + file_w + (pad / 2),
            border + 10,
            hr_w,
            y_pad + 10,
            hr_color,
        ); // Title separator
        y += y_pad;
        data.fill_rect(border + 10, y, WIDTH - 2 * (border + 10), hr_w, hr_color); // Title HR
        x += 10;
        y += 20;
        data.set_draw_scale(2);
        match self.mode {
            OpenMode::NewFile => {
                let max = std::cmp::min(self.paths.len(), self.scroll + MAX_ROWS);
                for (i, rom) in self.paths[self.scroll..max].iter().enumerate() {
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
            }
            OpenMode::Recent => {
                let max = std::cmp::min(self.recent_roms.len(), self.scroll + MAX_RECENTS);
                for (i, (rom, image)) in self.recent_roms[self.scroll..max].iter().enumerate() {
                    let color = if i == self.selected - self.scroll {
                        pixel::BLUE
                    } else {
                        pixel::WHITE
                    };
                    let mut rom = rom.clone();
                    rom.set_extension("");
                    if let Some(image) = image.to_str() {
                        let image = Image::from_file(image)?;
                        data.draw_image(x, y, &image);
                        y += image.height() * 2 + 10;
                    }
                    if let Some(path) = rom.file_name().and_then(|file| file.to_str()) {
                        data.draw_string(x, y, &path.to_string(), color);
                        y += 40;
                    }
                }
            }
        }

        // Darken edges
        let darken = Pixel([0, 0, 0, 128]);
        data.fill_rect(0, 0, WIDTH, border, darken); // Top border
        data.fill_rect(WIDTH - border, 0, WIDTH, HEIGHT, darken); // Right border
        data.fill_rect(0, HEIGHT - border, WIDTH, HEIGHT, darken); // Bottom border
        data.fill_rect(0, 0, border, HEIGHT, darken); // Left border

        // Window highlight
        let highlight = pixel::GRAY;
        let shadow = pixel::VERY_DARK_GRAY;
        let stroke = 3;
        let width = WIDTH - 2 * border;
        let height = HEIGHT - 2 * border;
        let left = border;
        let right = WIDTH - border;
        let top = border;
        let bottom = HEIGHT - border;
        data.fill_rect(right - stroke, top, stroke, height, shadow); // Right border
        data.fill_rect(left, top, stroke, height, highlight); // Left border
        data.fill_rect(left, bottom - stroke, width, stroke, shadow); // Bottom border
        data.fill_rect(left, top, width, stroke, highlight); // Top border
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
        self.load_paths(state)?;
        self.render_view(data)?;
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
        data: &mut StateData,
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
                        let (scroll_max, total) = match self.mode {
                            OpenMode::NewFile => (MAX_ROWS - 1, self.paths.len() - 1),
                            OpenMode::Recent => (MAX_RECENTS - 1, self.recent_roms.len() - 1),
                        };
                        if self.selected < total {
                            self.selected += 1;
                            // Should scroll when there is one item left in view
                            let should_scroll = self.selected - self.scroll >= scroll_max;
                            if should_scroll {
                                self.scroll += 1;
                            }
                        }
                    }
                    Action::SelectLeft => {
                        self.selected = 0;
                        self.scroll = 0;
                        self.mode = OpenMode::NewFile;
                    }
                    Action::SelectRight => {
                        self.selected = 0;
                        self.scroll = 0;
                        self.mode = OpenMode::Recent;
                    }
                    Action::SelectPath => {
                        let path = match self.mode {
                            OpenMode::NewFile => &self.paths[self.selected],
                            OpenMode::Recent => {
                                let (rom, _) = &self.recent_roms[self.selected];
                                rom
                            }
                        };
                        if path.is_dir() {
                            if path == &PathBuf::from("../") {
                                let current_path = &state.prefs.current_path;
                                if let Some(path) = current_path.parent() {
                                    state.prefs.current_path = path.to_path_buf();
                                }
                            } else {
                                state.prefs.current_path = path.clone();
                            }
                            self.load_paths(state)?;
                            self.selected = 0;
                            self.scroll = 0;
                        } else if path.extension() == Some(OsStr::new("nes")) {
                            state.queue_action(Action::LoadRom(path.clone()));
                        }
                    }
                    _ => state.queue_action(keybind.action.clone()),
                }
            }
            self.render_view(data)?;
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
