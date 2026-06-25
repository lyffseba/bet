<div align="center">

# 🎮 B$T (BET) - The Pi Agent Game Hub

**Play lightning-fast native TUI games in your terminal while you wait for the LLM to think!**

[![Pi Package](https://img.shields.io/badge/pi-package-blue.svg)](https://github.com/mariozechner/pi-coding-agent)
[![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

## ✨ What is this?

B$T (BET) has fully pivoted into a **native extension package for the `pi` AI coding agent**. 

Because `pi` generates LLM responses in the background, you no longer have to sit and stare at streaming text. Simply type `/b$t` after submitting a prompt, and a beautifully rendered, interactive game hub overlay will appear. You can play games natively using your arrow keys while the model continues to stream its response underneath!

## 🚀 Installation

Install this extension directly into your `pi` agent globally:

```bash
pi install ./bet
```
*(Note: Once published, you will be able to run `pi install npm:bet-pi-hub`)*

## 🕹️ Gameplay

Once installed, simply type the following command at any point in your `pi` session:

```bash
/b$t
```

1. **Select your game:** Press `1` for Tic-Tac-Toe (more games like Hangman, Chess, and Pong are actively being ported from the legacy Rust engine).
2. **Play:** Use the `Arrow Keys` (or `WASD`) to move the cursor, and press `Enter` or `Space` to make your move.
3. **Exit:** Press `Q` or `Esc` to instantly close the overlay and return to the chat stream.

## 🛠️ Architecture Pivot

This project previously existed as a standalone Rust application built with `ratatui`. To provide the ultimate AI developer experience, it has been completely rewritten in TypeScript to run natively within `pi`'s custom TUI rendering engine.

The original Rust source code has been safely archived in the `legacy_rust/` directory for reference during the porting process.

## 🤝 Contributing

We are currently porting the remaining games from the Rust engine to TypeScript. Pull requests are welcome!
