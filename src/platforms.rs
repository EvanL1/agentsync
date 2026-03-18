// Platform definitions — verified config paths for each AI coding tool
//
// Sources:
//   Claude Code: https://code.claude.com/docs/en/skills
//   Codex CLI:   https://developers.openai.com/codex/guides/agents-md/
//   Gemini CLI:  https://google-gemini.github.io/gemini-cli/docs/get-started/configuration.html
//   Cursor:      https://docs.cursor.com/context/rules
//   Copilot:     https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot
//   Windsurf:    https://windsurf.com/editor/directory
//   Cline:       https://docs.cline.bot/cline-cli/configuration

pub struct Platform {
    pub name: &'static str,

    // Root instruction file
    pub root_md: &'static str,            // filename: CLAUDE.md, AGENTS.md, GEMINI.md, etc.
    pub root_md_in_subdir: bool,          // true = inside project_dir, false = project root

    // Project-level directories
    pub project_dir: &'static str,        // .claude, .cursor, .github, etc.
    pub rules_dir: Option<&'static str>,  // subdirectory for rules (None = not supported)
    pub rules_ext: &'static str,          // file extension for rules: "md", "mdc", "instructions.md"
    pub skills_dir: Option<&'static str>, // subdirectory for skills/commands
    pub agents_dir: Option<&'static str>, // subdirectory for agents

    // User-level (global) directory
    pub user_dir: Option<&'static str>,   // ~/.claude, ~/.codex, ~/.gemini, etc.
    pub user_root_md: Option<&'static str>, // root md filename in user dir (may differ)
}

pub const PLATFORMS: &[Platform] = &[
    // ── Claude Code ──
    // CLAUDE.md at project root, .claude/{rules,commands,agents}/*.md
    // User: ~/.claude/CLAUDE.md, ~/.claude/{rules,commands}/*.md
    Platform {
        name: "claude",
        root_md: "CLAUDE.md",
        root_md_in_subdir: false,

        project_dir: ".claude",
        rules_dir: Some("rules"),
        rules_ext: "md",
        skills_dir: Some("commands"),     // Claude calls them "commands" (slash commands)
        agents_dir: Some("agents"),
        user_dir: Some("~/.claude"),
        user_root_md: Some("CLAUDE.md"),
    },

    // ── Codex CLI (OpenAI) ──
    // AGENTS.md at project root (walks up to git root)
    // User: ~/.codex/AGENTS.md or AGENTS.override.md
    // No subdirectory convention for rules/skills
    Platform {
        name: "codex",
        root_md: "AGENTS.md",
        root_md_in_subdir: false,         // AGENTS.md at project root

        project_dir: ".codex",
        rules_dir: None,                  // Codex has no rules subdirectory
        rules_ext: "md",
        skills_dir: None,                 // Codex has no skills subdirectory
        agents_dir: None,
        user_dir: Some("~/.codex"),
        user_root_md: Some("AGENTS.md"),  // or AGENTS.override.md
    },

    // ── Gemini CLI (Google) ──
    // GEMINI.md at project root, .gemini/commands/*.toml for slash commands
    // User: ~/.gemini/GEMINI.md, ~/.gemini/commands/*.toml
    Platform {
        name: "gemini",
        root_md: "GEMINI.md",
        root_md_in_subdir: false,         // GEMINI.md at project root

        project_dir: ".gemini",
        rules_dir: None,                  // Gemini has no rules dir, uses GEMINI.md
        rules_ext: "md",
        skills_dir: None,                 // commands are .toml, not .md — skip for now
        agents_dir: None,
        user_dir: Some("~/.gemini"),
        user_root_md: Some("GEMINI.md"),
    },

    // ── Cursor ──
    // .cursorrules at project root (legacy), .cursor/rules/*.mdc (current)
    // User: ~/.cursor/rules/*.mdc
    Platform {
        name: "cursor",
        root_md: ".cursorrules",
        root_md_in_subdir: false,

        project_dir: ".cursor",
        rules_dir: Some("rules"),
        rules_ext: "mdc",                // Cursor uses .mdc format, not .md
        skills_dir: None,
        agents_dir: None,
        user_dir: Some("~/.cursor"),
        user_root_md: None,              // Cursor uses .cursor/rules/ not a root md
    },

    // ── GitHub Copilot ──
    // .github/copilot-instructions.md, .github/instructions/*.instructions.md
    // No user-level config directory
    Platform {
        name: "copilot",
        root_md: "copilot-instructions.md",
        root_md_in_subdir: true,          // .github/copilot-instructions.md

        project_dir: ".github",
        rules_dir: Some("instructions"),
        rules_ext: "instructions.md",     // Copilot uses .instructions.md suffix
        skills_dir: None,
        agents_dir: None,
        user_dir: None,                   // No user-level directory
        user_root_md: None,
    },

    // ── Windsurf ──
    // .windsurfrules at project root, .windsurf/rules/*.md
    Platform {
        name: "windsurf",
        root_md: ".windsurfrules",
        root_md_in_subdir: false,

        project_dir: ".windsurf",
        rules_dir: Some("rules"),
        rules_ext: "md",
        skills_dir: None,
        agents_dir: None,
        user_dir: None,
        user_root_md: None,
    },

    // ── Cline ──
    // .clinerules at project root (single file) OR .clinerules/ directory with multiple files
    // User: ~/.cline/
    Platform {
        name: "cline",
        root_md: ".clinerules",
        root_md_in_subdir: false,
        project_dir: ".clinerules",       // .clinerules/ doubles as the rules dir
        rules_dir: None,                  // rules go directly in .clinerules/
        rules_ext: "md",
        skills_dir: None,
        agents_dir: None,
        user_dir: Some("~/.cline"),
        user_root_md: None,
    },
];

/// AGENTS.md is the universal standard — always synced to project root
pub const UNIVERSAL_ROOT_MD: &str = "AGENTS.md";

pub fn find_platform(name: &str) -> Option<&'static Platform> {
    PLATFORMS.iter().find(|p| p.name == name)
}

pub fn platform_names() -> Vec<&'static str> {
    PLATFORMS.iter().map(|p| p.name).collect()
}
