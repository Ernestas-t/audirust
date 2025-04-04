use crate::audio_player::AudioPlayer;
use crate::file_manager::FileManager;
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
    FileBrowser, // New mode for file browsing
}

// App state
pub struct App {
    pub player: AudioPlayer,
    pub file_manager: FileManager,
    pub should_quit: bool,
    pub mode: AppMode,
    pub show_help: bool,
    pub current_audio_file: Option<String>, // Add this to track the current audio file name
}

impl App {
    pub fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        Self {
            player: AudioPlayer::new(stream_handle),
            file_manager: FileManager::new(),
            should_quit: false,
            mode: AppMode::Normal,
            show_help: false,
            current_audio_file: None,
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
            AppMode::FileBrowser => self.handle_file_browser_mode(key_code)?,
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
                // Instead of hardcoded file, use selected file
                if let Some(file_path) = self.file_manager.get_selected_file() {
                    if !file_path.is_dir() && self.file_manager.is_audio_file(&file_path) {
                        // Set current audio file name
                        self.current_audio_file = file_path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string());

                        self.player
                            .play_sound(file_path.to_str().unwrap_or("example.wav"), false)?;
                    }
                } else {
                    // Fallback to example.wav if no file selected
                    self.player.play_sound("example.wav", false)?;
                }
            }
            KeyCode::Char('r') => {
                // Loop selected file
                if let Some(file_path) = self.file_manager.get_selected_file() {
                    if !file_path.is_dir() && self.file_manager.is_audio_file(&file_path) {
                        // Set current audio file name
                        self.current_audio_file = file_path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string());

                        self.player
                            .play_sound(file_path.to_str().unwrap_or("example.wav"), true)?;
                    }
                } else {
                    // Fallback to example.wav if no file selected
                    self.player.play_sound("example.wav", true)?;
                }
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
            KeyCode::Char('f') => {
                if self.show_help {
                    self.mode = AppMode::FileBrowser;
                    self.show_help = false;
                    // Refresh files when entering browser
                    self.file_manager.refresh_files();
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

    fn handle_file_browser_mode(&mut self, key_code: KeyCode) -> io::Result<()> {
        match key_code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.file_manager.select_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.file_manager.select_prev();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.file_manager.go_to_parent_dir();
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if let Some(selected) = self.file_manager.get_selected_file() {
                    if selected.is_dir() {
                        self.file_manager.change_directory(selected);
                    } else if self.file_manager.is_audio_file(&selected) {
                        // Set current audio file name
                        self.current_audio_file = selected
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string());

                        // Play the selected file
                        self.player
                            .play_sound(selected.to_str().unwrap_or("example.wav"), false)?;
                        // Return to normal mode
                        self.mode = AppMode::Normal;
                    }
                }
            }
            KeyCode::Char('p') => {
                // Instead of hardcoded file, use selected file
                if let Some(file_path) = self.file_manager.get_selected_file() {
                    if !file_path.is_dir() && self.file_manager.is_audio_file(&file_path) {
                        // Set current audio file name
                        self.current_audio_file = file_path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string());

                        self.player
                            .play_sound(file_path.to_str().unwrap_or("example.wav"), false)?;
                    } else {
                        // Fallback to example.wav if selected file is not playable
                        self.current_audio_file = Some("example.wav".to_string());
                        self.player.play_sound("example.wav", false)?;
                    }
                } else {
                    // Fallback to example.wav if no file selected
                    self.current_audio_file = Some("example.wav".to_string());
                    self.player.play_sound("example.wav", false)?;
                }
            }
            KeyCode::Char('r') => {
                // Loop selected file
                if let Some(file_path) = self.file_manager.get_selected_file() {
                    if !file_path.is_dir() && self.file_manager.is_audio_file(&file_path) {
                        // Set current audio file name
                        self.current_audio_file = file_path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string());

                        self.player
                            .play_sound(file_path.to_str().unwrap_or("example.wav"), true)?;
                    } else {
                        // Fallback to example.wav if selected file is not playable
                        self.current_audio_file = Some("example.wav".to_string());
                        self.player.play_sound("example.wav", true)?;
                    }
                } else {
                    // Fallback to example.wav if no file selected
                    self.current_audio_file = Some("example.wav".to_string());
                    self.player.play_sound("example.wav", true)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn update(&mut self) {
        self.player.cleanup_finished();
        self.player.update_looping_sounds();
        self.player.update();
    }
}
