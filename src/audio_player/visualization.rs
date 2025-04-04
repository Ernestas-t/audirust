use crate::audio_player::effects::EffectManager;
use rodio::Sink;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct WaveformVisualizer {
    pub waveform_values: Vec<f32>,    // Values for sound wave visualization
    pub audio_samples: VecDeque<f32>, // Use a fixed-size buffer for recent samples
}

impl WaveformVisualizer {
    pub fn new(points: usize) -> Self {
        Self {
            waveform_values: vec![0.0; points],
            audio_samples: VecDeque::new(),
        }
    }

    pub fn update(
        &mut self,
        active_sinks: &[(Arc<Sink>, bool)],
        last_played: Option<Instant>,
        visual_only_mode: bool,
        effect_manager: &EffectManager,
    ) {
        // Reset waveform if no active sounds and last played was over 5 seconds ago
        if active_sinks.is_empty()
            && last_played.map_or(true, |t| t.elapsed() > Duration::from_secs(5))
        {
            for val in &mut self.waveform_values {
                *val = *val * 0.9; // Fade out
                if *val < 0.01 {
                    *val = 0.0;
                }
            }
            return;
        }

        let is_active = !active_sinks.is_empty();

        // Use actual audio samples if available
        if !self.audio_samples.is_empty() {
            self.update_from_samples(is_active, effect_manager);
        } else {
            // Fall back to simulated waveform
            self.simulate_waveform(is_active, visual_only_mode, effect_manager);
        }
    }

    fn update_from_samples(&mut self, is_active: bool, effect_manager: &EffectManager) {
        let waveform_len = self.waveform_values.len();
        let samples_len = self.audio_samples.len();

        // Map the audio samples to the waveform values
        for (i, val) in self.waveform_values.iter_mut().enumerate() {
            if is_active {
                // Calculate an appropriate index into the audio_samples buffer
                let sample_index = (i * samples_len / waveform_len).min(samples_len - 1);

                // Get the sample value and apply volume
                let sample = self.audio_samples[sample_index].abs() * effect_manager.get_volume();

                // Apply "visual" filter similar to lowpass
                let filter_factor = if effect_manager.get_lowpass_cutoff() < 20000 {
                    effect_manager.get_lowpass_cutoff() as f32 / 20000.0
                } else {
                    1.0
                };

                // Scale the sample based on current effects
                *val = (sample * filter_factor).min(1.0);

                // Add visual reverb effect if enabled
                if effect_manager.is_reverb_enabled() {
                    // Get a slightly offset sample for reverb effect
                    let reverb_index = (sample_index
                        + (effect_manager.get_reverb_delay() * 44100.0) as usize)
                        % samples_len;
                    let reverb_sample =
                        self.audio_samples[reverb_index].abs() * 0.3 * effect_manager.get_volume();
                    *val = (*val + reverb_sample).min(1.0);
                }
            } else {
                // Fade out
                *val = *val * 0.95;
                if *val < 0.01 {
                    *val = 0.0;
                }
            }
        }
    }

    fn simulate_waveform(
        &mut self,
        is_active: bool,
        visual_only_mode: bool,
        effect_manager: &EffectManager,
    ) {
        let time = Instant::now().elapsed().as_secs_f64();

        for (i, val) in self.waveform_values.iter_mut().enumerate() {
            if is_active || visual_only_mode {
                let x = i as f64 / 10.0;
                let base_wave = (time * 5.0 * effect_manager.get_playback_speed() as f64 + x).sin()
                    * effect_manager.get_volume() as f64;

                // Apply "visual" filter similar to lowpass
                let filter_factor = if effect_manager.get_lowpass_cutoff() < 20000 {
                    effect_manager.get_lowpass_cutoff() as f64 / 20000.0
                } else {
                    1.0
                };

                // Add harmonic for visuals
                let harmonic = (time * 10.0 * effect_manager.get_playback_speed() as f64 + x).sin()
                    * 0.3
                    * filter_factor;

                // Combine waves
                *val = (base_wave + harmonic).abs() as f32 * effect_manager.get_volume();

                // Add visual reverb effect
                if effect_manager.is_reverb_enabled() {
                    let reverb_wave =
                        (time * 5.0 * effect_manager.get_playback_speed() as f64 + x - 0.5).sin()
                            * 0.3
                            * effect_manager.get_volume() as f64;
                    *val += reverb_wave.abs() as f32;
                }

                // Normalize
                *val = (*val * 0.7).min(1.0);
            } else {
                // Fade out
                *val = *val * 0.95;
                if *val < 0.01 {
                    *val = 0.0;
                }
            }
        }
    }
}
