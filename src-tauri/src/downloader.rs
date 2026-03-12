use serde::Serialize;
use std::process::Command;
use std::path::Path;
use tauri::Emitter;

#[derive(Debug, Serialize, Clone)]
pub struct DownloadResult {
    pub title: String,
    pub file_path: String,
}

#[tauri::command]
pub async fn download_from_youtube(
    state: tauri::State<'_, crate::commands::AppState>,
    app: tauri::AppHandle,
    url: String,
    download_dir: String,
) -> Result<DownloadResult, String> {

    let output_template = format!("{}/%(title)s.%(ext)s", download_dir);

    let output = Command::new("yt-dlp")
        .args([
            "-x",
            "--audio-format",  "mp3",
            "--audio-quality", "0",
            "--embed-thumbnail",
            "--add-metadata",
            "-o",              &output_template,
            "--print",         "after_move:filepath", 
            &url,
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "yt-dlp not found. Install with: brew install yt-dlp".to_string()
            } else {
                format!("Failed to run yt-dlp: {e}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let file_path = stdout.trim().to_string();

    if file_path.is_empty() || !Path::new(&file_path).exists() {
        let newest = find_newest_audio_file(&download_dir)
            .ok_or("Download succeeded but could not find output file")?;
        return insert_and_return(&state, &app, &newest, &download_dir).await;
    }

    insert_and_return(&state, &app, &file_path, &download_dir).await
}

async fn insert_and_return(
    state: &tauri::State<'_, crate::commands::AppState>,
    app: &tauri::AppHandle,
    file_path: &str,
    download_dir: &str,
) -> Result<DownloadResult, String> {
    use crate::commands::{extract_tags, extract_embedded_art};

    let (title, artist, album, duration_secs) = extract_tags(file_path);

    let id = {
        let db = state.db.lock().map_err(|_| "DB lock failed")?;
        db.insert_track(&title, artist.as_deref(), album.as_deref(), duration_secs, file_path, None)
            .map_err(|e| e.to_string())?
    };

    if let Some(thumb) = extract_embedded_art(file_path, id, download_dir) {
        let db = state.db.lock().map_err(|_| "DB lock failed")?;
        let _ = db.update_track_thumbnail(id, Some(thumb));
    }

    app.emit("library://refreshed", ()).ok();

    Ok(DownloadResult {
        title,
        file_path: file_path.to_string(),
    })
}

fn find_newest_audio_file(dir: &str) -> Option<String> {
    let extensions = ["mp3", "flac", "ogg", "wav", "m4a", "opus"];
    let mut best: Option<(std::time::SystemTime, String)> = None;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext.to_lowercase().as_str()) {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if best.as_ref().map_or(true, |(t, _)| modified > *t) {
                                best = Some((modified, path.to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}