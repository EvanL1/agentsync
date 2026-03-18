# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is this project?

**aitoolsync** (`aisync`) is a zero-dependency Rust CLI that syncs AI agent configs from a single `.agents/` source directory to 7 platforms: Claude Code, Codex CLI, Gemini CLI, Cursor, Copilot, Windsurf, and Cline. It auto-converts file extensions per platform (`.md` → `.mdc` for Cursor, `.instructions.md` for Copilot).

Binary name: `aisync`. Crate name: `aitoolsync`.

## Build & Test

```bash
cargo build                  # debug build
cargo build --release        # release build
cargo check                  # type-check only
cargo clippy -- -D warnings  # lint (CI enforced, warnings = errors)
cargo test                   # run all tests
```

No external dependencies — the project uses only `std`.

## Architecture

Three source files, each with a clear responsibility:

- **`src/main.rs`** — CLI entry point. Manual arg parsing (no clap). Dispatches subcommands: `init`, `sync`/`push`, `status`, `import`, `user`, `platforms`, `help`, `version`. Contains ANSI color helpers.
- **`src/platforms.rs`** — Static platform definitions. Each platform is a `Platform` struct with paths for root MD, rules dir, skills dir, agents dir, and user-level dir. The `PLATFORMS` const array is the single source of truth for all platform config paths.
- **`src/sync.rs`** — Core sync engine. Handles: `init_source` (scaffold `.agents/`), `sync_project` (`.agents/` → platform dirs), `sync_user` (→ user-level `~/` dirs), `import_from` (reverse: platform → `.agents/`), `detect_platforms`. Rules sync includes extension conversion logic.

### Key data flow

```
.agents/AGENTS.md  →  CLAUDE.md, AGENTS.md, GEMINI.md, .cursorrules, ...
.agents/rules/*.md →  .claude/rules/*.md, .cursor/rules/*.mdc, .github/instructions/*.instructions.md, ...
.agents/skills/*.md → .claude/commands/*.md
.agents/agents/*.md → .claude/agents/*.md
```

### Adding a new platform

Add a `Platform` struct entry to the `PLATFORMS` array in `src/platforms.rs`. The sync engine picks it up automatically — no other changes needed.

## Distribution

- **npm**: `npm/` directory contains a wrapper package that downloads the binary via `postinstall` (`npm/install.js`)
- **Homebrew**: Separate tap repo `EvanL1/homebrew-aitoolsync`, auto-updated by release workflow
- **Shell script**: `install.sh` for curl-pipe-bash install
- **GitHub Releases**: Cross-compiled binaries for macOS (x86_64/aarch64), Linux (x86_64/aarch64), Windows (x86_64)

## CI/CD

- **CI** (`.github/workflows/ci.yml`): `cargo check` + `cargo clippy` + `cargo test` on push/PR to master
- **Release** (`.github/workflows/release.yml`): Triggered by `v*` tags. Builds 5 platform targets, creates GitHub release, publishes to npm, updates Homebrew formula
