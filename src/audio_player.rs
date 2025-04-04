use rodio::{Decoder, OutputStreamHandle, Sink, Source, dynamic_mixer::mixer};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufReader},
    sync::Arc,
    time::{Duration, Instant},
};

// Represents the current state of the audio player
pub struct AudioPlayer {
    pub stream_handle: Option<OutputStreamHandle>,
    pub active_sinks: Vec<(Arc<Sink>, bool)>,
    pub playback_speed: f32,
    pub volume: f32,
    pub lowpass_cutoff: u32,
    pub reverb_enabled: bool,
    pub reverb_delay: f32,
    pub messages: Vec<String>,
    pub last_played: Option<Instant>,
    pub waveform_values: Vec<f32>, // Values for sound wave visualization
    pub audio_samples: VecDeque<f32>, // Use a fixed-size buffer for recent samples
    pub visual_only_mode: bool,
}

impl AudioPlayer {
    // Initialize a new audio player
    pub fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        // Initialize with some random waveform data
        let waveform = (0..100)
            .map(|_| {
                // Start with no wave
                0.0
            })
            .collect();

        // Check if we're in visual-only mode before moving stream_handle
        let visual_only_mode = stream_handle.is_none();

        AudioPlayer {
            stream_handle,
            active_sinks: Vec::new(),
            playback_speed: 1.0,
            volume: 1.0,
            lowpass_cutoff: 20000,
            reverb_enabled: false,
            reverb_delay: 0.06,
            messages: Vec::new(),
            last_played: None,
            waveform_values: waveform,
            audio_samples: VecDeque::new(), // Initialize empty
            visual_only_mode,
        }
    }

    // Add a message to the log
    pub fn add_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
        // Keep only the last 5 messages
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }

    // Play a sound file (looping or non-looping)
    pub fn play_sound(&mut self, file_path: &str, is_looping: bool) -> io::Result<()> {
        // Check if we're in visual-only mode
        if self.visual_only_mode {
            // Simulate playing sound in visual-only mode
            self.last_played = Some(Instant::now());
            return Ok(());
        }

        // Create a new sink for the sound
        if let Some(stream_handle) = &self.stream_handle {
            if let Ok(sink) = Sink::try_new(stream_handle) {
                let sink = Arc::new(sink);

                // Create a mixer for combining multiple audio sources
                // 2 channels (stereo), 44100 Hz sample rate
                let (mixer_controller, mixer) = mixer(2, 44100);

                // Process main audio and add to mixer
                if let Ok(file) = File::open(file_path) {
                    let file_buf = BufReader::new(file);
                    if let Ok(source) = Decoder::new(file_buf) {
                        // Apply base effects to main sound
                        let main_source = source.speed(self.playback_speed).amplify(self.volume);

                        // Add to mixer
                        mixer_controller.add(main_source);

                        // Add reverb effect if enabled
                        if self.reverb_enabled {
                            // Open a second instance of the file for reverb
                            if let Ok(reverb_file) = File::open(file_path) {
                                let reverb_buf = BufReader::new(reverb_file);
                                if let Ok(reverb_source) = Decoder::new(reverb_buf) {
                                    // Create reverb effect - delayed and quieter
                                    let reverb = reverb_source
                                        .speed(self.playback_speed)
                                        .amplify(self.volume * 0.4)
                                        .delay(Duration::from_secs_f32(self.reverb_delay));

                                    // Add reverb to mixer
                                    mixer_controller.add(reverb);
                                }
                            }
                        }

                        // Apply low-pass filter to the entire mix if needed
                        if self.lowpass_cutoff < 20000 {
                            // Convert mixer's output to f32 samples
                            let filtered_mix =
                                mixer.convert_samples().low_pass(self.lowpass_cutoff);
                            sink.append(filtered_mix);
                        } else {
                            // No filter needed
                            sink.append(mixer);
                        }

                        // Store the sink and mixer controller to keep them alive
                        self.active_sinks.push((Arc::clone(&sink), is_looping));

                        // Mark the time we last played a sound
                        self.last_played = Some(Instant::now());
                    } else {
                        self.add_message("Error decoding audio file");
                    }
                } else {
                    self.add_message(&format!(
                        "Error opening file: Make sure {} exists!",
                        file_path
                    ));
                }
            }
        }

        Ok(())
    }

    // Change the playback speed/pitch
    pub fn change_pitch(&mut self, increase: bool) {
        if increase {
            self.playback_speed = (self.playback_speed + 0.1).min(3.0);
        } else {
            self.playback_speed = (self.playback_speed - 0.1).max(0.1);
        }
    }

    // Change volume
    pub fn change_volume(&mut self, increase: bool) {
        if increase {
            self.volume = (self.volume + 0.1).min(2.0);
        } else {
            self.volume = (self.volume - 0.1).max(0.0);
        }
    }

    // Change low-pass filter
    pub fn change_lowpass(&mut self, increase: bool) {
        if increase {
            self.lowpass_cutoff = (self.lowpass_cutoff + 500).min(20000);
        } else {
            self.lowpass_cutoff = (self.lowpass_cutoff - 500).max(500);
        }
    }

    // Toggle reverb effect
    pub fn toggle_reverb(&mut self) {
        self.reverb_enabled = !self.reverb_enabled;
    }

    // Update the looping sounds if needed
    pub fn update_looping_sounds(&self) {
        // In visual-only mode, we don't need to update looping sounds
        if self.visual_only_mode {
            return;
        }

        // Manually handle looping by checking if a looping sink is empty
        for (sink, is_looping) in &self.active_sinks {
            if *is_looping && sink.empty() {
                // Create new mixer for looping sound
                let (mixer_controller, mixer) = mixer(2, 44100);

                // Open and decode the sound file
                if let Ok(file) = File::open("example.wav") {
                    let file_buf = BufReader::new(file);
                    if let Ok(source) = Decoder::new(file_buf) {
                        // Apply effects to main sound
                        let main_source = source.speed(self.playback_speed).amplify(self.volume);

                        // Add to mixer
                        mixer_controller.add(main_source);

                        // Add reverb if enabled
                        if self.reverb_enabled {
                            if let Ok(reverb_file) = File::open("example.wav") {
                                let reverb_buf = BufReader::new(reverb_file);
                                if let Ok(reverb_source) = Decoder::new(reverb_buf) {
                                    let reverb = reverb_source
                                        .speed(self.playback_speed)
                                        .amplify(self.volume * 0.4)
                                        .delay(Duration::from_secs_f32(self.reverb_delay));

                                    mixer_controller.add(reverb);
                                }
                            }
                        }

                        // Apply final processing and add to sink
                        if self.lowpass_cutoff < 20000 {
                            let filtered_mix =
                                mixer.convert_samples().low_pass(self.lowpass_cutoff);
                            sink.append(filtered_mix);
                        } else {
                            sink.append(mixer);
                        }
                    }
                }
            }
        }
    }

    // Clean up finished sounds
    pub fn cleanup_finished(&mut self) {
        // Only remove non-looping sinks that are empty
        self.active_sinks
            .retain(|(sink, is_looping)| *is_looping || !sink.empty());
    }

    // Update waveform visualization
    pub fn update_waveform(&mut self) {
        // Reset waveform if no active sounds and last played was over 5 seconds ago
        if self.active_sinks.is_empty()
            && self
                .last_played
                .map_or(true, |t| t.elapsed() > Duration::from_secs(5))
        {
            for val in &mut self.waveform_values {
                *val = *val * 0.9; // Fade out
                if *val < 0.01 {
                    *val = 0.0;
                }
            }
            return;
        }

        let is_active = !self.active_sinks.is_empty();

        // Use actual audio samples if available
        if !self.audio_samples.is_empty() {
            // Get the lengths before iteration to avoid borrowing issues
            let waveform_len = self.waveform_values.len();
            let samples_len = self.audio_samples.len();

            // Map the audio samples to the waveform values
            for (i, val) in self.waveform_values.iter_mut().enumerate() {
                if is_active {
                    // Calculate an appropriate index into the audio_samples buffer
                    let sample_index = (i * samples_len / waveform_len).min(samples_len - 1);

                    // Get the sample value and apply volume
                    let sample = self.audio_samples[sample_index].abs() * self.volume;

                    // Apply "visual" filter similar to lowpass
                    let filter_factor = if self.lowpass_cutoff < 20000 {
                        self.lowpass_cutoff as f32 / 20000.0
                    } else {
                        1.0
                    };

                    // Scale the sample based on current effects
                    *val = (sample * filter_factor).min(1.0);

                    // Add visual reverb effect if enabled
                    if self.reverb_enabled {
                        // Get a slightly offset sample for reverb effect
                        let reverb_index =
                            (sample_index + (self.reverb_delay * 44100.0) as usize) % samples_len;
                        let reverb_sample =
                            self.audio_samples[reverb_index].abs() * 0.3 * self.volume;
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
        } else {
            // Fall back to simulated waveform if no audio samples available
            // or if we're in visual-only mode
            let time = Instant::now().elapsed().as_secs_f64();

            for (i, val) in self.waveform_values.iter_mut().enumerate() {
                if is_active || self.visual_only_mode {
                    let x = i as f64 / 10.0;
                    let base_wave =
                        (time * 5.0 * self.playback_speed as f64 + x).sin() * self.volume as f64;

                    // Apply "visual" filter similar to lowpass
                    let filter_factor = if self.lowpass_cutoff < 20000 {
                        self.lowpass_cutoff as f64 / 20000.0
                    } else {
                        1.0
                    };

                    // Add harmonic for visuals
                    let harmonic =
                        (time * 10.0 * self.playback_speed as f64 + x).sin() * 0.3 * filter_factor;

                    // Combine waves
                    *val = (base_wave + harmonic).abs() as f32 * self.volume;

                    // Add visual reverb effect
                    if self.reverb_enabled {
                        let reverb_wave = (time * 5.0 * self.playback_speed as f64 + x - 0.5).sin()
                            * 0.3
                            * self.volume as f64;
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

    // Is any sound currently playing?
    pub fn is_playing(&self) -> bool {
        self.visual_only_mode || !self.active_sinks.is_empty()
    }
}
