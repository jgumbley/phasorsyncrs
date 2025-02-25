# Makefile for Rust project

define success 
	@tput setaf 2; \
	echo ""; \
	owls="🦉 🦆 🦢 🐦 🦜"; \
	n=$$(expr $$(od -An -N2 -tu2 /dev/urandom | tr -d ' ') % 5 + 1); \
	owl=$$(echo $$owls | cut -d' ' -f$$n); \
	printf "%s > \033[33m%s\033[0m completed [OK]\n" "$$owl" "$(@)"; \
	tput sgr0;
endef

# Dependencies

CARGO ?= cargo

# Targets

.PHONY: run build test check fmt clippy doc lint clean ci clean_log list-devices followlog run-oxi run-test-note

# Main targets
run: clean_log build
	$(CARGO) run

run-oxi: clean_log build
	$(CARGO) run -- --bind-to-device "OXI ONE:OXI ONE MIDI 1 20:0"

run-test-note: clean_log build
	$(CARGO) run -- --test-note --midi-output "Midi Through Port-0"

fmt:
	$(CARGO) fmt --all
	$(call success)

clippy: fmt
	$(CARGO) clippy -- -D warnings -D clippy::cognitive-complexity \
	  -D clippy::too-many-arguments -D clippy::too-many-lines \
	  -D clippy::nonminimal-bool -D clippy::needless_continue \
	  -D clippy::large-enum-variant -D clippy::result_large_err \
	  -D clippy::type-complexity -D clippy::excessive-nesting
	$(call success)

lint:
	@echo "Linting complete"

test: clippy
	$(CARGO) test
	$(call success)

build: test
	$(CARGO) build
	$(call success)

# Testing targets

ci: build
	@echo "CI tests complete"
	$(call success)

coverage:
	cargo llvm-cov --summary-only
	$(call success)

# Code quality targets

check:
	$(CARGO) check
	$(call success)


doc:
	$(CARGO) doc
	$(call success)


# Cleanup
clean:
	$(CARGO) clean
	$(call success)

clean_log:
	> app.log
	$(call success)