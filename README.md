# Hangman Terminal Game

A simple, fast, and reliable terminal hangman game written in Rust.

## Features

- Language selection at startup (English, Spanish, Portuguese)
- Solo mode: guess a random movie title (localized titles)
- Multiplayer mode: one player sets a word, the other guesses
- Full UI translation (menu, prompts, messages) in selected language
- Colorful terminal UI with Unicode box‑drawing art
- Smooth animation: body parts appear with a subtle delay
- Line input: type a letter, see it, then press Enter (allows backspace)
- Supports accented letters (e.g., é, ñ, á, ã)
- Lightweight with minimal dependencies (crossterm + rand)

## Requirements

- Rust (stable) and Cargo
- Terminal that supports ANSI colors and Unicode characters

## Installation

### Install for current user (recommended)

```bash
make install
```

This will install the binary to `~/.local/bin`. Ensure that directory is in your `PATH`.

### Install system‑wide (requires sudo)

```bash
sudo make install-system
```

### Manual build

```bash
cargo build --release
./target/release/hangman
```

## Usage

```bash
hangman
```

You will be prompted to select a language (English, Spanish, or Portuguese). Then follow the on‑screen menu:

1. Solo (random movie) - Computer picks a random movie title (in the selected language)
2. Multiplayer (one player sets word) - Player 1 enters a word (hidden), Player 2 guesses
3. Quit / Salir / Sair

## Gameplay

- Guess letters one at a time (type a letter and press Enter)
- You have 6 attempts before the hangman is complete
- Guessed letters are displayed
- After a wrong guess, a new body part is drawn with a short delay
- Accented letters (é, ñ, á, etc.) are supported
- Type 'salir', 'quit', or 'q' to exit a game

## Multiplayer

When selecting multiplayer mode, Player 1 will be prompted to enter a word.
The word will not be displayed (hidden input). After entering the word,
Player 2 can start guessing.

## License

MIT

## Contributing

Pull requests welcome!
