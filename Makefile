# Makefile for Rust project

# Dependencies

CARGO ?= cargo

# Targets

.PHONY: run build test check fmt clippy doc unittest slowtest lint clean list-devices followlog

# Main targets
run: build
	$(CARGO) run

list-devices: build
	$(CARGO) run -- --device-list

run-oxi: build
	$(CARGO) run -- --bind-to-device "OXI ONE:OXI ONE MIDI 1 20:0"

build: lint test
	$(CARGO) build

# Testing targets
test: fmt unittest
	@echo "All tests complete (skipping slow running, run with slowtest)"

unittest:
	$(CARGO) test --features test-mock -- --skip ignored

slowtest:
	$(CARGO) test --features test-mock -- --ignored

# Code quality targets
lint: fmt clippy
	@echo "Linting complete"

check:
	$(CARGO) check

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy -- -D warnings -D clippy::cognitive-complexity \
	  -D clippy::too-many-arguments -D clippy::too-many-lines \
	  -D clippy::nonminimal-bool -D clippy::needless_continue \
	  -D clippy::large-enum-variant -D clippy::result_large_err \
	  -D clippy::type-complexity -D clippy::mod_module_files

doc:
	$(CARGO) doc

# Logging
followlog:
	tail -f $(HOME)/.local/share/phasorsyncrs/logs/app.log | ack --passthru WARNING

# Cleanup
clean:
	$(CARGO) clean