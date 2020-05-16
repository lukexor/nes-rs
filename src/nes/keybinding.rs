use pix_engine::event::PixEvent;

#[derive(Debug, Copy, Clone)]
pub struct Keybind {
    pub event: PixEvent,
    pub pressed: bool,
    pub repeat: bool,
    pub action: Action,
}

#[rustfmt::skip]
#[derive(Debug, Copy, Clone)]
pub enum Action {
    P1A, P1B, P1Select, P1Start, P1Up, P1Down, P1Left, P1Right,
    P2A, P2B, P2Select, P2Start, P2Up, P2Down, P2Left, P2Right,
    TogglePause, ToggleFastForward, IncSpeed, DecSpeed, Rewind, ToggleFullscreen,
    ToggleSound, ToggleNtscVideo, ToggleVsync, ToggleRecording, Screenshot,
    SetSaveSlot(u8), SaveState, LoadState,
    Quit, Reset, PowerCycle, HelpMenu, PreferenceMenu, OpenMenu,
    ToggleCpuDebug, TogglePpuDebug, IncLogLevel, DecLogLevel,
    IncDebugScanline(u8), DecDebugScanline(u8),
    DebugStepInto, DebugStepOver, DebugStepOut, DebugStepScanline, DebugStepFrame,
}

impl Keybind {
    pub fn new(event: PixEvent, pressed: bool, repeat: bool, action: Action) -> Self {
        Self {
            event,
            pressed,
            repeat,
            action,
        }
    }
}
