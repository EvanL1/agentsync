use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

// ── Config types ─────────────────────────────────────────────────────────────

pub struct RemoteHost {
    pub alias: String,
    pub host: String,  // user@hostname
    pub port: u16,     // default 22
    pub path: String,  // remote project path, default "."
}

// ── TOML parser / writer ──────────────────────────────────────────────────────

pub fn load_remotes(project_dir: &Path) -> Vec<RemoteHost> {
    let content = match fs::read_to_string(project_dir.join(".agents/remotes.toml")) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut remotes: Vec<RemoteHost> = Vec::new();
    let mut cur: Option<RemoteHost> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if line == "[[remote]]" {
            if let Some(r) = cur.take() { remotes.push(r); }
            cur = Some(RemoteHost { alias: String::new(), host: String::new(), port: 22, path: ".".into() });
            continue;
        }
        if let (Some(r), Some((k, v))) = (cur.as_mut(), parse_kv(line)) {
            match k {
                "alias" => r.alias = v.into(),
                "host"  => r.host  = v.into(),
                "port"  => r.port  = v.parse().unwrap_or(22),
                "path"  => r.path  = v.into(),
                _ => {}
            }
        }
    }
    if let Some(r) = cur { remotes.push(r); }
    remotes
}

fn parse_kv(line: &str) -> Option<(&str, &str)> {
    let (k, rest) = line.split_once('=')?;
    Some((k.trim(), rest.trim().trim_matches('"')))
}

pub fn save_remotes(project_dir: &Path, remotes: &[RemoteHost]) -> std::io::Result<()> {
    let mut out = String::new();
    for r in remotes {
        out.push_str(&format!(
            "[[remote]]\nalias = \"{}\"\nhost = \"{}\"\nport = {}\npath = \"{}\"\n\n",
            r.alias, r.host, r.port, r.path
        ));
    }
    fs::write(project_dir.join(".agents/remotes.toml"), out)
}

pub fn add_remote(project_dir: &Path, alias: &str, host: &str) -> std::io::Result<()> {
    let mut remotes = load_remotes(project_dir);
    let (resolved_host, port) = split_host_port(host);
    remotes.push(RemoteHost { alias: alias.into(), host: resolved_host, port, path: ".".into() });
    save_remotes(project_dir, &remotes)
}

pub fn remove_remote(project_dir: &Path, alias: &str) -> std::io::Result<bool> {
    let mut remotes = load_remotes(project_dir);
    let before = remotes.len();
    remotes.retain(|r| r.alias != alias);
    if remotes.len() < before { save_remotes(project_dir, &remotes)?; Ok(true) } else { Ok(false) }
}

fn split_host_port(host: &str) -> (String, u16) {
    if let Some(i) = host.rfind(':') {
        if let Ok(p) = host[i + 1..].parse::<u16>() {
            return (host[..i].into(), p);
        }
    }
    (host.into(), 22)
}

// ── SSH push ──────────────────────────────────────────────────────────────────

pub fn push_to_remote(project_dir: &Path, remote: &RemoteHost, dry_run: bool) -> Result<usize, String> {
    let agents_src = project_dir.join(".agents");
    if !agents_src.exists() { return Err("No .agents/ directory found".into()); }

    let port = remote.port.to_string();
    let src  = format!("{}/", agents_src.display());
    let dst  = format!("{}:{}/.agents/", remote.host, remote.path);

    let has_rsync = Command::new("rsync").arg("--version").status()
        .map(|s| s.success()).unwrap_or(false);

    if dry_run {
        println!("[dry-run] Would push .agents/ to {} via {}", remote.alias, if has_rsync { "rsync" } else { "scp" });
        return Ok(0);
    }

    if has_rsync {
        let ssh_opt = format!("ssh -p {port} -o ConnectTimeout=10 -o BatchMode=yes");
        let out = Command::new("rsync")
            .args(["-az", "--delete", "-e", &ssh_opt, &src, &dst])
            .output().map_err(|e| format!("rsync: {e}"))?;
        if !out.status.success() {
            return Err(format!("rsync failed: {}", String::from_utf8_lossy(&out.stderr)));
        }
        let remote_cmd = format!(
            "cd {path} && aisync sync 2>/dev/null || echo 'aisync not found on remote, .agents/ synced but not applied'",
            path = remote.path
        );
        let _ = Command::new("ssh")
            .args(["-p", &port, "-o", "ConnectTimeout=10", "-o", "BatchMode=yes", &remote.host, &remote_cmd])
            .status();
    } else {
        let out = Command::new("scp")
            .args(["-P", &port, "-r", &src, &dst])
            .output().map_err(|e| format!("scp: {e}"))?;
        if !out.status.success() {
            return Err(format!("scp failed: {}", String::from_utf8_lossy(&out.stderr)));
        }
    }

    Ok(count_files_in(&agents_src))
}

fn count_files_in(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else { return 0 };
    entries.flatten().map(|e| {
        let p = e.path();
        if p.is_dir() { count_files_in(&p) } else { 1 }
    }).sum()
}

// ── HTTP pull client ──────────────────────────────────────────────────────────

pub fn pull_from(project_dir: &Path, url: &str, dry_run: bool) -> Result<usize, String> {
    let (host, port, _prefix) = parse_url(url)?;
    let addr = format!("{host}:{port}");

    let manifest_bytes = http_get(&addr, &host, "/manifest")?;
    let manifest = String::from_utf8_lossy(&manifest_bytes);
    let files: Vec<&str> = manifest.lines().filter(|l| !l.is_empty()).collect();

    if dry_run {
        println!("[dry-run] Would pull {} file(s) from {url}", files.len());
        return Ok(files.len());
    }

    let agents_dir = project_dir.join(".agents");
    let mut count = 0;
    for file_path in &files {
        let body = http_get(&addr, &host, &format!("/file/{file_path}"))?;
        let dest = agents_dir.join(file_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
        }
        fs::write(&dest, &body).map_err(|e| format!("write {file_path}: {e}"))?;
        count += 1;
    }
    Ok(count)
}

fn http_get(addr: &str, host: &str, path: &str) -> Result<Vec<u8>, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| format!("connect {addr}: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\n\r\n");
    stream.write_all(req.as_bytes()).map_err(|e| format!("write: {e}"))?;
    read_http_response(&mut stream)
}

fn parse_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url.strip_prefix("http://")
        .ok_or_else(|| format!("URL must start with http://: {url}"))?;
    let (hostport, path_prefix) = match rest.find('/') {
        Some(i) => (&rest[..i], rest[i..].into()),
        None     => (rest, "/".into()),
    };
    let (host, port) = match hostport.rfind(':') {
        Some(i) => (hostport[..i].into(), hostport[i+1..].parse::<u16>()
            .map_err(|_| format!("invalid port in: {url}"))?),
        None     => (hostport.into(), 9753u16),
    };
    Ok((host, port, path_prefix))
}

fn read_http_response(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = stream.read(&mut tmp).map_err(|e| format!("read: {e}"))?;
        if n == 0 { break; }
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") { break; }
    }
    let sep = buf.windows(4).position(|w| w == b"\r\n\r\n")
        .ok_or("malformed HTTP: no header separator")?;
    let headers = String::from_utf8_lossy(&buf[..sep]);
    let mut body = buf[sep + 4..].to_vec();
    let content_length: Option<usize> = headers.lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
        .and_then(|l| l.split_once(':'))
        .and_then(|(_, v)| v.trim().parse().ok());
    if let Some(cl) = content_length {
        while body.len() < cl {
            let n = stream.read(&mut tmp).map_err(|e| format!("read: {e}"))?;
            if n == 0 { break; }
            body.extend_from_slice(&tmp[..n]);
        }
        body.truncate(cl);
    } else {
        loop {
            let n = stream.read(&mut tmp).map_err(|e| format!("read: {e}"))?;
            if n == 0 { break; }
            body.extend_from_slice(&tmp[..n]);
        }
    }
    Ok(body)
}
