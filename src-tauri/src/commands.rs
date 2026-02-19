use tauri::State;
use crate::player::PlayerState;
use crate::player::engine;
use crate::library::scanner::{self, Song};
use tauri::Manager;

#[tauri::command]
pub fn play_song(path: String, state: State<'_, PlayerState>) -> Result<(), String> {
    engine::play_file(&path, &state.sink)
}

#[tauri::command]
pub fn toggle_pause(state: State<'_, PlayerState>) -> Result<bool, String> {
    let sink = state.sink.lock().map_err(|_| "Lock error")?;
    if sink.is_paused() {
        sink.play();
        Ok(false)
    } else {
        sink.pause();
        Ok(true)
    }
}

#[tauri::command]
pub async fn scan_library(path: String) -> Result<Vec<Song>, String> {
    Ok(scanner::scan_directory(&path))
}

#[tauri::command]
pub fn save_library(songs: Vec<Song>, handle: tauri::AppHandle) -> Result<(), String> {
    let path = handle.path().app_config_dir().unwrap().join("library.json");
    let json = serde_json::to_string(&songs).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_volume(volume: f32, state: tauri::State<'_, crate::player::PlayerState>) -> Result<(), String> {
    println!("set_volume called with: {}", volume);
    let sink = state.sink.lock().map_err(|_| "Failed to lock sink")?;
    println!("sink volume before: {}", sink.volume());
    sink.set_volume(volume);
    println!("sink volume after: {}", sink.volume());
    Ok(())
}