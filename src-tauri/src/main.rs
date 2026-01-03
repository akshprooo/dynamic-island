mod spotify;

use tauri::Manager;

#[tauri::command]
async fn spotify_now_playing() -> Option<spotify::SpotifyTrack> {
    spotify::get_current_track().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("main window not found");

            // Click-through overlay
            window.set_ignore_cursor_events(true).ok();

            // Always on top
            window.set_always_on_top(true).ok();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            spotify_now_playing
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}