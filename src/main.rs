use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use rodio::dynamic_mixer::{DynamicMixerController, mixer};
use std::fs::File;
use std::io::{self, BufReader};
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType};
use crossterm::execute;
use std::io::stdout;
use std::sync::Arc;

// Represents the current state of the audio player
struct AudioPlayer {
    stream_handle: OutputStreamHandle,
    active_sinks: Vec<(Arc<Sink>, bool)>,
    playback_speed: f32,
    volume: f32,
    lowpass_cutoff: u32,
    reverb_enabled: bool,
    reverb_delay: f32,
    current_line: u16,
}

impl AudioPlayer {
    // Initialize a new audio player
    fn new(stream_handle: OutputStreamHandle) -> Self {
        AudioPlayer {
            stream_handle,
            active_sinks: Vec::new(),
            playback_speed: 1.0,
            volume: 1.0,
            lowpass_cutoff: 20000,
            reverb_enabled: false,
            reverb_delay: 0.06,
            current_line: 10,
        }
    }
    
    // Play a sound file (looping or non-looping)
    fn play_sound(&mut self, file_path: &str, is_looping: bool) -> io::Result<()> {
        // Move to next line for new output
        execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
        if is_looping {
            println!("Playing looping sound...");
        } else {
            println!("Playing sound once...");
        }
        self.current_line += 1;
        
        // Create a new sink for the sound
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            let sink = Arc::new(sink);
            
            // Create a mixer for combining multiple audio sources
            // 2 channels (stereo), 44100 Hz sample rate
            let (mixer_controller, mixer) = mixer(2, 44100);
            
            // Process main audio and add to mixer
            if let Ok(file) = File::open(file_path) {
                let file_buf = BufReader::new(file);
                if let Ok(source) = Decoder::new(file_buf) {
                    // Apply base effects to main sound
                    let main_source = source
                        .speed(self.playback_speed)
                        .amplify(self.volume);
                    
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
                        let filtered_mix = mixer.convert_samples().low_pass(self.lowpass_cutoff);
                        sink.append(filtered_mix);
                    } else {
                        // No filter needed
                        sink.append(mixer);
                    }
                    
                    // Store the sink and mixer controller to keep them alive
                    self.active_sinks.push((Arc::clone(&sink), is_looping));
                    
                    execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
                    println!("Sound is playing! Effects: Pitch={:.1}x, Vol={:.1}x, LP={}Hz, Reverb={}",
                        self.playback_speed, 
                        self.volume, 
                        if self.lowpass_cutoff < 20000 { self.lowpass_cutoff.to_string() } else { "OFF".to_string() },
                        if self.reverb_enabled { "ON" } else { "OFF" }
                    );
                    self.current_line += 1;
                } else {
                    execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
                    println!("Error decoding audio file");
                    self.current_line += 1;
                }
            } else {
                execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
                println!("Error opening file: Make sure {} exists!", file_path);
                self.current_line += 1;
            }
        }
        
        Ok(())
    }
    
    // Change the playback speed/pitch
    fn change_pitch(&mut self, increase: bool) -> io::Result<()> {
        if increase {
            self.playback_speed = (self.playback_speed + 0.1).min(3.0);
            execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
            println!("Pitch increased to {:.1}x", self.playback_speed);
        } else {
            self.playback_speed = (self.playback_speed - 0.1).max(0.1);
            execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
            println!("Pitch decreased to {:.1}x", self.playback_speed);
        }
        self.current_line += 1;
        Ok(())
    }
    
    // Change volume
    fn change_volume(&mut self, increase: bool) -> io::Result<()> {
        if increase {
            self.volume = (self.volume + 0.1).min(2.0);
            execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
            println!("Volume increased to {:.1}x", self.volume);
        } else {
            self.volume = (self.volume - 0.1).max(0.0);
            execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
            println!("Volume decreased to {:.1}x", self.volume);
        }
        self.current_line += 1;
        Ok(())
    }
    
    // Change low-pass filter
    fn change_lowpass(&mut self, increase: bool) -> io::Result<()> {
        if increase {
            self.lowpass_cutoff = (self.lowpass_cutoff + 500).min(20000);
        } else {
            self.lowpass_cutoff = (self.lowpass_cutoff - 500).max(500);
        }
        
        execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
        if self.lowpass_cutoff >= 20000 {
            println!("Low-pass filter: OFF");
        } else {
            println!("Low-pass filter: {}Hz", self.lowpass_cutoff);
        }
        self.current_line += 1;
        Ok(())
    }
    
    // Toggle reverb effect
    fn toggle_reverb(&mut self) -> io::Result<()> {
        self.reverb_enabled = !self.reverb_enabled;
        execute!(stdout(), crossterm::cursor::MoveTo(0, self.current_line))?;
        println!("Reverb effect: {}", if self.reverb_enabled { "ON" } else { "OFF" });
        self.current_line += 1;
        Ok(())
    }
    
    // Update the looping sounds if needed
    fn update_looping_sounds(&self) {
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
                        let main_source = source
                            .speed(self.playback_speed)
                            .amplify(self.volume);
                        
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
                            let filtered_mix = mixer.convert_samples().low_pass(self.lowpass_cutoff);
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
        self.active_sinks.retain(|(sink, is_looping)| *is_looping || !sink.empty());
    }
}

// Display the user interface
fn display_ui() -> io::Result<()> {
    execute!(stdout(), Clear(ClearType::All))?;
    execute!(stdout(), crossterm::cursor::MoveTo(0, 0))?;
    println!("Audio Player");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 1))?;
    println!("Press 'p' to play sound");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 2))?;
    println!("Press 'r' to play looping sound");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 3))?;
    println!("Press 'j' to decrease pitch, 'k' to increase pitch");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 4))?;
    println!("Press 'v' to decrease volume, 'b' to increase volume");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 5))?;
    println!("Press 'f' to decrease low-pass filter, 'g' to increase");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 6))?;
    println!("Press 'e' to toggle reverb effect");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 7))?;
    println!("Press 'c' to clear console output");
    execute!(stdout(), crossterm::cursor::MoveTo(0, 8))?;
    println!("Press 'q' to quit");
    Ok(())
}

// Process key presses
fn handle_key_event(key_event: event::KeyEvent, player: &mut AudioPlayer) -> io::Result<bool> {
    match key_event.code {
        KeyCode::Char('p') => {
            player.play_sound("example.wav", false)?;
        },
        KeyCode::Char('r') => {
            player.play_sound("example.wav", true)?;
        },
        KeyCode::Char('j') => {
            player.change_pitch(false)?;
        },
        KeyCode::Char('k') => {
            player.change_pitch(true)?;
        },
        KeyCode::Char('v') => {
            player.change_volume(false)?;
        },
        KeyCode::Char('b') => {
            player.change_volume(true)?;
        },
        KeyCode::Char('f') => {
            player.change_lowpass(false)?;
        },
        KeyCode::Char('g') => {
            player.change_lowpass(true)?;
        },
        KeyCode::Char('e') => {
            player.toggle_reverb()?;
        },
        KeyCode::Char('c') => {
            // Clear the screen and redisplay UI
            display_ui()?;
            player.current_line = 10;
        },
        KeyCode::Char('q') => {
            execute!(stdout(), crossterm::cursor::MoveTo(0, player.current_line))?;
            println!("Quitting...");
            return Ok(true); // Signal to quit
        },
        _ => {}
    }
    Ok(false) // Continue running
}

fn main() -> io::Result<()> {
    // Enable raw mode to get key presses without waiting for Enter
    enable_raw_mode()?;
    
    // Display the UI
    display_ui()?;
    
    // Set up audio - with error handling
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(output) => output,
        Err(e) => {
            execute!(stdout(), crossterm::cursor::MoveTo(0, 10))?;
            println!("Failed to initialize audio: {}. Exiting.", e);
            disable_raw_mode()?;
            return Ok(());
        }
    };
    
    // Create the audio player
    let mut player = AudioPlayer::new(stream_handle);
    
    // Main application loop
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                // Handle key events, quit if requested
                if handle_key_event(key_event, &mut player)? {
                    break;
                }
            }
        }
        
        // Update looping sounds
        player.update_looping_sounds();
        
        // Clean up finished sounds
        player.cleanup_finished();
    }
    
    // Restore terminal settings
    disable_raw_mode()?;
    Ok(())
}
