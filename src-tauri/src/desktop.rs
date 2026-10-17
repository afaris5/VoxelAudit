use crate::{
    engine, source,
    types::{Progress, Report},
};
use base64::Engine;
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct Session {
    project: Option<PathBuf>,
    source: Option<source::Source>,
    report: Option<Report>,
}

#[derive(Default)]
struct AppState {
    session: Mutex<Session>,
    busy: AtomicBool,
    cancel: Arc<AtomicBool>,
}

struct BusyGuard<'a>(&'a AtomicBool);

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn emit_progress(app: &tauri::AppHandle, progress: Progress) {
    if let Err(error) = app.emit("scan-progress", progress) {
        eprintln!("Progress event could not be delivered: {error}");
    }
}

#[tauri::command]
async fn scan(app: tauri::AppHandle, mode: String) -> Result<Option<Report>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        if state.busy.swap(true, Ordering::SeqCst) {
            return Err("A scan is already running.".into());
        }
        let _guard = BusyGuard(&state.busy);
        state.cancel.store(false, Ordering::SeqCst);
        let path = match mode.as_str() {
            "directory" => app
                .dialog()
                .file()
                .set_title("Open resource pack or mod asset directory")
                .blocking_pick_folder()
                .map(|p| p.into_path())
                .transpose()
                .map_err(|_| "Select a local filesystem directory.")?,
            "archive" => app
                .dialog()
                .file()
                .add_filter("Resource pack", &["zip"])
                .blocking_pick_file()
                .map(|p| p.into_path())
                .transpose()
                .map_err(|_| "Select a local ZIP file.")?,
            "rescan" => state
                .session
                .lock()
                .map_err(|_| "Project session is unavailable.")?
                .project
                .clone(),
            _ => return Err("Unknown import mode.".into()),
        };
        let Some(path) = path else {
            return Ok(None);
        };
        let started = std::time::Instant::now();
        let source = source::load(&path, &state.cancel, &|p| emit_progress(&app, p))?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let mut report = engine::analyze(
            &source,
            name,
            if path.is_dir() {
                "directory"
            } else {
                "archive"
            }
            .into(),
            &state.cancel,
            &|p| emit_progress(&app, p),
        )?;
        report.duration_ms = started.elapsed().as_millis();
        if state.cancel.load(Ordering::SeqCst) {
            return Err("Scan cancelled.".into());
        }
        let mut session = state
            .session
            .lock()
            .map_err(|_| "Project session is unavailable.")?;
        session.project = Some(path);
        session.source = Some(source);
        session.report = Some(report.clone());
        Ok(Some(report))
    })
    .await
    .map_err(|_| "The scan worker stopped unexpectedly. Try reopening the project.".to_string())?
}

#[tauri::command]
fn cancel_scan(state: State<AppState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Preview {
    text: Option<String>,
    image: Option<String>,
}

#[tauri::command]
async fn preview(app: tauri::AppHandle, path: String) -> Result<Preview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let session = state.session.lock().map_err(|_| "Project session is unavailable.")?;
        let bytes = session.source.as_ref().and_then(|s| s.files.get(&path)).ok_or("File is not in the current project.")?;
        if path.ends_with(".png") {
            engine::inspect_png(bytes)?;
            Ok(Preview { text: None, image: Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes))) })
        } else if path.ends_with(".json") || path.ends_with(".mcmeta") {
            if bytes.len() > 1024 * 1024 { return Err("Text preview is limited to 1 MiB. Validation still covers the full file.".into()); }
            Ok(Preview { text: Some(String::from_utf8_lossy(bytes).into_owned()), image: None })
        } else { Err("Preview is available for PNG and JSON files. OGG files are indexed for reference checks.".into()) }
    }).await.map_err(|_| "Preview worker stopped unexpectedly.".to_string())?
}

#[tauri::command]
async fn export_report(app: tauri::AppHandle) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (report, project) = {
            let session = state
                .session
                .lock()
                .map_err(|_| "Project session is unavailable.")?;
            (
                session.report.clone().ok_or("Open a project first.")?,
                session.project.clone().ok_or("Open a project first.")?,
            )
        };
        let Some(target) = app
            .dialog()
            .file()
            .set_file_name("voxelaudit-report.json")
            .add_filter("JSON report", &["json"])
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let target = target
            .into_path()
            .map_err(|_| "Select a local output file.")?;
        let parent = target
            .parent()
            .ok_or("Invalid report destination.")?
            .canonicalize()
            .map_err(|_| "Report destination is not accessible.")?;
        let project = project
            .canonicalize()
            .map_err(|_| "Original project is no longer accessible.")?;
        if parent.starts_with(&project) || target == project {
            return Err(
                "Save the report outside the inspected project to protect source files.".into(),
            );
        }
        if target.exists() {
            return Err(
                "Choose a new report filename. Existing files are never overwritten.".into(),
            );
        }
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|e| format!("Cannot create report: {}.", e.kind()))?;
        serde_json::to_writer_pretty(file, &report)
            .map_err(|_| "Report could not be written completely. Check available disk space.")?;
        Ok(true)
    })
    .await
    .map_err(|_| "Export worker stopped unexpectedly.".to_string())?
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan,
            cancel_scan,
            preview,
            export_report
        ])
        .run(tauri::generate_context!())
        .expect("VoxelAudit could not start");
}
