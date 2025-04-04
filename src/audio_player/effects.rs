// Main effect manager to handle all audio effects
pub struct EffectManager {
    pub playback_speed: f32,
    pub volume: f32,
    pub lowpass_cutoff: u32,
    pub reverb_enabled: bool,
    pub reverb_delay: f32,
}

impl EffectManager {
    pub fn new() -> Self {
        Self {
            playback_speed: 1.0,
            volume: 1.0,
            lowpass_cutoff: 20000,
            reverb_enabled: false,
            reverb_delay: 0.06,
        }
    }

    // Volume methods
    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn change_volume(&mut self, increase: bool) {
        if increase {
            self.volume = (self.volume + 0.1).min(2.0);
        } else {
            self.volume = (self.volume - 0.1).max(0.0);
        }
    }

    // Pitch/speed methods
    pub fn get_playback_speed(&self) -> f32 {
        self.playback_speed
    }

    pub fn change_pitch(&mut self, increase: bool) {
        if increase {
            self.playback_speed = (self.playback_speed + 0.1).min(3.0);
        } else {
            self.playback_speed = (self.playback_speed - 0.1).max(0.1);
        }
    }

    // Lowpass filter methods
    pub fn get_lowpass_cutoff(&self) -> u32 {
        self.lowpass_cutoff
    }

    pub fn change_lowpass(&mut self, increase: bool) {
        if increase {
            self.lowpass_cutoff = (self.lowpass_cutoff + 500).min(20000);
        } else {
            self.lowpass_cutoff = (self.lowpass_cutoff - 500).max(500);
        }
    }

    // Reverb methods
    pub fn is_reverb_enabled(&self) -> bool {
        self.reverb_enabled
    }

    pub fn get_reverb_delay(&self) -> f32 {
        self.reverb_delay
    }

    pub fn toggle_reverb(&mut self) {
        self.reverb_enabled = !self.reverb_enabled;
    }
}
