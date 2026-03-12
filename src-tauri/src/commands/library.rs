use crate::commands::{AppState, TrackRow};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Title,
    Artist,
    Album,
    Duration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[tauri::command]
pub async fn search_tracks(
    state: tauri::State<'_, AppState>,
    query: Option<String>,
    sort_by: Option<SortField>,
    sort_order: Option<SortOrder>,
) -> Result<Vec<TrackRow>, String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    let all = db.get_all_tracks().map_err(|e| e.to_string())?;

    let q = query.as_deref().unwrap_or("").to_lowercase();
    let q = q.trim();

    let mut filtered: Vec<TrackRow> = if q.is_empty() {
        all
    } else {
        all.into_iter()
            .filter(|t| {
                t.title.to_lowercase().contains(q)
                    || t.artist.as_deref().unwrap_or("").to_lowercase().contains(q)
                    || t.album.as_deref().unwrap_or("").to_lowercase().contains(q)
            })
            .collect()
    };

    let asc = matches!(sort_order, Some(SortOrder::Asc) | None);

    match sort_by.unwrap_or(SortField::Title) {
        SortField::Title => filtered.sort_by(|a, b| {
            let c = a.title.to_lowercase().cmp(&b.title.to_lowercase());
            if asc { c } else { c.reverse() }
        }),
        SortField::Artist => filtered.sort_by(|a, b| {
            let c = a.artist.as_deref().unwrap_or("").to_lowercase()
                     .cmp(&b.artist.as_deref().unwrap_or("").to_lowercase());
            if asc { c } else { c.reverse() }
        }),
        SortField::Album => filtered.sort_by(|a, b| {
            let c = a.album.as_deref().unwrap_or("").to_lowercase()
                     .cmp(&b.album.as_deref().unwrap_or("").to_lowercase());
            if asc { c } else { c.reverse() }
        }),
        SortField::Duration => filtered.sort_by(|a, b| {
            let c = a.duration.unwrap_or(0).cmp(&b.duration.unwrap_or(0));
            if asc { c } else { c.reverse() }
        }),
    }

    Ok(filtered)
}

#[tauri::command]
pub fn format_duration(seconds: i64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[tauri::command]
pub async fn manual_update_artwork(
    state: tauri::State<'_, AppState>, 
    track_id: i64,
    thumbnail_path: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "DB lock failed")?;
    
    db.update_track_thumbnail(track_id, Some(thumbnail_path))
        .map_err(|e| format!("Failed to update artwork in DB: {}", e))?;

    Ok(())
}