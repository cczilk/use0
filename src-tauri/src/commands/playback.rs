/// commands/playback.rs
///
/// Tauri command layer for the Player engine.
///
/// BEFORE: useAudio.js managed AudioContext, BufferSource nodes, gain nodes,
/// EQ nodes, a preload buffer, shuffle history, and requestAnimationFrame timers
/// all inside a React hook. Any re-render or React strict-mode double-invoke
/// could reset audio state.
///
/// AFTER: Every playback action is a thin Tauri command that delegates to
/// player::Player which lives in managed state. The frontend is event-driven:
/// it listens to `player://state-changed` and `player://track-changed` events
/// instead of polling via rAF.

use crate::{
    commands::AppState,
    player::{Player, TrackChangedPayload, QueueUpdatedPayload},
};
use tauri::{AppHandle, Emitter, State};

// ── Playback ──────────────────────────────────────────────────────────────────

/// Start playback for a track ID.
/// Replaces: `loadAndPlay(trackId)` in useAudio.js
#[tauri::command]
pub async fn player_play_track(
    state: State<'_, AppState>,
    player: State<'_, Player>,
    app: AppHandle,
    track_id: i64,
) -> Result<(), String> {
    // 1. Fetch file_path and metadata from DB
    let (file_path, info) = {
        let db = state.db.lock().map_err(|_| "DB lock")?;
        let info = db.get_track_by_id(track_id)
            .map_err(|e| e.to_string())?
            .ok_or("Track not found")?;
        (info.file_path.clone(), info)
    };

    // 2. Tell the player engine to play it
    let duration = player.play_track(&file_path, track_id, &app)?;

    // 3. Emit track-changed so NowPlayingWithVisualizer re-renders
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

/// Replaces: `pause()` in useAudio.js
#[tauri::command]
pub fn player_pause(player: State<'_, Player>) -> Result<(), String> {
    player.pause()
}

/// Replaces: `play()` (resume) in useAudio.js
#[tauri::command]
pub fn player_resume(player: State<'_, Player>) -> Result<(), String> {
    player.resume()
}

/// Replaces: `stop()` in useAudio.js
#[tauri::command]
pub fn player_stop(player: State<'_, Player>) -> Result<(), String> {
    player.stop()
}

/// Replaces: `seek(time)` in useAudio.js
/// Previously required stopping/restarting a Web Audio BufferSourceNode
/// and tracking pauseTimeRef manually. Now a single Rust call.
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

/// Replaces: `setVolume(vol)` in useAudio.js
/// `vol` is 0–100 (same as frontend convention), normalised to 0.0–1.0 internally
#[tauri::command]
pub fn player_set_volume(player: State<'_, Player>, volume: u32) -> Result<(), String> {
    player.set_volume(volume as f32 / 100.0)
}

// ── Queue management ──────────────────────────────────────────────────────────

/// Replaces: `setPlayQueue(trackIds, startIndex)` in useAudio.js
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

/// Replaces: `playNext()` in useAudio.js
/// The shuffle / history logic now lives in Player::next_track_id()
#[tauri::command]
pub async fn player_next(
    state: State<'_, AppState>,
    player: State<'_, Player>,
    app: AppHandle,
) -> Result<(), String> {
    if let Some(next_id) = player.next_track_id()? {
        player_play_track(state, player, app, next_id).await
    } else {
        Ok(()) // end of queue — nothing to do
    }
}

/// Replaces: `playPrevious()` in useAudio.js
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

// ── Config ────────────────────────────────────────────────────────────────────

/// Replaces: `setShuffle(bool)` in useAudio.js
#[tauri::command]
pub fn player_set_shuffle(player: State<'_, Player>, enabled: bool) -> Result<(), String> {
    player.set_shuffle(enabled)
}

/// Replaces: `setAutoplay(bool)` in useAudio.js
#[tauri::command]
pub fn player_set_autoplay(player: State<'_, Player>, enabled: bool) -> Result<(), String> {
    player.set_autoplay(enabled)
}

/// Snapshot of playback state — used for initial load only.
/// Ongoing state is pushed via `player://state-changed` events.
#[derive(serde::Serialize)]
pub struct PlaybackSnapshot {
    pub is_playing: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: u32, // 0–100
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