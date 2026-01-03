use zbus::Connection;
use serde::Serialize;
use std::collections::HashMap;
use zbus::zvariant::{OwnedValue, Value};

#[derive(Serialize, Debug, Clone)]
pub struct SpotifyTrack {
    pub title: String,
    pub artist: String,
    pub cover_art: String,
    pub is_playing: bool,  
}

#[zbus::proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2.spotify",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait SpotifyPlayer {
    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
    
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;  // NEW: Get playback status
}

pub async fn get_current_track() -> Option<SpotifyTrack> {
    let connection = match Connection::session().await {
        Ok(conn) => {
            conn
        },
        Err(e) => {
            return None;
        }
    };
    
    let proxy = match SpotifyPlayerProxy::new(&connection).await {
        Ok(p) => {
            p
        },
        Err(e) => {
            return None;
        }
    };

    // Get playback status
    let playback_status = match proxy.playback_status().await {
        Ok(status) => {
            eprintln!("[Spotify] Playback status: {}", status);
            status
        },
        Err(e) => {
            eprintln!("[Spotify] Failed to get playback status: {}", e);
            return None;
        }
    };

    let is_playing = playback_status == "Playing";

    let metadata = match proxy.metadata().await {
        Ok(m) => {
            eprintln!("[Spotify] Metadata received with {} keys", m.len());
            m
        },
        Err(e) => {
            eprintln!("[Spotify] Failed to get metadata: {}", e);
            return None;
        }
    };

    // Extract title
    let title = match metadata
        .get("xesam:title")
        .and_then(|v| v.downcast_ref::<zbus::zvariant::Str>().ok())
        .map(|s| s.to_string()) {
        Some(t) => {
            eprintln!("[Spotify] Title: {}", t);
            t
        },
        None => {
            eprintln!("[Spotify] Failed to extract title");
            return None;
        }
    };

    // Extract artist
    let artist = match metadata
        .get("xesam:artist")
        .and_then(|v| v.downcast_ref::<zbus::zvariant::Array>().ok())
        .and_then(|arr| {
            arr.get(0)
                .ok()
                .flatten()
                .and_then(|v: &Value| v.downcast_ref::<zbus::zvariant::Str>().ok())
                .map(|s: zbus::zvariant::Str| s.to_string())
        }) {
        Some(a) => {
            eprintln!("[Spotify] Artist: {}", a);
            a
        },
        None => {
            eprintln!("[Spotify] Failed to extract artist");
            return None;
        }
    };

    // Extract cover art
    let cover_art = match metadata
        .get("mpris:artUrl")
        .and_then(|v| v.downcast_ref::<zbus::zvariant::Str>().ok())
        .map(|s| s.to_string()) {
        Some(c) => {
            eprintln!("[Spotify] Cover art: {}", c);
            c
        },
        None => {
            eprintln!("[Spotify] Failed to extract cover art");
            return None;
        }
    };

    let track = SpotifyTrack {
        title,
        artist,
        cover_art,
        is_playing,
    };
    
    eprintln!("[Spotify] Successfully created track: {:?}", track);
    Some(track)
}