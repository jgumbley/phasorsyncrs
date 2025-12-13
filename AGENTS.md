## How to work in this repo
- Strictly make as entry point to invoke all targets, runtime and test.
- Operate only inside this pwd unless explicitly told otherwise (PWD rules apply).
- Read the Makefile first before using any tools or adding targets.
- Call `make digest` to understand the codebase; it is the sanctioned way to learn the structure.
- All execution happens via make; add or adjust Make targets rather than invoking tools or scripts directly.

## Architectural alignment
- Align with the existing architecture. Reuse what is here; do not reframe components.
- Do not add modules (files, packages, services) unless explicitly approved by the operator.
- The stub worker and spawner are the only additions needed for this milestone, routed through the Makefile.

## Principles (Prime directives)
- YAGNI - build only what this stub needs.
- DRY - if something exists, reuse it.
- KISS - keep it straightforward; no optional branches or toggles.
- No fallbacks - they hide failures; let issues surface immediately.
