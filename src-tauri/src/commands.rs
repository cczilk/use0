pub mod library;
pub mod playback;

use crate::database::Database;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub db: Mutex<Database>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackRow {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<i64>,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistRow {
    pub id: i64,
    pub name: String,
    pub track_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: i64,
    pub skipped: i64,
    pub failed: i64,
}

#[tauri::command]
pub fn get_all_tracks(state: tauri::State<'_, AppState>) -> Result<Vec<TrackRow>, String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.get_all_tracks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_track_info(state: tauri::State<'_, AppState>, track_id: i64) -> Result<Option<TrackRow>, String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.get_track_by_id(track_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_thumbnail(state: tauri::State<'_, AppState>, track_id: i64) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    let track = db.get_track_by_id(track_id).map_err(|e| e.to_string())?;
    Ok(track.and_then(|t| t.thumbnail_path))
}

#[tauri::command]
pub fn delete_track(state: tauri::State<'_, AppState>, track_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    if let Ok(Some(track)) = db.get_track_by_id(track_id) {
        if let Some(thumb) = track.thumbnail_path {
            let _ = std::fs::remove_file(thumb);
        }
    }
    db.delete_track(track_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_local_file(
    state: tauri::State<'_, AppState>,
    file_path: String,
    download_dir: String,
) -> Result<TrackRow, String> {
    {
        let db = state.db.lock().map_err(|_| "DB lock failed")?;
        if let Ok(Some(_)) = db.get_track_by_path(&file_path) {
            return Err(format!("Already imported: {file_path}"));
        }
    }
    let (title, artist, album, duration_secs) = extract_tags(&file_path);
    let id = {
        let db = state.db.lock().map_err(|_| "DB lock failed")?;
        db.insert_track(&title, artist.as_deref(), album.as_deref(), duration_secs, &file_path, None)
            .map_err(|e| e.to_string())?
    };
    let thumb = extract_embedded_art(&file_path, id, &download_dir);
    if let Some(ref thumb_path) = thumb {
        let db = state.db.lock().map_err(|_| "DB lock failed")?;
        let _ = db.update_track_thumbnail(id, Some(thumb_path.clone()));
    }
    Ok(TrackRow { id, title, artist, album, duration: duration_secs, file_path, thumbnail_path: thumb })
}

#[tauri::command]
pub async fn import_folder(
    state: tauri::State<'_, AppState>,
    folder_path: String,
    download_dir: String,
) -> Result<ImportResult, String> {
    use std::path::Path;
    let audio_exts = ["mp3","flac","wav","ogg","m4a","aac","opus","webm","mp4","wma","aiff","ape"];
    let mut files: Vec<String> = Vec::new();

    fn walk(dir: &Path, exts: &[&str], out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, exts, out);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if exts.contains(&ext.to_lowercase().as_str()) {
                    out.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    walk(Path::new(&folder_path), &audio_exts, &mut files);

    let existing_paths: std::collections::HashSet<String> = {
        let db = state.db.lock().map_err(|_| "DB lock failed")?;
        db.get_all_tracks().map_err(|e| e.to_string())?
            .into_iter().map(|t| t.file_path).collect()
    };

    let mut imported = 0i64;
    let mut skipped  = 0i64;
    let mut failed   = 0i64;

    for file_path in files {
        if existing_paths.contains(&file_path) { skipped += 1; continue; }
        let (title, artist, album, duration_secs) = extract_tags(&file_path);
        let result = {
            let db = state.db.lock().map_err(|_| "DB lock failed")?;
            db.insert_track(&title, artist.as_deref(), album.as_deref(), duration_secs, &file_path, None)
        };
        match result {
            Ok(id) => {
                let thumb = extract_embedded_art(&file_path, id, &download_dir);
                if let Some(ref tp) = thumb {
                    let db = state.db.lock().map_err(|_| "DB lock failed")?;
                    let _ = db.update_track_thumbnail(id, Some(tp.clone()));
                }
                imported += 1;
            }
            Err(_) => { failed += 1; }
        }
    }
    Ok(ImportResult { imported, skipped, failed })
}

#[tauri::command]
pub fn update_track_metadata(
    state: tauri::State<'_, AppState>,
    track_id: i64,
    title: String,
    artist: Option<String>,
    album: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.update_track_metadata(track_id, &title, artist.as_deref(), album.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_track_artwork(
    state: tauri::State<'_, AppState>,
    track_id: i64,
    image_path: String,
    download_dir: String,
) -> Result<String, String> {
    use std::path::Path;
    let src = Path::new(&image_path);
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let dest = Path::new(&download_dir).join(format!("thumb_{}.{}", track_id, ext));
    std::fs::copy(src, &dest).map_err(|e| format!("Copy artwork failed: {e}"))?;
    let dest_str = dest.to_string_lossy().to_string();
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.update_track_thumbnail(track_id, Some(dest_str.clone())).map_err(|e| e.to_string())?;
    Ok(dest_str)
}

#[tauri::command]
pub fn create_playlist(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    name: String,
) -> Result<PlaylistRow, String> {
    use tauri::Emitter;
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    let id = db.create_playlist(&name).map_err(|e| e.to_string())?;
    app.emit("playlists://updated", ()).ok();
    Ok(PlaylistRow { id, name, track_count: 0 })
}

#[tauri::command]
pub fn get_all_playlists(state: tauri::State<'_, AppState>) -> Result<Vec<PlaylistRow>, String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.get_all_playlists().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_track_to_playlist(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    playlist_id: i64,
    track_id: i64,
) -> Result<(), String> {
    use tauri::Emitter;
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.add_track_to_playlist(playlist_id, track_id).map_err(|e| e.to_string())?;
    app.emit("playlists://updated", ()).ok();
    Ok(())
}

#[tauri::command]
pub fn get_playlist_tracks(
    state: tauri::State<'_, AppState>,
    playlist_id: i64,
) -> Result<Vec<TrackRow>, String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.get_playlist_tracks(playlist_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rescan_artwork(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    let tracks = {
        let db = state.db.lock().map_err(|_| "DB lock failed")?;
        db.get_all_tracks().map_err(|e| e.to_string())?
    };
    let mut count = 0i64;
    for track in tracks {
        if track.thumbnail_path.is_some() { continue; }
        let dir = std::path::Path::new(&track.file_path)
            .parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        if let Some(thumb) = extract_embedded_art(&track.file_path, track.id, &dir) {
            let db = state.db.lock().map_err(|_| "DB lock failed")?;
            let _ = db.update_track_thumbnail(track.id, Some(thumb.clone()));
            count += 1;
        }
    }
    Ok(count)
}


#[tauri::command]
pub fn remove_from_playlist(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    playlist_id: i64,
    track_id: i64,
) -> Result<(), String> {
    use tauri::Emitter;
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    db.remove_from_playlist(playlist_id, track_id).map_err(|e| e.to_string())?;
    app.emit("playlists://updated", ()).ok();
    Ok(())
}

pub fn extract_tags(path: &str) -> (String, Option<String>, Option<String>, Option<i64>) {
    use lofty::prelude::*;
    use lofty::probe::Probe;
    let p = std::path::Path::new(path);
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Unknown".to_string());
    match Probe::open(p).and_then(|pr| pr.read()) {
        Ok(tagged) => {
            let dur = Some(tagged.properties().duration().as_secs() as i64);
            let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
            if let Some(tag) = tag {
                (tag.title().map(|t| t.to_string()).unwrap_or(stem), tag.artist().map(|a| a.to_string()), tag.album().map(|a| a.to_string()), dur)
            } else { (stem, None, None, dur) }
        }
        Err(_) => (stem, None, None, None),
    }
}

pub fn extract_embedded_art(path: &str, track_id: i64, download_dir: &str) -> Option<String> {
    use lofty::prelude::*;
    use lofty::probe::Probe;
    let tagged = Probe::open(path).ok()?.read().ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag())?;
    let pic = tag.pictures().first()?;
    let ext = match pic.mime_type() { Some(lofty::picture::MimeType::Png) => "png", _ => "jpg" };
    let dest = std::path::Path::new(download_dir).join(format!("thumb_{}.{}", track_id, ext));
    std::fs::write(&dest, pic.data()).ok()?;
    Some(dest.to_string_lossy().to_string())
}