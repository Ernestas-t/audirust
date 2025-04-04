mod app;
mod audio_player;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use rodio::OutputStream;
use std::{
    io::{self, stdout},
    time::Duration,
};

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
        terminal.draw(|f| ui::draw(f, &app))?;

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
