.PHONY: all build test test-core test-protocol clippy e2e wasm goldens run clean \
	install install-system uninstall uninstall-system \
	install-extension uninstall-extension typecheck ci

all: build

build:
	cargo build -p bet-cli --release

test: test-core test-protocol

test-core:
	cargo test -p bet-core

test-protocol:
	cargo test -p bet-protocol

clippy:
	cargo clippy -p bet-core -p bet-protocol -- -D warnings
	cargo clippy -p bet-core -p bet-wasm --target wasm32-unknown-unknown -- -D warnings

e2e: build
	bash scripts/e2e-mp.sh

wasm:
	bash scripts/build-wasm.sh

goldens: wasm
	cd packages/bet-ts && node --experimental-strip-types test/goldens.mts

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

install-extension: wasm
	pi install .

uninstall-extension:
	pi remove bet-pi-hub

typecheck:
	npm run typecheck

ci: test clippy build e2e wasm goldens
	@echo "local CI OK"
