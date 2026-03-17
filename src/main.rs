mod platforms;
mod sync;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "init" | "i" => cmd_init(),
        "sync" | "s" => cmd_sync(&args[1..]),
        "push" | "p" => cmd_sync(&args[1..]),  // alias
        "status" | "st" => cmd_status(),
        "import" => cmd_import(&args[1..]),
        "user" => cmd_user(),
        "platforms" | "ls" => cmd_platforms(),
        "help" | "h" | "--help" | "-h" => cmd_help(),
        "version" | "-v" | "--version" => println!("agentsync {}", env!("CARGO_PKG_VERSION")),
        other => {
            eprintln!("Unknown command: {other}\nRun `agentsync help` for usage.");
            std::process::exit(1);
        }
    }
}

fn project_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn home_dir() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"))
}

fn cmd_init() {
    let dir = project_dir();
    match sync::init_source(&dir) {
        Ok(path) => {
            println!("✓ Initialized {}/", path.strip_prefix(&dir).unwrap_or(&path).display());
            println!("  ├── AGENTS.md       ← shared instructions (edit this)");
            println!("  ├── rules/          ← shared rules");
            println!("  ├── skills/         ← shared skills/commands");
            println!("  └── agents/         ← shared agent definitions");
            println!("\nNext: edit .agents/AGENTS.md, add rules/skills, then run `agentsync sync`");
        }
        Err(e) => eprintln!("✗ Init failed: {e}"),
    }
}

fn cmd_sync(args: &[String]) {
    let dir = project_dir();
    let targets: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let results = sync::sync_project(&dir, &targets);

    if results.is_empty() {
        eprintln!("Nothing to sync. Run `agentsync init` first.");
        return;
    }

    let mut total = 0;
    for r in &results {
        if r.files_synced > 0 {
            println!("  ✓ {:10} {} files", r.platform, r.files_synced);
            total += r.files_synced;
        }
        for e in &r.errors {
            eprintln!("  ✗ {:10} {e}", r.platform);
        }
    }

    // Also sync AGENTS.md to project root
    let root = dir.join(".agents/AGENTS.md");
    if root.exists() {
        println!("  ✓ {:10} AGENTS.md (project root)", "universal");
    }

    println!("\n{total} files synced across {} platforms.", results.iter().filter(|r| r.files_synced > 0).count());
}

fn cmd_status() {
    let dir = project_dir();
    let source = dir.join(".agents");

    println!("agentsync status\n");

    // Source
    if source.exists() {
        let rules = count_md_files(&source.join("rules"));
        let skills = count_md_files(&source.join("skills"));
        let agents = count_md_files(&source.join("agents"));
        let has_root = source.join("AGENTS.md").exists();
        println!("Source: .agents/");
        println!("  AGENTS.md  : {}", if has_root { "✓" } else { "✗ missing" });
        println!("  rules/     : {} files", rules);
        println!("  skills/    : {} files", skills);
        println!("  agents/    : {} files", agents);
    } else {
        println!("Source: not initialized (run `agentsync init`)");
        return;
    }

    println!("\nTargets:");
    let detected = sync::detect_platforms(&dir);
    for p in platforms::PLATFORMS {
        let exists = dir.join(p.project_dir).exists();
        let detected = detected.iter().any(|d| d.name == p.name);
        let status = if exists { "✓ found" } else if detected { "○ will create" } else { "- not detected" };
        println!("  {:10} {status}", p.name);
    }

    // User level
    let home = home_dir();
    println!("\nUser-level:");
    for p in platforms::PLATFORMS {
        if let (Some(user_dir), Some(user_md)) = (p.user_dir, p.user_root_md) {
            let user_path = home.join(user_dir.trim_start_matches("~/"));
            if user_path.join(user_md).exists() {
                println!("  {:10} ✓ ~/{}/{}", p.name, user_dir.trim_start_matches("~/"), user_md);
            }
        }
    }
}

fn cmd_import(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: agentsync import <platform>\nPlatforms: {}", platforms::platform_names().join(", "));
        return;
    }
    let platform = &args[0];
    let dir = project_dir();
    match sync::import_from(&dir, platform) {
        Ok(count) => println!("✓ Imported {count} files from {platform} → .agents/"),
        Err(e) => eprintln!("✗ Import failed: {e}"),
    }
}

fn cmd_user() {
    let home = home_dir();
    let source = project_dir().join(".agents");
    if !source.exists() {
        eprintln!("No .agents/ directory. Run `agentsync init` first.");
        return;
    }

    let results = sync::sync_user(&home, &source);
    if results.is_empty() {
        println!("No files to sync to user level.");
        return;
    }

    let mut total = 0;
    for r in &results {
        if r.files_synced > 0 {
            println!("  ✓ {} ({} files)", r.platform, r.files_synced);
            total += r.files_synced;
        }
    }
    println!("\n{total} files synced to user-level configs.");
}

fn cmd_platforms() {
    println!("Supported platforms:\n");
    for p in platforms::PLATFORMS {
        println!("  {:10}  project: {:<15}  root: {}", p.name, p.project_dir, p.root_md);
    }
}

fn cmd_help() {
    println!("agentsync — Sync AI agent configs across all platforms\n");
    println!("Usage: agentsync <command> [args]\n");
    println!("Commands:");
    println!("  init              Create .agents/ source directory");
    println!("  sync [platforms]  Sync .agents/ → all platforms (or specific ones)");
    println!("  import <platform> Import existing platform config into .agents/");
    println!("  user              Sync .agents/ → user-level configs (~/.claude/ etc.)");
    println!("  status            Show source and target status");
    println!("  platforms         List supported platforms");
    println!("  help              Show this help\n");
    println!("Workflow:");
    println!("  1. agentsync init                    # create .agents/");
    println!("  2. agentsync import claude            # import from existing Claude config");
    println!("  3. # edit .agents/AGENTS.md, rules/, skills/");
    println!("  4. agentsync sync                     # push to all platforms");
    println!("  5. agentsync user                     # also sync user-level configs\n");
    println!("Platforms: {}\n", platforms::platform_names().join(", "));
    println!("Source layout:");
    println!("  .agents/");
    println!("  ├── AGENTS.md     → CLAUDE.md, .codex/AGENTS.md, GEMINI.md, ...");
    println!("  ├── rules/        → .claude/rules/, .codex/rules/, .cursor/rules/, ...");
    println!("  ├── skills/       → .claude/commands/, .codex/skills/, .gemini/skills/, ...");
    println!("  └── agents/       → .claude/agents/, .codex/agents/, ...");
}

fn count_md_files(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| entries.flatten().filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md")).count())
        .unwrap_or(0)
}
