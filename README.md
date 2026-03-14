# Hangman Terminal Game

A simple, fast, and reliable terminal hangman game written in Rust.

## Features

- Solo mode: guess a random movie title
- Multiplayer mode: one player sets a word, the other guesses
- Colorful terminal UI
- Lightweight with minimal dependencies

## Requirements

- Rust (stable) and Cargo
- Terminal that supports ANSI colors

## Installation

### Install for current user (recommended)

```bash
make install
```

This will install the binary to `~/.local/bin`. Ensure that directory is in your `PATH`.

### Install system-wide (requires sudo)

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

Follow the on-screen menu:

1. Solo (random movie) - Computer picks a random movie title
2. Multiplayer (one player sets word) - Player 1 enters a word (hidden), Player 2 guesses
3. Quit

## Gameplay

- Guess letters one at a time
- You have 6 attempts before the hangman is complete
- Guessed letters are displayed
- Type 'quit' or 'q' to exit a game

## Multiplayer

When selecting multiplayer mode, Player 1 will be prompted to enter a word.
The word will not be displayed (hidden input). After entering the word,
Player 2 can start guessing.

## License

MIT

## Contributing

Pull requests welcome!
