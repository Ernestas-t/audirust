use crate::audio_player::AudioPlayer;
use crossterm::event::KeyCode;
use rodio::OutputStreamHandle;
use std::{
    io,
    time::{Duration, Instant},
};

// App state
pub struct App {
    pub player: AudioPlayer,
    pub should_quit: bool,
    pub last_update: Instant,
}

impl App {
    pub fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        Self {
            player: AudioPlayer::new(stream_handle),
            should_quit: false,
            last_update: Instant::now(),
        }
    }

    pub fn handle_key_events(&mut self, key_code: KeyCode) -> io::Result<()> {
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

    pub fn update(&mut self) {
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
