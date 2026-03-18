use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;

/// Recursively collect all .md files under dir, returning paths relative to dir
fn collect_files(dir: &Path, prefix: &str) -> Vec<String> {
    let mut files = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let rel = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if path.is_dir() {
            files.extend(collect_files(&path, &rel));
        } else if name.ends_with(".md") {
            files.push(rel);
        }
    }
    files.sort();
    files
}

fn send_200(stream: &mut impl Write, body: &[u8]) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.0 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)
}

fn send_404(stream: &mut impl Write) -> std::io::Result<()> {
    stream.write_all(b"HTTP/1.0 404 Not Found\r\n\r\n")
}

fn handle_request(stream: &mut impl Write, agents_dir: &Path, request_path: &str) {
    println!("GET {request_path}");

    if request_path == "/manifest" {
        let files = collect_files(agents_dir, "");
        let body = files.join("\n");
        let _ = send_200(stream, body.as_bytes());
        return;
    }

    if let Some(rel) = request_path.strip_prefix("/file/") {
        // Reject path traversal
        if rel.contains("..") {
            let _ = send_404(stream);
            return;
        }
        let file_path = agents_dir.join(rel);
        match fs::read(&file_path) {
            Ok(contents) => {
                let _ = send_200(stream, &contents);
            }
            Err(_) => {
                let _ = send_404(stream);
            }
        }
        return;
    }

    let _ = send_404(stream);
}

/// Start HTTP server, blocks until Ctrl+C
pub fn serve(project_dir: &Path, bind: &str, port: u16) -> std::io::Result<()> {
    let agents_dir = project_dir.join(".agents");
    let addr = format!("{bind}:{port}");
    let listener = TcpListener::bind(&addr)?;
    println!("Serving .agents/ on http://{addr}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let reader = BufReader::new(match stream.try_clone() {
            Ok(r) => r,
            Err(_) => continue,
        });
        let first_line = match reader.lines().next() {
            Some(Ok(line)) => line,
            _ => continue,
        };
        // Parse: GET /path HTTP/1.0
        let mut parts = first_line.splitn(3, ' ');
        let _method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("").to_string();
        handle_request(&mut stream, &agents_dir, &path);
    }

    Ok(())
}
