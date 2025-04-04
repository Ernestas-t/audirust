use crate::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Sparkline},
};

pub fn draw(f: &mut Frame, app: &App) {
    // If in file browser mode, show that instead of normal UI
    if app.mode == AppMode::FileBrowser {
        render_file_browser(f, app);
        return;
    }

    // Normal UI rendering for other modes
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(3), // Current file
                Constraint::Length(3), // Volume
                Constraint::Length(3), // Speed
                Constraint::Length(3), // Effects area
                Constraint::Length(3), // Controls
                Constraint::Min(0),    // Waveform visualization
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title with playback status and current mode
    let mode_text = match app.mode {
        AppMode::Normal => "",
        AppMode::Volume => " [VOLUME MODE]",
        AppMode::Pitch => " [PITCH MODE]",
        AppMode::Filter => " [FILTER MODE]",
        AppMode::FileBrowser => " [FILE BROWSER]",
    };

    let status = if app.player.is_playing() {
        if app.player.visual_only_mode {
            " [VISUAL MODE]"
        } else {
            " [PLAYING]"
        }
    } else {
        ""
    };

    let title = Paragraph::new(format!("Audio Player{}{}", status, mode_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("TUI Audio Player"),
        )
        .style(Style::default().fg(Color::Cyan))
        .alignment(ratatui::prelude::Alignment::Center);
    f.render_widget(title, chunks[0]);

    let file_text = match &app.current_audio_file {
        Some(file_name) => format!("🎵 {}", file_name),
        None => "No file selected".to_string(),
    };

    let current_file = Paragraph::new(file_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Current Audio File"),
        )
        .style(Style::default().fg(Color::Cyan))
        .alignment(ratatui::prelude::Alignment::Center);

    f.render_widget(current_file, chunks[1]);

    // Volume gauge
    let volume_percent = (app.player.effect_manager.get_volume() / 2.0 * 100.0) as u16;
    let volume_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.mode == AppMode::Volume {
                    "Volume (j/k to adjust)"
                } else {
                    "Volume"
                }),
        )
        .gauge_style(Style::default().fg(if app.mode == AppMode::Volume {
            Color::Red
        } else {
            Color::Yellow
        }))
        .percent(volume_percent)
        .label(format!("{:.1}x", app.player.effect_manager.get_volume()));
    f.render_widget(volume_gauge, chunks[2]);

    // Speed gauge
    let speed_percent = (app.player.effect_manager.get_playback_speed() / 3.0 * 100.0) as u16;
    let speed_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.mode == AppMode::Pitch {
                    "Playback Speed (j/k to adjust)"
                } else {
                    "Playback Speed"
                }),
        )
        .gauge_style(Style::default().fg(if app.mode == AppMode::Pitch {
            Color::Red
        } else {
            Color::Green
        }))
        .percent(speed_percent)
        .label(format!(
            "{:.1}x",
            app.player.effect_manager.get_playback_speed()
        ));
    f.render_widget(speed_gauge, chunks[3]);

    // Effects area - split horizontally
    let effects_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[4]);

    // Low-pass filter
    let lowpass_cutoff = app.player.effect_manager.get_lowpass_cutoff();
    let filter_text = if lowpass_cutoff >= 20000 {
        "OFF".to_string()
    } else {
        format!("{}Hz", lowpass_cutoff)
    };

    let filter_percent = if lowpass_cutoff >= 20000 {
        100
    } else {
        (lowpass_cutoff as f32 / 20000.0 * 100.0) as u16
    };

    let lowpass_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.mode == AppMode::Filter {
                    "Low-Pass Filter (j/k to adjust)"
                } else {
                    "Low-Pass Filter"
                }),
        )
        .gauge_style(Style::default().fg(if app.mode == AppMode::Filter {
            Color::Red
        } else {
            Color::Blue
        }))
        .percent(filter_percent)
        .label(filter_text);
    f.render_widget(lowpass_gauge, effects_chunks[0]);

    // Simplified reverb indicator
    let reverb_enabled = app.player.effect_manager.is_reverb_enabled();
    let reverb_title = if reverb_enabled {
        "Reverb: ON"
    } else {
        "Reverb: OFF"
    };

    let reverb_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(reverb_title))
        .gauge_style(if reverb_enabled {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .percent(if reverb_enabled { 100 } else { 0 })
        .label(if reverb_enabled {
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

    // Update controls based on mode
    let controls_text = match app.mode {
        AppMode::Normal => format!(
            "p: Play  r: Loop  <Space>: Menu  e: Reverb  q: Quit{}",
            playing_info
        ),
        AppMode::Volume => "j/k: Adjust Volume  Esc: Exit mode".to_string(),
        AppMode::Pitch => "j/k: Adjust Pitch  Esc: Exit mode".to_string(),
        AppMode::Filter => "j/k: Adjust Filter  Esc: Exit mode".to_string(),
        AppMode::FileBrowser => {
            "j/k: Navigate  Enter: Select/Play  h: Up Dir  Esc: Exit".to_string()
        }
    };

    let controls = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(ratatui::prelude::Alignment::Center);
    f.render_widget(controls, chunks[5]);

    // Waveform visualization (now with more space at the bottom)
    let wave_block = Block::default()
        .borders(Borders::ALL)
        .title("Sound Visualization");

    // Create a sparkline for audio waveform
    let waveform_data: Vec<u64> = app
        .player
        .visualizer
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

    f.render_widget(sparkline, chunks[6]);

    fn render_help_popup(f: &mut Frame) {
        // Calculate popup size and position
        let area = f.area();
        let popup_width = 40;
        let popup_height = 12; // Increased height for file browser option
        let popup_x = (area.width - popup_width) / 2;
        let popup_y = (area.height - popup_height) / 2;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Render the popup background
        f.render_widget(Clear, popup_area);

        // Create the popup block
        let help_block = Block::default()
            .title("Command Menu")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        f.render_widget(help_block, popup_area);

        // Create the inner area for text
        let inner_area = Rect::new(popup_x + 2, popup_y + 2, popup_width - 4, popup_height - 4);

        // Help text
        let help_text = vec![
            Line::from(vec![
                Span::styled(
                    "v",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Volume mode"),
            ]),
            Line::from(vec![
                Span::styled(
                    "c",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Pitch mode"),
            ]),
            Line::from(vec![
                Span::styled(
                    "g",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Filter mode"),
            ]),
            Line::from(vec![
                Span::styled(
                    "f",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": File browser"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Close menu"),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_text);
        f.render_widget(help_paragraph, inner_area);
    }

    fn render_file_browser(f: &mut Frame, app: &App) {
        // Use the bottom section for file browser
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // File list
                Constraint::Length(3), // Instructions
            ])
            .margin(1)
            .split(f.area());

        // Create a block for file browser
        let block = Block::default()
            .title(format!(
                " File Browser: {} ",
                app.file_manager.current_dir.to_string_lossy()
            ))
            .borders(Borders::ALL);
        f.render_widget(block, f.area());

        // Header
        let header =
            Paragraph::new("Use j/k to navigate, Enter to select/play, h to go up, Esc to exit")
                .style(Style::default().add_modifier(Modifier::BOLD))
                .alignment(ratatui::prelude::Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Create list of files
        let mut items = Vec::new();
        for (i, path) in app.file_manager.entries.iter().enumerate() {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "???".to_string());

            let is_dir = path.is_dir();
            let prefix = if is_dir { "📁 " } else { "🎵 " };

            let display_name = format!("{}{}", prefix, file_name);

            if i == app.file_manager.selected_index {
                // Highlight selected item
                items.push(Line::from(vec![Span::styled(
                    format!("> {}", display_name),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
            } else {
                items.push(Line::from(display_name));
            }
        }

        if items.is_empty() {
            items.push(Line::from(vec![Span::styled(
                "No files or directories found",
                Style::default().fg(Color::Red),
            )]));
        }

        let file_list = Paragraph::new(items).block(Block::default());
        f.render_widget(file_list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::styled(
                "p",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Play file  "),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Loop file  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Exit browser"),
        ])]))
        .alignment(ratatui::prelude::Alignment::Center);
        f.render_widget(instructions, chunks[2]);
    }

    // Render help popup if needed
    if app.show_help {
        render_help_popup(f);
    }
}
