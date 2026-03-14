<div align="center">

# 🎮 Terminal Hangman

**A lightning-fast, zero-flicker terminal hangman game built with Rust and Ratatui.**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Ratatui](https://img.shields.io/badge/ratatui-%23F05032.svg?style=for-the-badge&logo=rust&logoColor=white)](https://ratatui.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

</div>

## ✨ Key Features

- 🦀 **Powered by Rust & Ratatui:** Enjoy a robust, incredibly fast, and memory-safe terminal experience.
- ⚡ **Zero-Flicker Rendering:** Smooth UI rendering thanks to Ratatui's intelligent terminal backend.
- 🌍 **5 Localized Languages:** Play in English, Spanish (Español), Portuguese (Português), German (Deutsch), and Dutch (Nederlands).
- ⏱️ **Real-Time Timer:** Feel the pressure with a live countdown timer built right into the gameplay!
- 🎨 **Modern Terminal UI:** Beautifully centered layouts, styled text, and classic ASCII art scaling elegantly with your window.
- 🔤 **Unicode Support:** Native support for accented characters (é, ñ, á, ã, etc.).
- ⌨️ **Intuitive Controls:** Simple keystroke detection (no typing delays). Press `Ctrl-C` or `Esc` to instantly exit and gracefully restore your terminal.

## 🚀 Installation

### Using Cargo (Recommended)

If you have Rust installed, you can build and install the game directly from the source:

```bash
cargo install --path .
```

### Manual Build

```bash
# Build the release version
cargo build --release

# Run the executable
./target/release/hangman
```

## 🕹️ Gameplay

Start the game simply by running:

```bash
hangman
```

1. **Select your language:** Press `1-5` to choose from the startup menu.
2. **Guess letters:** Press any alphabetic key to make a guess. The game responds instantly!
3. **Beat the clock:** You have a strict time limit per turn. Make your move before the timer hits 0.0s!
4. **Survive:** 6 wrong attempts and the hangman will be complete.

## 🛠️ Tech Stack Highlights

This project stands as a testament to modern CLI development:
- **[Rust](https://www.rust-lang.org/):** Core logic, blazing fast, and memory-safe.
- **[Ratatui](https://ratatui.rs/):** Next-generation Terminal User Interface library, replacing older TUI crates.
- **[Crossterm](https://github.com/crossterm-rs/crossterm):** Cross-platform terminal manipulation (handling raw modes, key events, and graceful exits).

## 🤝 Contributing

Pull requests are fully welcome! Feel free to open issues to discuss bugs or new feature ideas.

## 📝 License

This project is licensed under the **MIT** License.
