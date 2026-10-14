use crate::provider::pi::discovery::{head, paths};
use crate::provider::{FileRole, Provider, ReadMode, Scope, SessionFile};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn all_paths(_scope: &Scope) -> Vec<PathBuf> {
    std::env::var_os("HOME")
        .map(|h| paths(&PathBuf::from(h).join(".rusty"), 2))
        .unwrap_or_default()
}

fn root_dir(path: &Path) -> Option<&Path> {
    let mut dir = path.parent()?;
    while dir.file_name()?.to_str()?.starts_with("sub-") {
        dir = dir.parent()?;
    }
    Some(dir)
}

pub fn session_file(path: &Path) -> Option<SessionFile> {
    let mut file = classify_head(path, &head(path)?, crate::provider::modified(path))?;
    file.project_key = std::fs::read_to_string(root_dir(path)?.join("cwd"))
        .unwrap_or_default()
        .trim()
        .into();
    Some(file)
}

pub fn classify_head(path: &Path, head: &str, modified: SystemTime) -> Option<SessionFile> {
    let v = super::wire::parse_line(head.lines().find(|l| !l.trim().is_empty())?)?;
    if !super::wire::is_header(&v) {
        return None;
    }
    let session = root_dir(path)?.file_name()?.to_str()?.to_owned();
    let child = path.file_stem()?.to_str()?.starts_with("sub-");
    Some(SessionFile {
        provider: Provider::Rusty,
        path: path.into(),
        session: session.clone(),
        role: if child {
            FileRole::Agent { parent: session }
        } else {
            FileRole::Root
        },
        read: ReadMode::Tail,
        project_key: v["data"]["cwd"].as_str().unwrap_or_default().into(),
        modified,
    })
}

pub fn related_paths(file: &SessionFile) -> Vec<PathBuf> {
    root_dir(&file.path)
        .map(|p| paths(p, 2))
        .unwrap_or_default()
}

pub fn project_key(cwd: &Path) -> String {
    cwd.to_string_lossy().into_owned()
}

pub fn stream_for(file: &SessionFile) -> super::Stream {
    let owner = if file.role == FileRole::Root {
        "main".into()
    } else {
        file.path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    };
    super::Stream::new(owner)
}
