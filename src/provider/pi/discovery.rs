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
    let mut file = classify_head(path, &head(path)?, crate::provider::modified(path))?;
    let mut current = path.to_path_buf();
    let mut seen = std::collections::HashSet::new();
    while seen.insert(current.clone()) {
        let v = super::wire::parse_line(&head(&current)?)?;
        let Some(parent) = parent_path(&current, &v) else {
            return Some(file);
        };
        let meta = super::wire::parse_line(&head(&parent)?)?;
        if !super::wire::is_header(&meta) {
            return None;
        }
        if current == path {
            file.role = FileRole::Agent {
                parent: meta["id"].as_str()?.into(),
            };
        }
        file.session = meta["id"].as_str()?.into();
        current = parent;
    }
    None
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

pub fn related_paths(file: &SessionFile) -> Vec<PathBuf> {
    let mut out = file.path.parent().map(|p| paths(p, 0)).unwrap_or_default();
    let mut current = file.path.clone();
    let mut seen = std::collections::HashSet::new();
    while seen.insert(current.clone()) {
        let Some(v) = head(&current).and_then(|s| super::wire::parse_line(&s)) else {
            break;
        };
        let Some(parent) = parent_path(&current, &v) else {
            break;
        };
        out.extend(parent.parent().map(|p| paths(p, 0)).unwrap_or_default());
        out.push(parent.clone());
        current = parent;
    }
    out.sort();
    out.dedup();
    out
}

fn parent_path(path: &Path, v: &serde_json::Value) -> Option<PathBuf> {
    let parent = PathBuf::from(v["parentSession"].as_str()?);
    if parent.is_absolute() {
        Some(parent)
    } else {
        Some(path.parent()?.join(parent))
    }
}

pub fn stream_for(file: &SessionFile) -> super::Stream {
    let parent = match &file.role {
        FileRole::Agent { parent } => Some(if parent == &file.session {
            "main".into()
        } else {
            parent.clone()
        }),
        _ => None,
    };
    super::Stream::for_file(parent)
}

pub fn project_key(cwd: &Path) -> String {
    cwd.to_string_lossy().into_owned()
}
