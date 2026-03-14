.PHONY: all build run clean install install-system uninstall uninstall-system

all: build

build:
	cargo build --release

run:
	cargo run --release

clean:
	cargo clean

install: build
	install -d ~/.local/bin
	install -m 755 target/release/bet ~/.local/bin/
	ln -sf ~/.local/bin/bet ~/.local/bin/hangman
	ln -sf ~/.local/bin/bet ~/.local/bin/tictactoe
	ln -sf ~/.local/bin/bet ~/.local/bin/chess

install-system: build
	install -d /usr/local/bin
	install -m 755 target/release/bet /usr/local/bin/
	ln -sf /usr/local/bin/bet /usr/local/bin/hangman
	ln -sf /usr/local/bin/bet /usr/local/bin/tictactoe
	ln -sf /usr/local/bin/bet /usr/local/bin/chess

uninstall:
	rm -f ~/.local/bin/bet ~/.local/bin/hangman ~/.local/bin/tictactoe ~/.local/bin/chess

uninstall-system:
	rm -f /usr/local/bin/bet /usr/local/bin/hangman /usr/local/bin/tictactoe /usr/local/bin/chess
