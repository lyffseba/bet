.PHONY: all build test test-core run clean install install-system uninstall \
	install-extension uninstall-extension typecheck ci

all: build install-extension

build:
	cargo build -p bet-cli --release

test: test-core

test-core:
	cargo test -p bet-core

run:
	cargo run -p bet-cli --release

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

typecheck:
	npm run typecheck

ci: test-core test-protocol build
	@echo "CI rust targets OK (run npm i && make typecheck for TS)"

test-protocol:
	cargo test -p bet-protocol
