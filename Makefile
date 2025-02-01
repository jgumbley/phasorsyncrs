# Makefile for Rust project

# Dependencies

CARGO ?= cargo

# Targets

.PHONY: run build test check fmt clippy doc unittest lint clean list-devices

# Main targets
run: build
	$(CARGO) run

list-devices: build
	$(CARGO) run -- --device-list

build: lint test
	$(CARGO) build

# Testing targets
test: fmt unittest
	$(CARGO) test --features test-mock

unittest:
	$(CARGO) test --features test-mock

# Code quality targets
lint: fmt clippy
	@echo "Linting complete"

check:
	$(CARGO) check

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy -- -D warnings

doc:
	$(CARGO) doc

# Cleanup
clean:
	$(CARGO) clean