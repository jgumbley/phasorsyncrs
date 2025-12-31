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

.PHONY: run build test check fmt clippy doc lint clean ci clean_log list-devices followlog user-shell run-oxi run-bind run-direct-test deps play-wavs sample-wav umc1820-hw-params umc1820-record umc1820-record-stereo umc1820-mixer arecord-app-capture arecord-umc1820 record run-umc1820

UMC1820_DEV ?= hw:UMC1820,0
UMC1820_PLUG_DEV ?= plughw:UMC1820,0
UMC1820_RATE ?= 48000
UMC1820_FORMAT ?= S32_LE
UMC1820_CHANNELS ?= 18
UMC1820_SECONDS ?= 30
UMC1820_OUT ?= wav_files/umc1820_$(UMC1820_CHANNELS)ch_$(UMC1820_RATE)Hz.wav

# Audio monitor/capture devices used by the runtime (ALSA -D arguments)
AUDIO_CAPTURE_DEV ?= default
AUDIO_PLAYBACK_DEV ?= default

# User-facing ALSA capture knobs (mirrors app defaults; mapped to PHASOR_* envs internally)
ALSA_CAPTURE_DEVICE ?= $(AUDIO_CAPTURE_DEV)
ALSA_CAPTURE_STEREO_PAIR ?= 1
ALSA_ARECORD_SECONDS ?= 15
ALSA_ARECORD_SAMPLE_RATE ?= 48000
ALSA_ARECORD_CHANNELS ?= 2
ALSA_ARECORD_FORMAT ?= S32_LE
ALSA_ARECORD_TEMPLATE ?= wav_files/take_%Y%m%d_%H%M%S_pair1.wav
BPM ?= 120

# Map ALSA_* -> PHASOR_* so the runtime picks up the same settings
export PHASOR_ALSA_CAPTURE_DEVICE=$(ALSA_CAPTURE_DEVICE)
export PHASOR_ALSA_CAPTURE_STEREO_PAIR=$(ALSA_CAPTURE_STEREO_PAIR)
export PHASOR_ARECORD_SECONDS=$(ALSA_ARECORD_SECONDS)
export PHASOR_ARECORD_SAMPLE_RATE=$(ALSA_ARECORD_SAMPLE_RATE)
export PHASOR_ARECORD_CHANNELS=$(ALSA_ARECORD_CHANNELS)
export PHASOR_ARECORD_FORMAT=$(ALSA_ARECORD_FORMAT)
export PHASOR_ARECORD_TEMPLATE=$(ALSA_ARECORD_TEMPLATE)
export AUDIO_CAPTURE_DEV

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
	PHASOR_ALSA_CAPTURE_DEVICE="$(ALSA_CAPTURE_DEVICE)" PHASOR_ALSA_CAPTURE_STEREO_PAIR="$(ALSA_CAPTURE_STEREO_PAIR)" PHASOR_ALSA_PLAYBACK_DEVICE="$(AUDIO_PLAYBACK_DEV)" $(CARGO) run -- --bpm "$(BPM)"

run-oxi: deps clean_log build
	$(CARGO) run -- --bind-to-device "OXI ONE:OXI ONE MIDI 1 20:0"

run-bind: deps clean_log build
	@if [ -z "$(MIDI_IN)" ]; then \
		echo "error: set MIDI_IN to a substring of the desired ALSA MIDI input port (try: make list-devices)"; \
		exit 1; \
	fi
	$(CARGO) run -- --bind-to-device "$(MIDI_IN)"

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

arecord-app-capture: deps
	@mkdir -p wav_files
	@pair="$${ALSA_CAPTURE_STEREO_PAIR:-$(ALSA_CAPTURE_STEREO_PAIR)}"; \
	device="$${ALSA_CAPTURE_DEVICE:-$(ALSA_CAPTURE_DEVICE)}"; \
	seconds="$${ALSA_ARECORD_SECONDS:-$(ALSA_ARECORD_SECONDS)}"; \
	if [ "$$pair" != "1" ]; then \
		echo "arecord backend currently supports only stereo pair 1/2; set ALSA_CAPTURE_STEREO_PAIR=1"; \
		exit 1; \
	fi; \
	if [ -z "$$device" ]; then \
	echo "ALSA_CAPTURE_DEVICE is empty; set it to an ALSA device string (e.g. default, plughw:Card,Device)"; \
	exit 1; \
fi; \
echo "Recording $$device for $$seconds seconds to $(ALSA_ARECORD_TEMPLATE)"; \
arecord -D "$$device" -f $(ALSA_ARECORD_FORMAT) -r $(ALSA_ARECORD_SAMPLE_RATE) -c $(ALSA_ARECORD_CHANNELS) -t wav -N --use-strftime -d "$$seconds" "$(ALSA_ARECORD_TEMPLATE)"
	$(call success)

arecord-umc1820: record

record: deps
	@mkdir -p wav_files
	@if [ ! -t 0 ]; then \
		echo "error: this target needs an interactive terminal (stdin)"; \
		exit 1; \
	fi
	@device="$(UMC1820_PLUG_DEV)"; \
	if [ -z "$$device" ]; then \
		echo "error: UMC1820_PLUG_DEV is empty"; \
		exit 1; \
	fi; \
	echo ""; \
	echo "Recording from $$device to $(ALSA_ARECORD_TEMPLATE)"; \
	echo "Press Enter to stop and finalize the WAV file."; \
	arecord -D "$$device" -f $(ALSA_ARECORD_FORMAT) -r $(ALSA_ARECORD_SAMPLE_RATE) -c $(ALSA_ARECORD_CHANNELS) -t wav -N --use-strftime "$(ALSA_ARECORD_TEMPLATE)" & \
	pid=$$!; \
	sleep 0.1; \
	if ! kill -0 $$pid 2>/dev/null; then \
		wait $$pid; \
		rc=$$?; \
		echo "arecord exited immediately with status $$rc"; \
		exit $$rc; \
	fi; \
	read -r _; \
	echo "Stopping..."; \
	if kill -0 $$pid 2>/dev/null; then kill -TERM $$pid; fi; \
	wait $$pid; \
	rc=$$?; \
	if [ $$rc -ne 0 ] && [ $$rc -ne 143 ]; then \
		echo "arecord exited with status $$rc"; \
		exit $$rc; \
	fi
	$(call success)

run-umc1820:
	ALSA_CAPTURE_DEVICE=plughw:UMC1820,0 AUDIO_PLAYBACK_DEV=plughw:UMC1820,0 BPM=$(BPM) $(MAKE) run

umc1820-hw-params: deps
	arecord -D $(UMC1820_DEV) --dump-hw-params -f $(UMC1820_FORMAT) -r $(UMC1820_RATE) -c $(UMC1820_CHANNELS) /dev/null
	$(call success)

umc1820-record: deps
	@mkdir -p wav_files
	arecord -D $(UMC1820_DEV) -f $(UMC1820_FORMAT) -r $(UMC1820_RATE) -c $(UMC1820_CHANNELS) -d $(UMC1820_SECONDS) $(UMC1820_OUT)
	$(call success)

umc1820-record-stereo: deps
	@mkdir -p wav_files
	arecord -D $(UMC1820_PLUG_DEV) -f $(UMC1820_FORMAT) -r $(UMC1820_RATE) -c 2 -d $(UMC1820_SECONDS) wav_files/umc1820_stereo_$(UMC1820_RATE)Hz.wav
	$(call success)

umc1820-mixer: deps
	alsamixer
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
		aplay -D "$(AUDIO_PLAYBACK_DEV)" "$$f"; \
	done
	$(call success)

sample-wav:
	@python3 -c 'import os, math, struct, wave; os.makedirs("wav_files", exist_ok=True); sr=44100; dur=1.0; freq=440.0; amp=0.2; n=int(sr*dur); frames=b"".join(struct.pack("<hh", int(amp*32767*math.sin(2*math.pi*freq*i/sr)), int(amp*32767*math.sin(2*math.pi*freq*i/sr))) for i in range(n)); w=wave.open("wav_files/sample.wav","wb"); w.setnchannels(2); w.setsampwidth(2); w.setframerate(sr); w.writeframes(frames); w.close()'
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

followlog:
	tail -n 200 -f app.log

user-shell:
	@bash
