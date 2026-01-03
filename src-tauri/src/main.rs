mod media;  // Changed from 'spotify'

use tauri::Manager;

#[tauri::command]
async fn spotify_now_playing() -> Option<media::MediaTrack> {  // Changed return type
    eprintln!("[Main] spotify_now_playing command called");
    let result = media::get_current_track().await;
    eprintln!("[Main] Returning result: {:?}", result);
    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    eprintln!("[Main] Starting Tauri application");
    
    tauri::Builder::default()
        .setup(|app| {
            eprintln!("[Main] Running setup...");
            let window = app
                .get_webview_window("main")
                .expect("main window not found");

            // Click-through overlay
            window.set_ignore_cursor_events(true).ok();

            // Always on top
            window.set_always_on_top(true).ok();

            eprintln!("[Main] Setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            spotify_now_playing
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}