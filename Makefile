# Makefile for Rust project

# Targets for common development tasks

# Dependencies

CARGO ?= cargo

# Targets

build:

	$(CARGO) build

test:

	$(CARGO) test

check:

	$(CARGO) check

fmt:

	$(CARGO) fmt

clippy:

	$(CARGO) clippy

doc:

	$(CARGO) doc