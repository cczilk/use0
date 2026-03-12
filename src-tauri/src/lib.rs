pub mod database;
pub mod commands;
pub mod downloader;
pub mod player;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use database::Database;
    use commands::AppState;
    use player::Player;
    use std::sync::Mutex;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
            let db_path = app_dir.join("music_library.db");
            let db = Database::new(db_path).expect("Failed to initialize database");
            app.manage(AppState { db: Mutex::new(db) });
            let downloads_dir = app_dir.join("downloads");
            std::fs::create_dir_all(&downloads_dir).expect("Failed to create downloads directory");
            let player = Player::new(downloads_dir.clone()).expect("Failed to initialise audio engine");
            player.start_ticker(app.handle().clone());
            app.manage(player);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::search_tracks,
            commands::library::format_duration,
            commands::get_all_tracks,
            commands::get_track_info,
            commands::get_thumbnail,
            commands::get_thumbnail_base64,
            commands::delete_track,
            commands::import_local_file,
            commands::import_folder,
            commands::update_track_artwork,
            commands::update_track_metadata,
            commands::create_playlist,
            commands::get_all_playlists,
            commands::add_track_to_playlist,
            commands::get_playlist_tracks,
            commands::remove_from_playlist,
            commands::rescan_artwork,
            commands::playback::player_play_track,
            commands::playback::player_pause,
            commands::playback::player_resume,
            commands::playback::player_stop,
            commands::playback::player_seek,
            commands::playback::player_set_volume,
            commands::playback::player_set_queue,
            commands::playback::player_next,
            commands::playback::player_previous,
            commands::playback::player_set_shuffle,
            commands::playback::player_set_autoplay,
            commands::playback::player_get_state,
            commands::library::manual_update_artwork,
            downloader::download_from_youtube,
            get_downloads_dir,
            get_config_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_config_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn get_downloads_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    Ok(app_dir.join("downloads").to_string_lossy().to_string())
}