use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use rodio::dynamic_mixer::mixer;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufReader, stdout},
    sync::Arc,
    time::{Duration, Instant},
};

// Represents the current state of the audio player
struct AudioPlayer {
    stream_handle: Option<OutputStreamHandle>,
    active_sinks: Vec<(Arc<Sink>, bool)>,
    playback_speed: f32,
    volume: f32,
    lowpass_cutoff: u32,
    reverb_enabled: bool,
    reverb_delay: f32,
    messages: Vec<String>,
    last_played: Option<Instant>,
    waveform_values: Vec<f32>,    // Values for sound wave visualization
    audio_samples: VecDeque<f32>, // Use a fixed-size buffer for recent samples
    visual_only_mode: bool,
}

impl AudioPlayer {
    // Initialize a new audio player
    fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
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
    fn add_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
        // Keep only the last 5 messages
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }

    // Play a sound file (looping or non-looping)
    fn play_sound(&mut self, file_path: &str, is_looping: bool) -> io::Result<()> {
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
    fn change_pitch(&mut self, increase: bool) {
        if increase {
            self.playback_speed = (self.playback_speed + 0.1).min(3.0);
        } else {
            self.playback_speed = (self.playback_speed - 0.1).max(0.1);
        }
    }

    // Change volume
    fn change_volume(&mut self, increase: bool) {
        if increase {
            self.volume = (self.volume + 0.1).min(2.0);
        } else {
            self.volume = (self.volume - 0.1).max(0.0);
        }
    }

    // Change low-pass filter
    fn change_lowpass(&mut self, increase: bool) {
        if increase {
            self.lowpass_cutoff = (self.lowpass_cutoff + 500).min(20000);
        } else {
            self.lowpass_cutoff = (self.lowpass_cutoff - 500).max(500);
        }
    }

    // Toggle reverb effect
    fn toggle_reverb(&mut self) {
        self.reverb_enabled = !self.reverb_enabled;
    }

    // Update the looping sounds if needed
    fn update_looping_sounds(&self) {
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
    fn cleanup_finished(&mut self) {
        // Only remove non-looping sinks that are empty
        self.active_sinks
            .retain(|(sink, is_looping)| *is_looping || !sink.empty());
    }

    // Update waveform visualization
    fn update_waveform(&mut self) {
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
    fn is_playing(&self) -> bool {
        self.visual_only_mode || !self.active_sinks.is_empty()
    }
}

// App state
struct App {
    player: AudioPlayer,
    should_quit: bool,
    last_update: Instant,
}

impl App {
    fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        Self {
            player: AudioPlayer::new(stream_handle),
            should_quit: false,
            last_update: Instant::now(),
        }
    }

    fn handle_key_events(&mut self, key_code: KeyCode) -> io::Result<()> {
        match key_code {
            KeyCode::Char('p') => {
                self.player.play_sound("example.wav", false)?;
            }
            KeyCode::Char('r') => {
                self.player.play_sound("example.wav", true)?;
            }
            KeyCode::Char('j') => {
                self.player.change_pitch(false);
            }
            KeyCode::Char('k') => {
                self.player.change_pitch(true);
            }
            KeyCode::Char('v') => {
                self.player.change_volume(false);
            }
            KeyCode::Char('b') => {
                self.player.change_volume(true);
            }
            KeyCode::Char('f') => {
                self.player.change_lowpass(false);
            }
            KeyCode::Char('g') => {
                self.player.change_lowpass(true);
            }
            KeyCode::Char('e') => {
                self.player.toggle_reverb();
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self) {
        // Update app state
        self.player.update_looping_sounds();
        self.player.cleanup_finished();

        // Update waveform every 50ms
        if self.last_update.elapsed() > Duration::from_millis(50) {
            self.player.update_waveform();
            self.last_update = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(3), // Volume
                Constraint::Length(3), // Speed
                Constraint::Length(3), // Effects area
                Constraint::Length(3), // Controls
                Constraint::Min(0),    // Waveform visualization (now larger at the bottom)
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title with playback status
    let status = if app.player.is_playing() {
        if app.player.visual_only_mode {
            " [VISUAL MODE]"
        } else {
            " [PLAYING]"
        }
    } else {
        ""
    };

    let title = Paragraph::new(format!("Audio Player{}", status))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("TUI Audio Player"),
        )
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Volume gauge
    let volume_percent = (app.player.volume / 2.0 * 100.0) as u16;
    let volume_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Volume"))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(volume_percent)
        .label(format!("{:.1}x", app.player.volume));
    f.render_widget(volume_gauge, chunks[1]);

    // Speed gauge
    let speed_percent = (app.player.playback_speed / 3.0 * 100.0) as u16;
    let speed_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Playback Speed"),
        )
        .gauge_style(Style::default().fg(Color::Green))
        .percent(speed_percent)
        .label(format!("{:.1}x", app.player.playback_speed));
    f.render_widget(speed_gauge, chunks[2]);

    // Effects area - split horizontally
    let effects_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[3]);

    // Low-pass filter
    let filter_text = if app.player.lowpass_cutoff >= 20000 {
        "OFF".to_string()
    } else {
        format!("{}Hz", app.player.lowpass_cutoff)
    };

    let filter_percent = if app.player.lowpass_cutoff >= 20000 {
        100
    } else {
        (app.player.lowpass_cutoff as f32 / 20000.0 * 100.0) as u16
    };

    let lowpass_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Low-Pass Filter"),
        )
        .gauge_style(Style::default().fg(Color::Blue))
        .percent(filter_percent)
        .label(filter_text);
    f.render_widget(lowpass_gauge, effects_chunks[0]);

    // Simplified reverb indicator
    let reverb_title = if app.player.reverb_enabled {
        "Reverb: ON"
    } else {
        "Reverb: OFF"
    };

    let reverb_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(reverb_title))
        .gauge_style(if app.player.reverb_enabled {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .percent(if app.player.reverb_enabled { 100 } else { 0 })
        .label(if app.player.reverb_enabled {
            "Enabled"
        } else {
            "Disabled"
        });

    f.render_widget(reverb_gauge, effects_chunks[1]);

    // Controls with status
    let playing_info = if app.player.active_sinks.is_empty() {
        String::new()
    } else {
        let loop_count = app
            .player
            .active_sinks
            .iter()
            .filter(|(_, is_loop)| *is_loop)
            .count();
        format!(
            " | Playing: {} (Loops: {})",
            app.player.active_sinks.len(),
            loop_count
        )
    };

    let controls_text = format!(
        "p: Play  r: Loop  j/k: Pitch⬇/⬆  v/b: Vol⬇/⬆  f/g: Filter⬇/⬆  e: Reverb  q: Quit{}",
        playing_info
    );

    let controls = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(Alignment::Center);
    f.render_widget(controls, chunks[4]);

    // Waveform visualization (now with more space at the bottom)
    let wave_block = Block::default()
        .borders(Borders::ALL)
        .title("Sound Visualization");

    // Create a sparkline for audio waveform
    let waveform_data: Vec<u64> = app
        .player
        .waveform_values
        .iter()
        .map(|&v| (v * 100.0) as u64)
        .collect();

    let sparkline = ratatui::widgets::Sparkline::default()
        .block(wave_block)
        .data(&waveform_data)
        .style(if app.player.is_playing() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    f.render_widget(sparkline, chunks[5]);
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up audio - but continue even if it fails
    let stream_handle = match OutputStream::try_default() {
        Ok((_stream, handle)) => {
            // Keep stream alive by storing it in a tuple
            Some(handle)
        }
        Err(e) => {
            // Log the error and continue in visual-only mode
            eprintln!(
                "Audio device not available: {}. Running in visual-only mode.",
                e
            );
            None
        }
    };

    // Create the app state
    let mut app = App::new(stream_handle);

    // Add a message if we're in visual-only mode
    if app.player.visual_only_mode {
        app.player
            .add_message("Running in visual-only mode (no audio device)");
    }

    loop {
        // Draw the UI
        terminal.draw(|f| ui(f, &app))?;

        // Handle key events
        if event::poll(Duration::from_millis(16))? {
            // ~60fps
            if let Event::Key(key) = event::read()? {
                app.handle_key_events(key.code)?;
            }
        }

        // Update app state
        app.update();

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
