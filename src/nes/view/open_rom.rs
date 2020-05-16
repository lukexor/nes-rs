use super::{ViewType, Viewable};
use crate::{
    nes::{action::Action, keybinding::Keybind, state::NesState},
    NesResult,
};
use pix_engine::{
    draw::Rect,
    event::{Key, PixEvent},
    image::{Image, ImageRef},
    pixel::{self, ColorType, Pixel},
    StateData,
};

const TEXTURE_NAME: &str = "open_rom";

pub struct OpenRomView {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    active: bool,
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
            image: Image::new_ref(width, height),
            keybindings: Vec::new(),
        }
    }

    fn load_keybindings(&mut self) {
        let pressed = true;
        let repeat = false;
        let default_bindings = [
            (Key::Escape, pressed, repeat, None, Action::CloseView),
            (
                Key::O,
                pressed,
                repeat,
                Some(&[PixEvent::KeyPress(Key::Ctrl, true, false)][..]),
                Action::CloseView,
            ),
        ];
        for (key, pressed, repeat, modifiers, action) in &default_bindings {
            let keybind = Keybind::new(
                PixEvent::KeyPress(*key, *pressed, false),
                *pressed,
                *repeat,
                *modifiers,
                *action,
            );
            self.keybindings.push(keybind);
        }
    }
}

impl Viewable for OpenRomView {
    fn on_start(&mut self, _state: &mut NesState, data: &mut StateData) -> NesResult<bool> {
        self.load_keybindings();

        data.set_draw_target(self.image.clone());
        data.fill(Pixel([0, 0, 0, 128]));
        let (mut x, mut y) = (50, 50);
        data.fill_rect(
            x,
            y,
            self.width - 100,
            self.height - 100,
            pixel::VERY_DARK_GRAY,
        );
        x += 3;
        y += 3;
        data.fill_rect(x, y, self.width - 106, self.height - 106, pixel::DARK_GRAY);
        x += 10;
        y += 10;
        data.set_draw_scale(3);
        data.draw_string(x, y, "Open Rom", pixel::WHITE);
        y += 50;
        data.set_draw_scale(2);
        data.draw_string(x, y, "Not yet implemented", pixel::WHITE);

        data.create_texture(
            TEXTURE_NAME,
            ColorType::Rgba,
            Rect::new(0, 0, self.width, self.height),
            Rect::new(self.x, self.y, self.width, self.height),
        )?;

        Ok(true)
    }

    fn on_update(
        &mut self,
        _elapsed: f32,
        _state: &mut NesState,
        data: &mut StateData,
    ) -> NesResult<bool> {
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
                state.queue_action(keybind.action);
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
