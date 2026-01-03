use zbus::Connection;
use serde::Serialize;
use std::collections::HashMap;
use zbus::zvariant::{OwnedValue, Value};

#[derive(Serialize, Debug, Clone)]
pub struct MediaTrack {
    pub title: String,
    pub artist: String,
    pub cover_art: String,
    pub is_playing: bool,
    pub player_name: String,
}

#[zbus::proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait MediaPlayer {
    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
    
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;
}

// Find all MPRIS media players
async fn find_media_players() -> Vec<String> {
    let connection = match Connection::session().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("[Media] Failed to connect to D-Bus: {}", e);
            return Vec::new();
        }
    };
    
    let proxy = match zbus::fdo::DBusProxy::new(&connection).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[Media] Failed to create D-Bus proxy: {}", e);
            return Vec::new();
        }
    };
    
    let names = match proxy.list_names().await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[Media] Failed to list names: {}", e);
            return Vec::new();
        }
    };
    
    // Find all MPRIS media players
    let players: Vec<String> = names
        .into_iter()
        .filter(|name| name.starts_with("org.mpris.MediaPlayer2."))
        .map(|s| s.to_string())
        .collect();
    
    eprintln!("[Media] Found {} media players: {:?}", players.len(), players);
    players
}

// Get player priority (lower number = higher priority)
fn get_player_priority(player_name: &str) -> u32 {
    if player_name.contains("spotify") {
        return 1;
    } else if player_name.contains("vlc") {
        return 2;
    } else if player_name.contains("firefox") {
        return 3;
    } else if player_name.contains("chromium") || player_name.contains("chrome") {
        return 4;
    } else if player_name.contains("brave") {
        return 5;
    } else if player_name.contains("mpv") {
        return 6;
    }
    
    // Default priority for unknown players
    100
}

// Extract player display name
fn get_player_display_name(service_name: &str) -> String {
    let name = service_name
        .strip_prefix("org.mpris.MediaPlayer2.")
        .unwrap_or(service_name);
    
    // Handle instance IDs (e.g., "chromium.instance12345" -> "Chromium")
    let name = name.split('.').next().unwrap_or(name);
    
    // Capitalize first letter
    if let Some(first_char) = name.chars().next() {
        let rest: String = name.chars().skip(1).collect();
        format!("{}{}", first_char.to_uppercase(), rest)
    } else {
        name.to_string()
    }
}

async fn get_track_from_player(service_name: &str) -> Option<MediaTrack> {
    let connection = Connection::session().await.ok()?;
    
    // Build proxy with custom destination using the builder pattern
    let proxy_builder = zbus::proxy::Builder::new(&connection)
        .interface("org.mpris.MediaPlayer2.Player").ok()?
        .path("/org/mpris/MediaPlayer2").ok()?
        .destination(service_name).ok()?;
    
    let proxy: MediaPlayerProxy = proxy_builder.build().await.ok()?;

    // Get playback status
    let playback_status = proxy.playback_status().await.ok()?;
    let is_playing = playback_status == "Playing";
    
    // Skip if stopped
    if playback_status == "Stopped" {
        return None;
    }

    let metadata = proxy.metadata().await.ok()?;
    
    // Extract title
    let title = metadata
        .get("xesam:title")
        .and_then(|v: &OwnedValue| v.downcast_ref::<zbus::zvariant::Str>().ok())
        .map(|s: zbus::zvariant::Str| s.to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());

    // Extract artist (handle missing artist gracefully)
    let artist = metadata
        .get("xesam:artist")
        .and_then(|v: &OwnedValue| v.downcast_ref::<zbus::zvariant::Array>().ok())
        .and_then(|arr: zbus::zvariant::Array| {
            arr.get(0)
                .ok()
                .flatten()
                .and_then(|v: &Value| v.downcast_ref::<zbus::zvariant::Str>().ok())
                .map(|s: zbus::zvariant::Str| s.to_string())
        })
        .or_else(|| {
            // Try albumArtist as fallback
            metadata
                .get("xesam:albumArtist")
                .and_then(|v: &OwnedValue| v.downcast_ref::<zbus::zvariant::Array>().ok())
                .and_then(|arr: zbus::zvariant::Array| {
                    arr.get(0)
                        .ok()
                        .flatten()
                        .and_then(|v: &Value| v.downcast_ref::<zbus::zvariant::Str>().ok())
                        .map(|s: zbus::zvariant::Str| s.to_string())
                })
        })
        .unwrap_or_else(|| get_player_display_name(service_name));

    // Extract cover art (handle missing artwork)
    let cover_art = metadata
        .get("mpris:artUrl")
        .and_then(|v: &OwnedValue| v.downcast_ref::<zbus::zvariant::Str>().ok())
        .map(|s: zbus::zvariant::Str| s.to_string())
        .unwrap_or_else(|| "".to_string());

    let player_name = get_player_display_name(service_name);

    eprintln!("[Media] Found track from {}: {} - {}", player_name, title, artist);

    Some(MediaTrack {
        title,
        artist,
        cover_art,
        is_playing,
        player_name,
    })
}

pub async fn get_current_track() -> Option<MediaTrack> {
    eprintln!("[Media] Starting media fetch...");
    
    let players = find_media_players().await;
    
    if players.is_empty() {
        eprintln!("[Media] No media players found");
        return None;
    }
    
    // Get tracks from all players
    let mut tracks = Vec::new();
    
    for player in &players {
        if let Some(track) = get_track_from_player(player).await {
            tracks.push((player.clone(), track));
        }
    }
    
    if tracks.is_empty() {
        eprintln!("[Media] No playing tracks found");
        return None;
    }
    
    // Sort by priority (lowest priority number first)
    tracks.sort_by_key(|(service_name, _)| get_player_priority(service_name));
    
    // Return highest priority playing track
    let (service_name, track) = tracks.into_iter().next()?;
    eprintln!("[Media] Selected player: {} ({})", track.player_name, service_name);
    
    Some(track)
}