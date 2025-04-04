pub mod effects;
pub mod visualization;

use effects::{EffectManager, EffectType};
use rodio::{Decoder, OutputStreamHandle, Sink, Source, dynamic_mixer::mixer};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufReader},
    sync::Arc,
    time::{Duration, Instant},
};
use visualization::WaveformVisualizer;

pub struct AudioPlayer {
    pub stream_handle: Option<OutputStreamHandle>,
    pub active_sinks: Vec<(Arc<Sink>, bool)>,
    pub messages: Vec<String>,
    pub last_played: Option<Instant>,
    pub visual_only_mode: bool,

    // Effect management
    pub effect_manager: EffectManager,

    // Visualization
    pub visualizer: WaveformVisualizer,
}

impl AudioPlayer {
    pub fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        let visual_only_mode = stream_handle.is_none();

        AudioPlayer {
            stream_handle,
            active_sinks: Vec::new(),
            messages: Vec::new(),
            last_played: None,
            visual_only_mode,
            effect_manager: EffectManager::new(),
            visualizer: WaveformVisualizer::new(100), // 100 points for waveform
        }
    }

    pub fn add_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }

    pub fn play_sound(&mut self, file_path: &str, is_looping: bool) -> io::Result<()> {
        // In visual-only mode, just update timestamps without actual playback
        if self.visual_only_mode {
            self.last_played = Some(Instant::now());
            return Ok(());
        }

        if let Some(stream_handle) = &self.stream_handle {
            if let Ok(sink) = Sink::try_new(stream_handle) {
                let sink = Arc::new(sink);
                let (mixer_controller, mixer) = mixer(2, 44100);

                if let Ok(file) = File::open(file_path) {
                    let file_buf = BufReader::new(file);
                    if let Ok(source) = Decoder::new(file_buf) {
                        // Apply base effects
                        let mut main_source = source
                            .speed(self.effect_manager.get_playback_speed())
                            .amplify(self.effect_manager.get_volume());

                        mixer_controller.add(main_source);

                        // Add reverb if enabled
                        if self.effect_manager.is_reverb_enabled() {
                            if let Ok(reverb_file) = File::open(file_path) {
                                let reverb_buf = BufReader::new(reverb_file);
                                if let Ok(reverb_source) = Decoder::new(reverb_buf) {
                                    let reverb = reverb_source
                                        .speed(self.effect_manager.get_playback_speed())
                                        .amplify(self.effect_manager.get_volume() * 0.4)
                                        .delay(Duration::from_secs_f32(
                                            self.effect_manager.get_reverb_delay(),
                                        ));

                                    mixer_controller.add(reverb);
                                }
                            }
                        }

                        // Apply lowpass filter
                        let lowpass_cutoff = self.effect_manager.get_lowpass_cutoff();
                        if lowpass_cutoff < 20000 {
                            let filtered_mix = mixer.convert_samples().low_pass(lowpass_cutoff);
                            sink.append(filtered_mix);
                        } else {
                            sink.append(mixer);
                        }

                        self.active_sinks.push((Arc::clone(&sink), is_looping));
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

    pub fn update_looping_sounds(&self) {
        if self.visual_only_mode {
            return;
        }

        // Implementation remains similar but uses effect_manager
        // ...
    }

    pub fn cleanup_finished(&mut self) {
        self.active_sinks
            .retain(|(sink, is_looping)| *is_looping || !sink.empty());
    }

    pub fn is_playing(&self) -> bool {
        self.visual_only_mode || !self.active_sinks.is_empty()
    }

    pub fn update(&mut self) {
        self.visualizer.update(
            &self.active_sinks,
            self.last_played,
            self.visual_only_mode,
            &self.effect_manager,
        );
    }
}
