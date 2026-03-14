<div align="center">

# 🎮 bet

**A lightning-fast, zero-flicker terminal game hub containing Hangman, Tic-Tac-Toe, Chess, Pong, and an Ultimate Media Recommender, built with Rust and Ratatui.**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Ratatui](https://img.shields.io/badge/ratatui-%23F05032.svg?style=for-the-badge&logo=rust&logoColor=white)](https://ratatui.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

</div>

## ✨ Key Features

- 🦀 **Powered by Rust & Ratatui:** Enjoy a robust, incredibly fast, and memory-safe terminal experience.
- ⚡ **Zero-Flicker Rendering:** Smooth UI rendering thanks to Ratatui's intelligent terminal backend.
- 🌍 **5 Localized Languages:** Play in English, Spanish (Español), Portuguese (Português), German (Deutsch), and Dutch (Nederlands).
- 🎲 **Multiple Games:** Play classic Hangman, test your logic against a simple AI in Tic-Tac-Toe, or play Chess against an AI, bounce the ball in Pong, or get a Random Movie, Series, Manga, Book, Anime, Cartoon, Video Game, or Music Recommendation!
- 🎨 **Modern Terminal UI:** Beautifully centered layouts, styled text, and classic ASCII art scaling elegantly with your window.
- ⌨️ **Intuitive Controls:** Simple keystroke detection. Arrow keys to navigate boards, `Ctrl-C` or `Esc` to instantly exit and gracefully restore your terminal.

## 🚀 Installation

### Using Cargo (Recommended)

If you have Rust installed, you can build and install bet directly from crates.io!

```bash
cargo install bet-cli
```

Or install directly from the source:

```bash
cargo install --path .
```

### Manual Build

```bash
# Build the release version
make build

# Install locally to ~/.local/bin
make install

# Or install system-wide
sudo make install-system
```

## 🕹️ Gameplay

Start bet simply by running:

```bash
bet
```

You can also jump directly into a game using the installed aliases (or by passing arguments like `bet hangman`):

```bash
hangman
# or
tictactoe
# or
chess
# or
pong
# or
movie
# or
manga
# or
anime
# or
cartoon
# or
videogame
# or
music
# or
salsa
# or
reggae
```

1. **Select your language:** Press `1-5` to choose from the startup menu.
2. **Select your game:** Press `1` for Hangman, `2` for Tic-Tac-Toe, `3` for Chess, `4` for Pong, or `5` to enter the Media Recommender.
3. **Hangman:** Press any alphabetic key to make a guess before the timer hits 0.0s!
4. **Tic-Tac-Toe:** Use the Arrow Keys to move the cursor, and press `Enter` or `Space` to place your 'X'.

## 🛠️ Tech Stack Highlights

This project stands as a testament to modern CLI development:
- **[Rust](https://www.rust-lang.org/):** Core logic, blazing fast, and memory-safe.
- **[Ratatui](https://ratatui.rs/):** Next-generation Terminal User Interface library.
- **[Crossterm](https://github.com/crossterm-rs/crossterm):** Cross-platform terminal manipulation.

## 🤝 Contributing

Pull requests are fully welcome! Feel free to open issues to discuss bugs or new feature ideas.

## 👾 Join the BET Community!

Are you a fan of terminal games, Rust, or just looking to vibe with cool developers? Join our official **BET Discord Community**! 
We've even built a dope **Terminal QR Generator** right into the game—just press `6` on the main menu to scan and join instantly from your CLI. 

Or simply click the link/scan the QR code below:

[![Discord](https://img.shields.io/discord/1339678126786215987?color=7289da&logo=discord&logoColor=white)](https://discord.gg/MF6fMFURyC)  
<img src="https://api.qrserver.com/v1/create-qr-code/?size=150x150&data=https://discord.gg/MF6fMFURyC" width="150" height="150" alt="Discord QR" />

Come hang out, bet on your skills, and check out what we're building next! 🚀

---
**LYffseba**

## 📝 License

This project is licensed under the **MIT** License.

*(Note: Having an explicit MIT license guarantees that this project is fully open-source. It provides immense value by allowing other developers, organizations, and package managers (like `cargo`) to freely distribute, modify, and integrate `bet` into their own systems without legal friction or liability concerns!)*
