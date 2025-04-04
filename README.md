# AudioRust - Terminal-based Audio Player with VIM motions

![AudioRust Screenshot](screenshot.png)

AudioRust is a terminal-based audio player built in Rust with a Text User Interface (TUI) that offers real-time sound visualization and audio effects. It features a modal interface inspired by Vim/Neovim, making it both keyboard-friendly and highly customizable.

## Features

- **Play and loop audio** files with intuitive controls
- **Real-time audio visualization** with waveform display
- **Multiple audio effects**:
  - Volume control
  - Playback speed/pitch adjustment
  - Low-pass filter
  - Reverb effect
- **Modal interface** with leader key system (Vim/Neovim style)
- **Visual-only mode** that works even without audio devices (perfect for WSL)
- **Terminal UI** with customizable colors and feedback

## Installation

### Prerequisites
- Rust and Cargo (https://rustup.rs/)
- Audio libraries:
  - Linux: ALSA development libraries
    ```
    sudo apt install libasound2-dev
    ```
  - macOS: No additional requirements
  - Windows: No additional requirements

### Building from source

```bash
# Clone the repository
git clone https://github.com/yourusername/audirust.git
cd audirust

# Build and run the application
cargo run --release

# Or just build it
cargo build --release
```

The compiled binary will be at `target/release/audirust`.

## Usage

Place an audio file named `example.wav` in the same directory as the executable, or edit the source to point to your audio files.

### Keyboard Controls

#### Normal Mode
- `p` - Play sound once
- `r` - Play sound in loop
- `e` - Toggle reverb effect
- `Space` - Open command menu
- `q` - Quit application

#### Command Menu (Press `Space` to activate)
- `v` - Enter Volume Mode
- `c` - Enter Pitch Mode
- `g` - Enter Filter Mode
- `Esc` - Close menu

#### Volume Mode
- `j` / Down Arrow - Decrease volume
- `k` / Up Arrow - Increase volume
- `Esc` - Return to normal mode

#### Pitch Mode
- `j` / Down Arrow - Decrease playback speed
- `k` / Up Arrow - Increase playback speed
- `Esc` - Return to normal mode

#### Filter Mode
- `j` / Down Arrow - Lower the filter cutoff frequency
- `k` / Up Arrow - Raise the filter cutoff frequency
- `Esc` - Return to normal mode

## Technical Details

AudioRust is built with:

- [Rust](https://www.rust-lang.org/) - The programming language
- [ratatui](https://github.com/ratatui-org/ratatui) - For the terminal user interface
- [rodio](https://github.com/RustAudio/rodio) - For audio playback and effects
- [crossterm](https://github.com/crossterm-rs/crossterm) - For terminal manipulation

The architecture follows a modular design:
- **Main app**: Controls application flow and event handling
- **Audio player**: Manages audio playback and effects
- **UI module**: Renders the terminal interface
- **Effect management**: Handles all audio effects

## Compatible Platforms

- Linux
- macOS
- Windows
- WSL (Windows Subsystem for Linux) - Visual mode only

## Features in Development

- [ ] File browser for selecting audio files
- [ ] Playlist support
- [ ] More audio effects (e.g., equalizer)
- [ ] Configuration through config files
- [ ] Custom keybindings

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgements

- Thanks to the Rust Audio and TUI communities for their excellent libraries
- Inspired by terminal music players like cmus and ncmpcpp
