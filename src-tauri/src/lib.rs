pub mod player;
pub mod library;
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(player::PlayerState::new()) 
        .invoke_handler(tauri::generate_handler![
            commands::play_song,
            commands::toggle_pause,
            commands::scan_library 
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}