.PHONY: all build run clean install install-system uninstall uninstall-system install-extension uninstall-extension

all: build install-extension

build:
	cargo build --release

run:
	cargo run --release

clean:
	cargo clean

install: build
	install -d ~/.local/bin
	install -m 755 target/release/bet ~/.local/bin/

install-system: build
	install -d /usr/local/bin
	install -m 755 target/release/bet /usr/local/bin/

uninstall:
	rm -f ~/.local/bin/bet

uninstall-system:
	rm -f /usr/local/bin/bet

install-extension:
	pi install .

uninstall-extension:
	pi remove bet-pi-hub
