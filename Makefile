# Makefile for Rust project

include common.mk

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

.PHONY: run build test check fmt clippy doc lint clean ci clean_log list-devices followlog run-oxi run-direct-test deps play-wavs

deps:
	@if command -v pkg-config >/dev/null 2>&1 && pkg-config --exists alsa; then \
		echo "ALSA development libraries already installed"; \
	else \
		echo "Installing ALSA development libraries (requires sudo)"; \
		if command -v apt-get >/dev/null 2>&1; then \
			sudo apt-get update; \
			sudo apt-get install -y libasound2-dev pkg-config alsa-utils; \
		elif command -v dnf >/dev/null 2>&1; then \
			sudo dnf install -y alsa-lib-devel pkg-config alsa-utils; \
		else \
			echo "Please install ALSA development libs (libasound2-dev/alsa-lib-devel) and pkg-config"; \
			exit 1; \
		fi \
	fi
	$(call success)

# Main targets
run: deps clean_log build
	$(CARGO) run

run-oxi: deps clean_log build
	$(CARGO) run -- --bind-to-device "OXI ONE:OXI ONE MIDI 1 20:0"

fmt:
	$(CARGO) fmt --all
	$(call success)

clippy: deps fmt
	$(CARGO) clippy -- -D warnings -D clippy::cognitive-complexity \
	  -D clippy::too-many-arguments -D clippy::too-many-lines \
	  -D clippy::nonminimal-bool -D clippy::needless_continue \
	  -D clippy::large-enum-variant -D clippy::result_large_err \
	  -D clippy::type-complexity -D clippy::excessive-nesting
	$(call success)

lint:
	@echo "Linting complete"

list-devices:
	@echo "ALSA playback devices:"
	@if command -v aplay >/dev/null 2>&1; then aplay -l || true; else echo "aplay not available"; fi
	@echo ""
	@echo "ALSA capture devices:"
	@if command -v arecord >/dev/null 2>&1; then arecord -l || true; else echo "arecord not available"; fi
	@echo ""
	@echo "ALSA sequencer clients:"
	@if command -v aconnect >/dev/null 2>&1; then aconnect -l || true; else echo "aconnect not available"; fi
	$(call success)

play-wavs: deps
	@if [ ! -d wav_files ]; then \
		echo "wav_files directory not found"; \
		exit 1; \
	fi
	@set -e; \
	files=$$(find wav_files -maxdepth 1 -type f -name '*.wav' | sort); \
	if [ -z "$$files" ]; then \
		echo "No wav files to play"; \
		exit 1; \
	fi; \
	for f in $$files; do \
		echo "Playing $$f"; \
		aplay "$$f"; \
	done
	$(call success)

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
clean::
	$(CARGO) clean
	$(call success)

clean_log:
	> app.log
	$(call success)
