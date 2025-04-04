use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};

pub fn draw(f: &mut Frame, app: &App) {
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
        .alignment(ratatui::prelude::Alignment::Center);
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
        .alignment(ratatui::prelude::Alignment::Center);
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

    let sparkline = Sparkline::default()
        .block(wave_block)
        .data(&waveform_data)
        .style(if app.player.is_playing() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    f.render_widget(sparkline, chunks[5]);
}
