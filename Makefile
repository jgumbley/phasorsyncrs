# Makefile for Rust project

# Dependencies

CARGO ?= cargo

# Targets

.PHONY: run build test check fmt clippy doc unittest slowtest lint clean ci clean_log list-devices followlog

# Main targets
run: clean_log build
	$(CARGO) run 

list-devices: clean_log build
	$(CARGO) run -- --device-list && cat app.log

run-oxi: clean_log build
	$(CARGO) run -- --bind-to-device "OXI ONE:OXI ONE MIDI 1 20:0" && cat app.log

build: lint test
	$(CARGO) build

# Testing targets
test: fmt unittest
	@echo "All tests complete (skipping slow running, run with slowtest)"

ci: build
	@echo "CI tests complete"

unittest:
	$(CARGO) test

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
	  -D clippy::type-complexity -D clippy::excessive_nesting

doc:
	$(CARGO) doc


# Cleanup
clean:
	$(CARGO) clean

clean_log:
	> app.log