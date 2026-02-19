use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
    pub cover: Option<String>,
}

pub fn scan_directory(dir: &str) -> Vec<Song> {
    let mut songs = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_audio_file(path) {
            let path_str = path.to_string_lossy().to_string();
            if let Ok(tagged_file) = Probe::open(path).unwrap().read() {
                let properties = tagged_file.properties();
                let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

                let cover = tag.and_then(|t| t.pictures().first()).map(|pic| {
                    let data = pic.data();
                    let mime = pic.mime_type()
                        .map(|m| m.as_str())
                        .unwrap_or("image/jpeg");
                    format!(
                        "data:{};base64,{}",
                        mime,
                        general_purpose::STANDARD.encode(data)
                    )
                });

                songs.push(Song {
                    path: path_str,
                    title: tag.and_then(|t| t.title().map(|s| s.into_owned()))
                        .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().into()),
                    artist: tag.and_then(|t| t.artist().map(|s| s.into_owned()))
                        .unwrap_or_else(|| "Unknown Artist".to_string()),
                    album: tag.and_then(|t| t.album().map(|s| s.into_owned()))
                        .unwrap_or_else(|| "Unknown Album".to_string()),
                    duration: properties.duration().as_secs(),
                    cover,
                });
            }
        }
    }
    songs
}

fn is_audio_file(path: &Path) -> bool {
    let extensions = ["mp3", "flac", "wav", "m4a", "ogg"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}