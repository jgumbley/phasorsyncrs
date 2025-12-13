.PHONY: digest ingest clean

define success
	@printf '\033[32m\n'; \
	set -- 🕵️ 🔒 📡 🗝️ 🥃; \
	icon_idx=$$(( $$(od -An -N2 -tu2 /dev/urandom | tr -d ' ') % $$# + 1 )); \
	while [ $$icon_idx -gt 1 ]; do shift; icon_idx=$$((icon_idx - 1)); done; \
	icon=$$1; \
	parent_info=$$(ps -o ppid= -p $$$$ 2>/dev/null | tr -d ' '); \
	[ -n "$$parent_info" ] || parent_info="n/a"; \
	printf "%s > \033[33m%s\033[0m accomplished\n" "$$icon" "$(@)"; \
	printf "\033[90m{{{ %s | user=%s | host=%s | procid=%s | parentproc=%s }}}\033[0m\n\033[0m" "$$(date +%Y-%m-%d_%H:%M:%S)" "$$(whoami)" "$$(hostname)" "$$$$" "$$parent_info"
endef

CLIP := $(shell \
	if command -v pbcopy >/dev/null 2>&1; then \
		printf "%s" "pbcopy"; \
	elif command -v wl-copy >/dev/null 2>&1; then \
		printf "%s" "wl-copy"; \
	elif command -v xclip >/dev/null 2>&1; then \
		printf "%s" "xclip -selection clipboard"; \
	elif command -v xsel >/dev/null 2>&1; then \
		printf "%s" "xsel --clipboard --input --logfile /dev/null"; \
	else \
		printf "%s" ""; \
	fi \
)

.venv/: requirements.txt
	uv venv .venv/
	uv pip install -r requirements.txt
	$(call success)

digest:
	@echo "=== Project Digest ==="
	@for file in $$(find . -path "./.uv-cache" -prune -o -type f \( -name "*.py" -o -name "*.md" -o -name "*.txt" -o -name "*.mk" -o -name "Makefile" \) -print | grep -v venv | grep -v __pycache__ | sort); do \
		echo ""; \
		echo "--- $$file ---"; \
		cat "$$file"; \
	done
	$(call success)

ingest:
	@if [ -z "$(CLIP)" ]; then \
		echo "error: no clipboard tool found; install pbcopy (macOS) or wl-copy/xclip/xsel (Linux)"; \
		exit 1; \
	fi
	$(MAKE) digest | $(CLIP)
	$(call success)

clean::
	rm -Rf .venv/
	$(call success)
