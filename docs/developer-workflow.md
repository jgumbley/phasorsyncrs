# Rust Development Workflow Guide

This document outlines a Makefile-based workflow for Rust projects, following patterns established in our Python projects while respecting Rust ecosystem conventions.

## Core Makefile Targets

```makefile
# Environment setup
setup:
    @which cargo || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    cargo install cargo-edit cargo-watch
    rustup component add clippy rustfmt

# Linting and formatting
lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

# Testing
unit: lint
    cargo test --quiet -- --nocapture

test-watch:
    cargo watch -x 'test -- --nocapture'

# Building and running
run-release:
    cargo run --release

run-debug:
    cargo run

# Documentation
docs:
    cargo doc --no-deps --open

# Cleanup
clean:
    cargo clean
    rm -rf target

.PHONY: setup lint unit test-watch run-release run-debug docs clean
```

## Key Differences from Python Workflow

1. **Dependency Management**
   - Uses Cargo.toml instead of requirements.txt
   - No virtual environment needed - isolation via Cargo workspaces

2. **Tooling**
   - `cargo fmt` (rustfmt) replaces flake8 formatting
   - `cargo clippy` provides enhanced linting
   - `cargo watch` enables test/file watching

3. **Build Profiles**
   - Debug builds (default) for development
   - Release builds for production optimization
   - Cross-compilation support via rustup targets

4. **Documentation**
   - Built-in `cargo doc` generates API docs
   - Docs.rs hosts published crate documentation
   - MDBook integration for project documentation

## Recommended Workflow

1. `make setup` - One-time environment setup
2. `make run-debug` - Develop with debug symbols
3. `make test-watch` - Run tests on file changes
4. `make lint` - Check code quality before commits
5. `make docs` - Verify API documentation

## Advanced Patterns

```makefile
# Cross-compilation example
build-arm:
    rustup target add aarch64-unknown-linux-gnu
    cargo build --target aarch64-unknown-linux-gnu --release

# Benchmarking
bench:
    cargo bench --quiet

# Security auditing
audit:
    cargo audit
```

Follow Cargo's convention-over-configuration approach while maintaining the Makefile as a thin wrapper for complex operations and team consistency.