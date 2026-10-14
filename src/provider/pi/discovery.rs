use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::provider::{FileRole, Provider, ReadMode, Scope, SessionFile};

pub fn paths(dir: &Path, depth: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "jsonl") {
                out.push(path);
            } else if depth > 0 && entry.file_type().is_ok_and(|t| t.is_dir()) {
                out.extend(paths(&path, depth - 1));
            }
        }
    }
    out.sort();
    out
}

pub fn all_paths(_scope: &Scope) -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for root in [
        ".pi/vanilla-agent/sessions",
        ".pi/agent/sessions",
        ".pi/pii/sessions",
        ".prime/agent/sessions",
    ] {
        out.extend(paths(&home.join(root), 1));
    }
    out.sort();
    out.dedup();
    out
}

pub fn head(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};
    let mut line = String::new();
    BufReader::new(std::fs::File::open(path).ok()?)
        .read_line(&mut line)
        .ok()?;
    Some(line)
}

pub fn session_file(path: &Path) -> Option<SessionFile> {
    classify_head(path, &head(path)?, crate::provider::modified(path))
}

pub fn classify_head(path: &Path, head: &str, modified: SystemTime) -> Option<SessionFile> {
    let v = super::wire::parse_line(head.lines().find(|l| !l.trim().is_empty())?)?;
    if !super::wire::is_header(&v) {
        return None;
    }
    Some(SessionFile {
        provider: Provider::Pi,
        path: path.to_path_buf(),
        session: v["id"].as_str()?.into(),
        role: FileRole::Root,
        read: ReadMode::Tail,
        project_key: v["cwd"].as_str().unwrap_or_default().into(),
        modified,
    })
}

pub fn related_paths(_file: &SessionFile) -> Vec<PathBuf> {
    Vec::new()
}

pub fn project_key(cwd: &Path) -> String {
    cwd.to_string_lossy().into_owned()
}
