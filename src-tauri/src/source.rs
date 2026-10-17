use crate::types::{Finding, Progress};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Component, Path},
    sync::atomic::{AtomicBool, Ordering},
};
use walkdir::WalkDir;

pub const MAX_FILE: u64 = 16 * 1024 * 1024;
pub const MAX_TOTAL: u64 = 512 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 50_000;

pub struct Source {
    pub files: BTreeMap<String, Vec<u8>>,
    pub findings: Vec<Finding>,
    pub complete: bool,
}

pub fn safe_archive_path(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains(['\\', ':', '\0'])
        && name
            .split('/')
            .all(|p| p != ".." && p != "." && !p.is_empty())
        && Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

fn relevant(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    [".json", ".mcmeta", ".png", ".ogg"]
        .iter()
        .any(|e| lower.ends_with(e))
}

impl Source {
    fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            findings: vec![],
            complete: true,
        }
    }

    fn problem(&mut self, path: &str, message: impl Into<String>) {
        self.complete = false;
        self.findings.push(Finding::new(
            "error",
            "Import",
            "import.read",
            path,
            message,
        ));
    }

    fn insert(&mut self, path: String, size: u64, reader: impl Read, total: &mut u64) {
        if size > MAX_FILE || total.saturating_add(size) > MAX_TOTAL {
            self.problem(&path, "Safety limit reached: maximum 16 MiB per file and 512 MiB per project. This file was skipped.");
            return;
        }
        let mut bytes = Vec::new();
        match reader.take(MAX_FILE + 1).read_to_end(&mut bytes) {
            Ok(_)
                if bytes.len() as u64 <= MAX_FILE
                    && total.saturating_add(bytes.len() as u64) <= MAX_TOTAL =>
            {
                *total += bytes.len() as u64;
                if self.files.insert(path.clone(), bytes).is_some() {
                    self.problem(
                        &path,
                        "Duplicate archive entry; the last readable definition is inspected.",
                    );
                }
            }
            Ok(_) => self.problem(&path, "Expanded file exceeds the safety limit."),
            Err(e) => self.problem(&path, format!("Cannot read file: {}.", e.kind())),
        }
    }
}

pub fn load(
    path: &Path,
    cancel: &AtomicBool,
    progress: &impl Fn(Progress),
) -> Result<Source, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| {
        format!(
            "Cannot open selected project: {}. Check that it exists and is readable.",
            e.kind()
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err("Select a real directory or ZIP, not a symbolic link.".into());
    }
    let mut result = Source::new();
    let mut total = 0;
    if metadata.is_dir() {
        for (index, entry) in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .enumerate()
        {
            if cancel.load(Ordering::Relaxed) {
                return Err("Scan cancelled.".into());
            }
            if index >= MAX_ENTRIES {
                result.problem(
                    "",
                    "Project exceeds the 50,000 entry safety limit. Scan is incomplete.",
                );
                break;
            }
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    result.problem(
                        "",
                        format!(
                            "A directory entry is inaccessible: {}.",
                            e.io_error()
                                .map(|v| v.kind().to_string())
                                .unwrap_or_else(|| "invalid entry".into())
                        ),
                    );
                    continue;
                }
            };
            let relative = entry
                .path()
                .strip_prefix(path)
                .map_err(|_| "Invalid relative path.")?
                .to_string_lossy()
                .replace('\\', "/");
            if entry.file_type().is_symlink() {
                result.problem(
                    &relative,
                    "Symbolic link skipped to keep inspection inside the selected directory.",
                );
                continue;
            }
            if !entry.file_type().is_file() || !relevant(&relative) {
                continue;
            }
            match File::open(entry.path()).and_then(|f| f.metadata().map(|m| (f, m.len()))) {
                Ok((file, size)) => result.insert(relative, size, file, &mut total),
                Err(e) => result.problem(&relative, format!("Cannot open file: {}.", e.kind())),
            }
            progress(Progress {
                phase: "Reading files".into(),
                processed: result.files.len(),
                total: None,
            });
        }
    } else {
        let file = File::open(path).map_err(|_| "Cannot read the selected archive.")?;
        let mut zip = zip::ZipArchive::new(file).map_err(|_| {
            "Invalid or corrupt ZIP archive. Select an unencrypted resource pack ZIP."
        })?;
        if zip.len() > MAX_ENTRIES {
            return Err("Archive exceeds the 50,000 entry safety limit.".into());
        }
        for index in 0..zip.len() {
            if cancel.load(Ordering::Relaxed) {
                return Err("Scan cancelled.".into());
            }
            let entry = match zip.by_index(index) {
                Ok(v) => v,
                Err(_) => {
                    result.problem(
                        "",
                        "Cannot read archive entry; it may be corrupt or encrypted.",
                    );
                    continue;
                }
            };
            let name = entry.name().trim_end_matches('/').to_string();
            if !safe_archive_path(&name) {
                result.problem(&name, "Unsafe archive path rejected. Absolute paths, traversal and Windows path separators are forbidden.");
                continue;
            }
            if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
                result.problem(&name, "Archive symbolic link rejected.");
                continue;
            }
            if entry.is_dir() || !relevant(&name) {
                continue;
            }
            result.insert(name, entry.size(), entry, &mut total);
            progress(Progress {
                phase: "Reading archive".into(),
                processed: index + 1,
                total: Some(zip.len()),
            });
        }
    }
    Ok(result)
}
