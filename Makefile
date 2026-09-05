.PHONY: all build test test-core test-protocol clippy e2e wasm goldens verify \
	run clean install install-system uninstall uninstall-system \
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
	cargo clippy -p bet-core -p bet-protocol --all-targets -- -D warnings
	cargo clippy -p bet-core -p bet-wasm --target wasm32-unknown-unknown -- -D warnings
	cargo clippy -p bet-cli --all-targets -- -D warnings

e2e: build
	bash scripts/e2e-mp.sh

wasm:
	bash scripts/build-wasm.sh

goldens: wasm
	cd packages/bet-ts && node --experimental-strip-types test/goldens.mts
	cd packages/bet-ts && node --experimental-strip-types test/play.mts

verify:
	bash scripts/verify-engine.sh

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
	npm install
	pi install .

uninstall-extension:
	pi remove bet-pi-hub

typecheck:
	npm run typecheck

ci: verify
	@echo "ci: OK"
