use super::{
    view::{views::*, View, ViewType, Viewable},
    Nes,
};
use crate::NesResult;
use pix_engine::StateData;
use std::path::PathBuf;

/// UI Actions that can be performed and mapped to user events
#[rustfmt::skip]
#[derive(Debug, Clone)]
pub enum Action {
    // Player inputs (1-4)
    PA(u8), PB(u8), PATurbo(u8), PBTurbo(u8), PSelect(u8), PStart(u8),
    PUp(u8), PDown(u8), PLeft(u8), PRight(u8),
    FastForward, IncSpeed, DecSpeed, Rewind, ToggleFullscreen,
    ToggleSound, ToggleNtscVideo, ToggleVsync, ToggleRecording, Screenshot,
    Tab, SelectUp, SelectDown, SelectLeft, SelectRight, SelectPath, LoadRom(PathBuf),
    CloseView, OpenView(ViewType),
    SetSaveSlot(u8), SaveState, LoadState, Quit, Reset, PowerCycle,
    IncLogLevel, DecLogLevel, DebugScanlineUp, DebugScanlineDown,
    DebugStepInto, DebugStepOver, DebugStepOut, DebugStepScanline,
    DebugStepFrame,
}

impl Nes {
    pub fn open_view(&mut self, view_type: ViewType, data: &mut StateData) -> NesResult<()> {
        if let Some(view) = self.views.last_mut() {
            view.on_pause(&mut self.state, data)?;
        }
        let mut view: View = match view_type {
            ViewType::Emulation => EmulationView::new(&self.state.prefs).into(),
            ViewType::OpenRom => OpenRomView::new(self.state.prefs.scale).into(),
            _ => unimplemented!("View not implemented yet"),
        };
        view.on_start(&mut self.state, data)?;
        view.on_resume(&mut self.state, data)?;
        self.views.push(view);
        Ok(())
    }

    pub fn close_view(&mut self, data: &mut StateData) -> NesResult<()> {
        if self.views.len() == 1 && self.views[0].view_type() != ViewType::OpenRom {
            let _ = self.views.pop();
            self.state.queue_action(Action::OpenView(ViewType::OpenRom));
        } else {
            if let Some(view) = &mut self.views.pop() {
                view.on_stop(&mut self.state, data)?;
            }
            if let Some(view) = self.views.last_mut() {
                view.on_resume(&mut self.state, data)?;
            }
        }
        Ok(())
    }

    pub fn load_rom(&mut self, rom: PathBuf, data: &mut StateData) -> NesResult<()> {
        while let Some(view) = &mut self.views.pop() {
            view.on_stop(&mut self.state, data)?;
        }
        self.state.loaded_rom = Some(rom);
        self.state
            .queue_action(Action::OpenView(ViewType::Emulation));
        Ok(())
    }
}
