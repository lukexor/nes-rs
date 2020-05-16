use super::{
    view::{views::*, View, ViewType, Viewable},
    Nes,
};
use crate::NesResult;
use pix_engine::StateData;

/// UI Actions that can be performed and mapped to user events
#[rustfmt::skip]
#[derive(Debug, Copy, Clone)]
pub enum Action {
    // Player inputs (1-4)
    PA(u8), PB(u8), PATurbo(u8), PBTurbo(u8), PSelect(u8), PStart(u8),
    PUp(u8), PDown(u8), PLeft(u8), PRight(u8),
    TogglePause, ToggleFastForward, IncSpeed, DecSpeed, Rewind, ToggleFullscreen,
    ToggleSound, ToggleNtscVideo, ToggleVsync, ToggleRecording, Screenshot,
    // Save slot (1-4)
    SetSaveSlot(u8), SaveState, LoadState, Quit, Reset, PowerCycle,
    CloseView, OpenView(ViewType), IncLogLevel, DecLogLevel, DebugScanlineUp, DebugScanlineDown,
    DebugStepInto, DebugStepOver, DebugStepOut, DebugStepScanline, DebugStepFrame,
}

impl Nes {
    pub fn open_view(&mut self, view_type: ViewType, data: &mut StateData) -> NesResult<()> {
        if let Some(view) = self.views.last_mut() {
            view.on_pause(&mut self.state, data)?;
        }
        let mut view: View = match view_type {
            ViewType::Emulation => EmulationView::new(self.width, self.height).into(),
            ViewType::OpenRom => OpenRomView::new(self.width, self.height).into(),
        };
        view.on_start(&mut self.state, data)?;
        view.on_resume(&mut self.state, data)?;
        self.views.push(view);
        Ok(())
    }

    pub fn close_view(&mut self, data: &mut StateData) -> NesResult<()> {
        if let Some(view) = &mut self.views.pop() {
            view.on_stop(&mut self.state, data)?;
        }
        if let Some(view) = self.views.last_mut() {
            view.on_resume(&mut self.state, data)?;
        }
        Ok(())
    }
}
