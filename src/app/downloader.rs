use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Local;
use id3::TagLike;
use regex::Regex;
use serde_json::Value;
use url::Url;
use std::collections::HashMap;

pub mod downloader {

    // Logger.
    pub fn log_error(message: &str, log_file: Option<&str>) {
        let path = log_file.unwrap_or(LOG_FILE);
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "[{timestamp}] {message}");
        }
    }


    // URL utils
    pub fn clean_url(url: &str) -> String {
        match Url::parse(input) {
            Ok(mut parsed) => {
                parsed.set_query(None);
                parsed.set_fragment(None);
                parsed.to_string()
            }
            Err(_) => input.to_string(),
        }
    }

    pub fn is_playlist_url(url: &str) -> bool {
        let re = Regex::new(r"^https://soundcloud\.com/[^/]+/sets/[^/]+").unwrap();
        re.is_match(url)
    }


    // Filename helpers

    #[derive(Debug, Clone, Default)]
    pub struct TitleParts {
        pub album: String,
        pub artist: String,
        pub title: String,
    }

    pub fn parse_title(full_title: &str) -> TitleParts {
        let parts: Vec<String> = full_title.split('-').map(|p| p.trim().to_string()).collect();
        if parts.len() >= 3 {
            TitleParts {
                album: parts[0].clone(),
                artist: parts[1].clone(),
                title: parts[2].clone(),
            }
        } else if parts.len() == 2 {
            TitleParts {
                album: String::new(),
                artist: parts[0].clone(),
                title: parts[1].clone(),
            }
        } else {
            TitleParts {
                album: String::new(),
                artist: String::new(),
                title: full_title.to_string(),
            }
        }
    }

    pub fn sanitize(text: &str) -> String {
        let re = Regex::new(r"[^a-zA-Z0-9 \-_.]").unwrap();
        re.replace_all(text, "").to_string()
    }

    pub fn get_playlist_tracks(playlist_url: &str) -> (Vec<String>, String) {
        let playlist_url = clean_url(playlist_url);
        let fallback_title = "Playlist".to_string();

        let output = match Command::new("yt-dlp")
            .args(["--no-warnings", "--flat-playlist", "-J", &playlist_url])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                log_error(&format!("Error fetching playlist: {e}"), None);
                return (Vec::new(), fallback_title);
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log_error(&format!("Error fetching playlist: {stderr}"), None);
            return (Vec::new(), fallback_title);
        }

        let data: Value = match serde_json::from_slice(&output.stdout) {
            Ok(v) => v,
            Err(e) => {
                log_error(&format!("Error fetching playlist: {e}"), None);
                return (Vec::new(), fallback_title);
            }
        };

        let tracks = data["entries"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|e| e["url"].as_str())
                    .map(clean_url)
                    .collect()
            })
            .unwrap_or_default();

        let title = data["title"].as_str().unwrap_or("Playlist").to_string();
        (tracks, title)
    }



}