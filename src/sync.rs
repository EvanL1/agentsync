use std::fs;
use std::path::{Path, PathBuf};
use crate::platforms::{Platform, PLATFORMS, UNIVERSAL_ROOT_MD};

const SOURCE_DIR: &str = ".agents";

pub struct SyncResult {
    pub platform: String,
    pub files_synced: usize,
    pub errors: Vec<String>,
}

pub fn detect_platforms(project_dir: &Path) -> Vec<&'static Platform> {
    PLATFORMS
        .iter()
        .filter(|p| project_dir.join(p.project_dir).exists() || project_dir.join(SOURCE_DIR).exists())
        .collect()
}

/// Sync from .agents/ source to all (or specified) platforms
pub fn sync_project(project_dir: &Path, targets: &[&str]) -> Vec<SyncResult> {
    let source = project_dir.join(SOURCE_DIR);
    if !source.exists() {
        eprintln!("No {} directory found. Run `agentsync init` first.", SOURCE_DIR);
        return vec![];
    }

    let platforms: Vec<&Platform> = if targets.is_empty() {
        PLATFORMS.iter().collect()
    } else {
        targets.iter().filter_map(|t| crate::platforms::find_platform(t)).collect()
    };

    // Sync AGENTS.md to project root (universal)
    let root_md_src = source.join("AGENTS.md");
    if root_md_src.exists() {
        let _ = fs::copy(&root_md_src, project_dir.join(UNIVERSAL_ROOT_MD));
    }

    platforms.iter().map(|p| sync_platform(project_dir, &source, p)).collect()
}

fn sync_platform(project_dir: &Path, source: &Path, platform: &Platform) -> SyncResult {
    let mut result = SyncResult {
        platform: platform.name.to_string(),
        files_synced: 0,
        errors: vec![],
    };

    let target_base = project_dir.join(platform.project_dir);

    // 1. Root MD: AGENTS.md → platform's root md
    let root_md_src = source.join("AGENTS.md");
    if root_md_src.exists() {
        let dest = if platform.root_md_in_subdir {
            target_base.join(platform.root_md)
        } else {
            project_dir.join(platform.root_md)
        };

        match ensure_copy(&root_md_src, &dest) {
            Ok(_) => result.files_synced += 1,
            Err(e) => result.errors.push(format!("{}: {e}", platform.root_md)),
        }
    }

    // 2. Rules — respect platform's extension convention
    if let Some(rules_subdir) = platform.rules_dir {
        let rules_src = source.join("rules");
        if rules_src.is_dir() {
            let rules_dest = target_base.join(rules_subdir);
            result.files_synced += sync_rules(&rules_src, &rules_dest, platform.rules_ext, &mut result.errors);
        }
    }

    // 3. Skills/Commands
    if let Some(skills_subdir) = platform.skills_dir {
        let skills_src = source.join("skills");
        if skills_src.is_dir() {
            let skills_dest = target_base.join(skills_subdir);
            result.files_synced += copy_md_dir(&skills_src, &skills_dest, &mut result.errors);
        }
    }

    // 4. Agents
    if let Some(agents_subdir) = platform.agents_dir {
        let agents_src = source.join("agents");
        if agents_src.is_dir() {
            let agents_dest = target_base.join(agents_subdir);
            result.files_synced += copy_md_dir(&agents_src, &agents_dest, &mut result.errors);
        }
    }

    result
}

/// Sync rules with extension conversion:
///   .md source → .mdc for Cursor
///   .md source → .instructions.md for Copilot
fn sync_rules(src: &Path, dest: &Path, target_ext: &str, errors: &mut Vec<String>) -> usize {
    let mut count = 0;
    let entries = match fs::read_dir(src) {
        Ok(e) => e,
        Err(e) => { errors.push(format!("read {}: {e}", src.display())); return 0; }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }

        let stem = path.file_stem().unwrap().to_string_lossy();
        let dest_name = if target_ext == "md" {
            format!("{stem}.md")
        } else if target_ext == "mdc" {
            format!("{stem}.mdc")
        } else if target_ext == "instructions.md" {
            format!("{stem}.instructions.md")
        } else {
            format!("{stem}.{target_ext}")
        };

        let dest_file = dest.join(&dest_name);
        match ensure_copy(&path, &dest_file) {
            Ok(_) => count += 1,
            Err(e) => errors.push(format!("{dest_name}: {e}")),
        }
    }
    count
}

/// Copy all .md files as-is
fn copy_md_dir(src: &Path, dest: &Path, errors: &mut Vec<String>) -> usize {
    let mut count = 0;
    let entries = match fs::read_dir(src) {
        Ok(e) => e,
        Err(e) => { errors.push(format!("read {}: {e}", src.display())); return 0; }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let dest_file = dest.join(path.file_name().unwrap());
            match ensure_copy(&path, &dest_file) {
                Ok(_) => count += 1,
                Err(e) => errors.push(format!("{}: {e}", path.file_name().unwrap().to_string_lossy())),
            }
        }
    }
    count
}

/// Sync user-level configs
pub fn sync_user(home: &Path, source: &Path) -> Vec<SyncResult> {
    let root_md_src = source.join("AGENTS.md");
    let mut results = Vec::new();

    for platform in PLATFORMS {
        let user_dir_str = match platform.user_dir {
            Some(d) => d,
            None => continue,
        };

        let mut result = SyncResult {
            platform: format!("~/{}", user_dir_str.trim_start_matches("~/")),
            files_synced: 0,
            errors: vec![],
        };

        let user_dir = home.join(user_dir_str.trim_start_matches("~/"));

        // Root md
        if root_md_src.exists() {
            if let Some(user_md) = platform.user_root_md {
                let dest = user_dir.join(user_md);
                match ensure_copy(&root_md_src, &dest) {
                    Ok(_) => result.files_synced += 1,
                    Err(e) => result.errors.push(format!("{user_md}: {e}")),
                }
            }
        }

        // Rules
        if let Some(rules_subdir) = platform.rules_dir {
            let rules_src = source.join("rules");
            if rules_src.is_dir() {
                let rules_dest = user_dir.join(rules_subdir);
                result.files_synced += sync_rules(&rules_src, &rules_dest, platform.rules_ext, &mut result.errors);
            }
        }

        // Skills
        if let Some(skills_subdir) = platform.skills_dir {
            let skills_src = source.join("skills");
            if skills_src.is_dir() {
                let skills_dest = user_dir.join(skills_subdir);
                result.files_synced += copy_md_dir(&skills_src, &skills_dest, &mut result.errors);
            }
        }

        if result.files_synced > 0 {
            results.push(result);
        }
    }
    results
}

/// Import existing platform configs into .agents/ source
pub fn import_from(project_dir: &Path, platform_name: &str) -> Result<usize, String> {
    let platform = crate::platforms::find_platform(platform_name)
        .ok_or_else(|| format!("Unknown platform: {platform_name}"))?;

    let source = project_dir.join(SOURCE_DIR);
    fs::create_dir_all(source.join("rules")).map_err(|e| e.to_string())?;
    fs::create_dir_all(source.join("skills")).map_err(|e| e.to_string())?;
    fs::create_dir_all(source.join("agents")).map_err(|e| e.to_string())?;

    let platform_dir = project_dir.join(platform.project_dir);
    let mut count = 0;

    // Import root md → AGENTS.md
    let root_md = if platform.root_md_in_subdir {
        platform_dir.join(platform.root_md)
    } else {
        project_dir.join(platform.root_md)
    };
    if root_md.exists() {
        let dest = source.join("AGENTS.md");
        if !dest.exists() || fs::read_to_string(&dest).unwrap_or_default().trim().len() < 50 {
            fs::copy(&root_md, &dest).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    // Import rules (convert .mdc/.instructions.md → .md)
    if let Some(rules_subdir) = platform.rules_dir {
        let rules_dir = platform_dir.join(rules_subdir);
        if rules_dir.is_dir() {
            count += import_rules(&rules_dir, &source.join("rules"), platform.rules_ext)
                .map_err(|e| e.to_string())?;
        }
    }

    // Import skills/commands
    if let Some(skills_subdir) = platform.skills_dir {
        let skills_dir = platform_dir.join(skills_subdir);
        if skills_dir.is_dir() {
            count += import_md_files(&skills_dir, &source.join("skills"))
                .map_err(|e| e.to_string())?;
        }
    }

    // Import agents
    if let Some(agents_subdir) = platform.agents_dir {
        let agents_dir = platform_dir.join(agents_subdir);
        if agents_dir.is_dir() {
            count += import_md_files(&agents_dir, &source.join("agents"))
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(count)
}

/// Import rules and normalize extension to .md
fn import_rules(src: &Path, dest: &Path, src_ext: &str) -> std::io::Result<usize> {
    fs::create_dir_all(dest)?;
    let mut count = 0;
    for entry in fs::read_dir(src)?.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // Accept matching extension
        let matches = match src_ext {
            "md" => name.ends_with(".md"),
            "mdc" => name.ends_with(".mdc"),
            "instructions.md" => name.ends_with(".instructions.md"),
            _ => name.ends_with(&format!(".{src_ext}")),
        };
        if !matches { continue; }

        // Normalize to .md
        let stem = if src_ext == "instructions.md" {
            name.strip_suffix(".instructions.md").unwrap_or(&name)
        } else {
            path.file_stem().unwrap().to_str().unwrap_or(&name)
        };
        let dest_file = dest.join(format!("{stem}.md"));
        if !dest_file.exists() {
            fs::copy(&path, &dest_file)?;
            count += 1;
        }
    }
    Ok(count)
}

fn import_md_files(src: &Path, dest: &Path) -> std::io::Result<usize> {
    fs::create_dir_all(dest)?;
    let mut count = 0;
    for entry in fs::read_dir(src)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let dest_file = dest.join(path.file_name().unwrap());
            if !dest_file.exists() {
                fs::copy(&path, &dest_file)?;
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn init_source(project_dir: &Path) -> std::io::Result<PathBuf> {
    let source = project_dir.join(SOURCE_DIR);
    fs::create_dir_all(source.join("rules"))?;
    fs::create_dir_all(source.join("skills"))?;
    fs::create_dir_all(source.join("agents"))?;

    let agents_md = source.join("AGENTS.md");
    if !agents_md.exists() {
        fs::write(&agents_md, "# Agent Instructions\n\nShared instructions for all AI coding agents.\n")?;
    }
    Ok(source)
}

fn ensure_copy(src: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}
