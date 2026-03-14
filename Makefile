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
	install -m 755 target/release/hangman ~/.local/bin/

install-system: build
	install -m 755 target/release/hangman /usr/local/bin/

uninstall:
	rm -f ~/.local/bin/hangman

uninstall-system:
	rm -f /usr/local/bin/hangman
