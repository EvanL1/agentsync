# aisync

[![CI](https://github.com/EvanL1/aisync/actions/workflows/ci.yml/badge.svg)](https://github.com/EvanL1/aisync/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

English | [中文](README.zh-CN.md)

One command to sync your AI agent configs across **Claude Code, Codex, Gemini CLI, Cursor, Copilot, Windsurf, and Cline**.

```
.agents/          →    CLAUDE.md             (Claude Code)
├── AGENTS.md     →    AGENTS.md             (Codex CLI)
├── rules/        →    GEMINI.md             (Gemini CLI)
├── skills/       →    .cursor/rules/*.mdc   (Cursor)
└── agents/       →    .github/instructions/ (Copilot)
                  →    .windsurf/rules/      (Windsurf)
                  →    .clinerules           (Cline)
```

**One source of truth. Seven platforms. Zero dependencies.**

## The Problem

You maintain `CLAUDE.md` for Claude Code, `.cursorrules` for Cursor, `copilot-instructions.md` for Copilot… and they're all slightly different versions of the same rules. When you update one, you forget the others. When a teammate joins, half the configs are stale.

`aisync` fixes this: write your rules once in `.agents/`, and sync to all platforms with one command. File extensions are auto-converted (`.md` → `.mdc` for Cursor, `.instructions.md` for Copilot).

## Install

### npm (recommended)

```bash
npm install -g aisync
```

### Cargo (all platforms)

```bash
cargo install --git https://github.com/EvanL1/aisync
```

### Homebrew (macOS / Linux)

```bash
brew tap EvanL1/aisync
brew install aisync
```

### Shell script (macOS / Linux / WSL)

```bash
curl -fsSL https://raw.githubusercontent.com/EvanL1/aisync/master/install.sh | bash
```

### Manual download

Download from [Releases](https://github.com/EvanL1/aisync/releases):

| Platform | File |
|----------|------|
| macOS Apple Silicon | `aisync-darwin-aarch64.tar.gz` |
| macOS Intel | `aisync-darwin-x86_64.tar.gz` |
| Linux x86_64 | `aisync-linux-x86_64.tar.gz` |
| Linux ARM64 | `aisync-linux-aarch64.tar.gz` |
| Windows x64 | `aisync-windows-x86_64.zip` |

## Quick Start

```bash
aisync init                  # create .agents/ with starter AGENTS.md
aisync import claude         # pull existing Claude Code config (optional)
# edit .agents/AGENTS.md and .agents/rules/
aisync sync                  # push to all 7 platforms
```

**That's it.** Your rules now work everywhere.

## Real-World Workflow

```bash
# Team lead writes rules once
vim .agents/rules/code-style.md

# Push to all AI tools in ~2ms
aisync sync

# Preview before writing (safe mode)
aisync sync --dry-run

# Only sync specific platforms
aisync sync cursor copilot

# Commit everything — source + generated configs
git add .agents/ .claude/ .cursor/ .github/
git commit -m "chore: update AI agent rules"
```

## Source Layout

```
.agents/
├── AGENTS.md          # Root instructions → synced to each platform's convention
├── rules/             # Shared rules (auto-converted per platform)
│   ├── coding-style.md    → .claude/rules/coding-style.md
│   ├── coding-style.md    → .cursor/rules/coding-style.mdc
│   └── coding-style.md    → .github/instructions/coding-style.instructions.md
├── skills/            # Shared skills/commands
│   └── review.md          → .claude/commands/review.md
└── agents/            # Shared agent definitions
    └── planner.md         → .claude/agents/planner.md
```

## Platform Mapping

| Platform | Root MD | Rules Dir | Rules Ext | Skills Dir |
|----------|---------|-----------|-----------|------------|
| **Claude Code** | `CLAUDE.md` | `.claude/rules/` | `.md` | `.claude/commands/` |
| **Codex CLI** | `AGENTS.md` | — | — | — |
| **Gemini CLI** | `GEMINI.md` | — | — | — |
| **Cursor** | `.cursorrules` | `.cursor/rules/` | `.mdc` | — |
| **Copilot** | `.github/copilot-instructions.md` | `.github/instructions/` | `.instructions.md` | — |
| **Windsurf** | `.windsurfrules` | `.windsurf/rules/` | `.md` | — |
| **Cline** | `.clinerules` | — | — | — |

`AGENTS.md` is always synced to project root as the [universal standard](https://agents.md/).

## Commands

```bash
aisync init                    # Create .agents/ source directory
aisync import <platform>       # Import existing config into .agents/
aisync sync [platform...]      # Sync to all (or specific) platforms
aisync sync --dry-run          # Preview what would be synced
aisync user                    # Sync to user-level (~/.claude/ etc.)
aisync status                  # Show source and target status
aisync platforms               # List supported platforms
```

## Should I commit the generated files?

**Yes.** Commit both `.agents/` (your source of truth) and the generated platform files (`.claude/`, `.cursor/`, `.github/`, etc.). They're all just markdown — no binaries, no build artifacts. This way, every teammate and CI environment gets the right configs without needing to install aisync.

## How It Works

1. **Read** `.agents/` source directory (`.md` files)
2. **Convert** extensions per platform (`.md` → `.mdc` for Cursor, `.instructions.md` for Copilot)
3. **Write** to each platform's expected directory
4. **Root MD** is copied with platform-specific naming (`AGENTS.md` → `CLAUDE.md`, `GEMINI.md`, etc.)

No git hooks, no npm, no config files, no runtime dependencies. Just a single binary (~2ms execution).

## vs Alternatives

| | aisync | aisync-sync | manual copy |
|---|---|---|---|
| Dependencies | **None** (single binary) | Node.js + npm | N/A |
| Config needed | **Zero** | git repo + JSON | N/A |
| Extension conversion | **Automatic** | Manual | Manual |
| Platforms | 7 | ~10 | ∞ |
| Speed | **~2ms** | ~500ms | Slow |

## Contributing

```bash
git clone https://github.com/EvanL1/aisync
cd aisync
cargo build
cargo test
```

PRs welcome! If you'd like to add a new platform, edit `src/platforms.rs` — each platform is a single struct.

## License

MIT
