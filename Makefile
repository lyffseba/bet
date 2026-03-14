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
	ln -sf ~/.local/bin/bet ~/.local/bin/pong
	ln -sf ~/.local/bin/bet ~/.local/bin/movie
	ln -sf ~/.local/bin/bet ~/.local/bin/series
	ln -sf ~/.local/bin/bet ~/.local/bin/manga
	ln -sf ~/.local/bin/bet ~/.local/bin/book
	ln -sf ~/.local/bin/bet ~/.local/bin/anime
	ln -sf ~/.local/bin/bet ~/.local/bin/cartoon
	ln -sf ~/.local/bin/bet ~/.local/bin/recommend
	ln -sf ~/.local/bin/bet ~/.local/bin/music
	ln -sf ~/.local/bin/bet ~/.local/bin/salsa
	ln -sf ~/.local/bin/bet ~/.local/bin/reggae

install-system: build
	install -d /usr/local/bin
	install -m 755 target/release/bet /usr/local/bin/
	ln -sf /usr/local/bin/bet /usr/local/bin/hangman
	ln -sf /usr/local/bin/bet /usr/local/bin/tictactoe
	ln -sf /usr/local/bin/bet /usr/local/bin/chess
	ln -sf /usr/local/bin/bet /usr/local/bin/pong
	ln -sf /usr/local/bin/bet /usr/local/bin/movie
	ln -sf /usr/local/bin/bet /usr/local/bin/series
	ln -sf /usr/local/bin/bet /usr/local/bin/manga
	ln -sf /usr/local/bin/bet /usr/local/bin/book
	ln -sf /usr/local/bin/bet /usr/local/bin/anime
	ln -sf /usr/local/bin/bet /usr/local/bin/cartoon
	ln -sf /usr/local/bin/bet /usr/local/bin/recommend
	ln -sf /usr/local/bin/bet /usr/local/bin/music
	ln -sf /usr/local/bin/bet /usr/local/bin/salsa
	ln -sf /usr/local/bin/bet /usr/local/bin/reggae

uninstall:
	rm -f ~/.local/bin/bet ~/.local/bin/hangman ~/.local/bin/tictactoe ~/.local/bin/chess ~/.local/bin/pong ~/.local/bin/movie ~/.local/bin/series ~/.local/bin/manga ~/.local/bin/book ~/.local/bin/anime ~/.local/bin/cartoon ~/.local/bin/recommend ~/.local/bin/music ~/.local/bin/salsa ~/.local/bin/reggae

uninstall-system:
	rm -f /usr/local/bin/bet /usr/local/bin/hangman /usr/local/bin/tictactoe /usr/local/bin/chess /usr/local/bin/pong /usr/local/bin/movie /usr/local/bin/series /usr/local/bin/manga /usr/local/bin/book /usr/local/bin/anime /usr/local/bin/cartoon /usr/local/bin/recommend /usr/local/bin/music /usr/local/bin/salsa /usr/local/bin/reggae
