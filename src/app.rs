use crate::audio_player::AudioPlayer;
use crossterm::event::KeyCode;
use rodio::OutputStreamHandle;
use std::io;

// Define possible app modes for UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Volume,
    Pitch,
    Filter,
}

// App state
pub struct App {
    pub player: AudioPlayer,
    pub should_quit: bool,
    pub mode: AppMode,   // Current app mode
    pub show_help: bool, // Whether to show help popup
}

impl App {
    pub fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        Self {
            player: AudioPlayer::new(stream_handle),
            should_quit: false,
            mode: AppMode::Normal,
            show_help: false,
        }
    }

    pub fn handle_key_events(&mut self, key_code: KeyCode) -> io::Result<()> {
        // Handle global keys first
        match key_code {
            KeyCode::Char('q') => {
                if self.mode == AppMode::Normal {
                    self.should_quit = true;
                } else {
                    // In other modes, q returns to normal mode
                    self.mode = AppMode::Normal;
                }
                return Ok(());
            }
            KeyCode::Esc => {
                // Escape always returns to normal mode
                self.mode = AppMode::Normal;
                self.show_help = false;
                return Ok(());
            }
            _ => {}
        }

        // Handle mode-specific keys
        match self.mode {
            AppMode::Normal => self.handle_normal_mode(key_code)?,
            AppMode::Volume => self.handle_volume_mode(key_code),
            AppMode::Pitch => self.handle_pitch_mode(key_code),
            AppMode::Filter => self.handle_filter_mode(key_code),
        }

        Ok(())
    }

    fn handle_normal_mode(&mut self, key_code: KeyCode) -> io::Result<()> {
        match key_code {
            KeyCode::Char(' ') => {
                // Leader key - toggle help menu
                self.show_help = !self.show_help;
            }
            KeyCode::Char('p') => {
                self.player.play_sound("example.wav", false)?;
            }
            KeyCode::Char('r') => {
                self.player.play_sound("example.wav", true)?;
            }
            KeyCode::Char('v') => {
                if self.show_help {
                    self.mode = AppMode::Volume;
                    self.show_help = false;
                }
            }
            KeyCode::Char('c') => {
                if self.show_help {
                    self.mode = AppMode::Pitch;
                    self.show_help = false;
                }
            }
            KeyCode::Char('g') => {
                if self.show_help {
                    self.mode = AppMode::Filter;
                    self.show_help = false;
                }
            }
            KeyCode::Char('e') => {
                self.player.effect_manager.toggle_reverb();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_volume_mode(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('j') => {
                self.player.effect_manager.change_volume(false);
            }
            KeyCode::Char('k') => {
                self.player.effect_manager.change_volume(true);
            }
            KeyCode::Down => {
                self.player.effect_manager.change_volume(false);
            }
            KeyCode::Up => {
                self.player.effect_manager.change_volume(true);
            }
            _ => {}
        }
    }

    fn handle_pitch_mode(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('j') => {
                self.player.effect_manager.change_pitch(false);
            }
            KeyCode::Char('k') => {
                self.player.effect_manager.change_pitch(true);
            }
            KeyCode::Down => {
                self.player.effect_manager.change_pitch(false);
            }
            KeyCode::Up => {
                self.player.effect_manager.change_pitch(true);
            }
            _ => {}
        }
    }

    fn handle_filter_mode(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('j') => {
                self.player.effect_manager.change_lowpass(false);
            }
            KeyCode::Char('k') => {
                self.player.effect_manager.change_lowpass(true);
            }
            KeyCode::Down => {
                self.player.effect_manager.change_lowpass(false);
            }
            KeyCode::Up => {
                self.player.effect_manager.change_lowpass(true);
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.player.cleanup_finished();
        self.player.update_looping_sounds();
        self.player.update();
    }
}
