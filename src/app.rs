use crate::audio_player::AudioPlayer;
use crate::audio_player::effects::EffectType;
use crossterm::event::KeyCode;
use rodio::OutputStreamHandle;
use std::{io, time::Instant};

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
                self.player.effect_manager.change_pitch(false);
            }
            KeyCode::Char('k') => {
                self.player.effect_manager.change_pitch(true);
            }
            KeyCode::Char('v') => {
                self.player.effect_manager.change_volume(false);
            }
            KeyCode::Char('b') => {
                self.player.effect_manager.change_volume(true);
            }
            KeyCode::Char('f') => {
                self.player.effect_manager.change_lowpass(false);
            }
            KeyCode::Char('g') => {
                self.player.effect_manager.change_lowpass(true);
            }
            KeyCode::Char('e') => {
                self.player.effect_manager.toggle_reverb();
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
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
