How to work in this repo

    Strictly make as entry point to invoke all targets, runtime and test.
    Operate only inside this pwd unless explicitly told otherwise (PWD rules apply).
    Read the Makefile first before using any tools or adding targets.
    Call make digest to understand the codebase; it is the sanctioned way to learn the structure.
    All execution happens via make; add or adjust Make targets rather than invoking tools or scripts directly.

Architectural alignment

    Align with the existing architecture. Reuse what is here; do not reframe components.
    Do not add modules (files, packages, services) unless explicitly approved by the operator.
    The stub worker and spawner are the only additions needed for this milestone, routed through the Makefile.

Principles (Prime directives)

    YAGNI - build only what this stub needs.
    DRY - if something exists, reuse it.
    KISS - keep it straightforward; no optional branches or toggles.
    No fallbacks - they hide failures; let issues surface immediately.

This repository occasionally uses coding agents (Claude Code, Codex, etc.) to drive make targets that pause for sudo/BECOME prompts. The agents do not receive secrets, so we route those commands through tmux and let a human type passwords directly inside the pane.
Quick Start: make agent-core

    Start or attach to a tmux session in this repository.
    Run make agent-core.
    The target calls scripts/run-in-agent-pane.sh, which splits a pane above the current one and starts make core there.
    Pane split sizing: AGENT_PANE_PERCENT sets approximate height; we compute a minimum line count to avoid tmux "size missing" errors.

If you leave the pane open, you can rejoin it later with tmux join-pane -sb <pane-id>. The script prints the pane id after it starts, so the agent and human can refer to the same identifier.
When to use this pattern

    You need to watch long-running output together with the agent.
    You want the agent to recover context by scraping tmux instead of re-running.

You can reuse the helper for other commands:

make agent-core
# or
bash scripts/run-in-agent-pane.sh rpi-flash make -C setup-rpi flash

Scripts (minimal helpers)

    scripts/run-in-agent-pane.sh – splits a new tmux pane above and runs the requested make target/command.
    scripts/tmux-agent-pane.sh – sets the pane title, streams the command, and leaves the pane open for review.
