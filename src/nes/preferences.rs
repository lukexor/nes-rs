pub struct Preferences {
    pub sound_enabled: bool,
}

impl Preferences {
    pub fn new() -> Self {
        Self {
            sound_enabled: true,
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new()
    }
}
