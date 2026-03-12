use crate::{
    commands::AppState,
    player::{Player, TrackChangedPayload, QueueUpdatedPayload},
};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn player_play_track(
    state: State<'_, AppState>,
    player: State<'_, Player>,
    app: AppHandle,
    track_id: i64,
) -> Result<(), String> {
    let (file_path, info) = {
        let db = state.db.lock().map_err(|_| "DB lock")?;
        let info = db.get_track_by_id(track_id)
            .map_err(|e| e.to_string())?
            .ok_or("Track not found")?;
        (info.file_path.clone(), info)
    };

    let duration = player.play_track(&file_path, track_id, &app)?;

    app.emit("player://track-changed", TrackChangedPayload {
        track_id,
        title: info.title.clone(),
        artist: info.artist.clone().unwrap_or_default(),
        album: info.album.clone(),
        duration_secs: duration,
        file_path: file_path.clone(),
        thumbnail_path: info.thumbnail_path.clone(),
    }).ok();

    Ok(())
}

#[tauri::command]
pub fn player_pause(player: State<'_, Player>) -> Result<(), String> {
    player.pause()
}

#[tauri::command]
pub fn player_resume(player: State<'_, Player>) -> Result<(), String> {
    player.resume()
}

#[tauri::command]
pub fn player_stop(player: State<'_, Player>) -> Result<(), String> {
    player.stop()
}

#[tauri::command]
pub async fn player_seek(
    state: State<'_, AppState>,
    player: State<'_, Player>,
    track_id: i64,
    position_secs: f64,
) -> Result<(), String> {
    let file_path = {
        let db = state.db.lock().map_err(|_| "DB lock")?;
        db.get_track_by_id(track_id)
            .map_err(|e| e.to_string())?
            .ok_or("Track not found")?
            .file_path
    };
    player.seek(&file_path, position_secs)
}

#[tauri::command]
pub fn player_set_volume(player: State<'_, Player>, volume: u32) -> Result<(), String> {
    player.set_volume(volume as f32 / 100.0)
}

#[tauri::command]
pub fn player_set_queue(
    player: State<'_, Player>,
    app: AppHandle,
    track_ids: Vec<i64>,
    start_index: usize,
) -> Result<(), String> {
    player.set_queue(track_ids.clone(), start_index)?;
    app.emit("player://queue-updated", QueueUpdatedPayload {
        queue: track_ids,
        current_index: start_index,
    }).ok();
    Ok(())
}

#[tauri::command]
pub async fn player_next(
    state: State<'_, AppState>,
    player: State<'_, Player>,
    app: AppHandle,
) -> Result<(), String> {
    if let Some(next_id) = player.next_track_id()? {
        player_play_track(state, player, app, next_id).await
    } else {
        Ok(()) 
    }
}

#[tauri::command]
pub async fn player_previous(
    state: State<'_, AppState>,
    player: State<'_, Player>,
    app: AppHandle,
) -> Result<(), String> {
    if let Some(prev_id) = player.prev_track_id()? {
        player_play_track(state, player, app, prev_id).await
    } else {
        Ok(())
    }
}


#[tauri::command]
pub fn player_set_shuffle(player: State<'_, Player>, enabled: bool) -> Result<(), String> {
    player.set_shuffle(enabled)
}

#[tauri::command]
pub fn player_set_autoplay(player: State<'_, Player>, enabled: bool) -> Result<(), String> {
    player.set_autoplay(enabled)
}

#[derive(serde::Serialize)]
pub struct PlaybackSnapshot {
    pub is_playing: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: u32, 
    pub shuffle: bool,
    pub autoplay: bool,
}

#[tauri::command]
pub fn player_get_state(player: State<'_, Player>) -> Result<PlaybackSnapshot, String> {
    let config = player.config()?;
    Ok(PlaybackSnapshot {
        is_playing: player.is_playing(),
        position_secs: player.position(),
        duration_secs: player.duration(),
        volume: (config.volume * 100.0) as u32,
        shuffle: config.shuffle,
        autoplay: config.autoplay,
    })
}