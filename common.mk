NY: digest ingest clean

# Default uv cache inside the project to avoid permission issues with user-level cache
UV_CACHE_DIR ?= .uv-cache
export UV_CACHE_DIR

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

.venv/: requirements.txt
	@if command -v uv >/dev/null 2>&1; then \
		echo "Using uv for virtualenv"; \
		uv venv .venv/; \
		uv pip install -r requirements.txt; \
	else \
		echo "Using python3 -m venv (uv not found)"; \
		python3 -m venv .venv; \
		. .venv/bin/activate; \
		pip install --upgrade pip; \
		pip install -r requirements.txt; \
	fi
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
	$(MAKE) digest | $(CLIP)
	$(call success)

clean:
	rm -Rf .venv/
	$(call success)
# [ Server container ] -> traffic exits wireguard tunnel and is logged in full, routing traffic onward
