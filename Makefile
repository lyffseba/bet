.PHONY: all build run clean install

all: build

build:
	cargo build --release

run:
	cargo run --release

clean:
	cargo clean

install: build
	install -m 755 target/release/hangman /usr/local/bin/

uninstall:
	rm -f /usr/local/bin/hangman
