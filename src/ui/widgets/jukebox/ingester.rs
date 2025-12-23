// src/widgets/jukebox/ingester.rs
use std::path::Path;
use std::fs;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use color_eyre::Result;
use crate::modules::{Module, ModuleManager};
use crate::util::database::Database;
use super::database::{JukeboxDatabase, Album, Track};
use crate::{log_info, log_warn, log_debug};

pub struct JukeboxIngester<'a> {
    database: &'a Database,
}

impl<'a> JukeboxIngester<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    /// Check if we need to reingest based on checksum (like knowledge ingester)
    pub fn should_reingest(&self) -> Result<bool> {
        let current_checksum = self.calculate_albums_checksum()?;

        match self.database.get_module_state("jukebox_checksum") {
            Ok(Some(stored_checksum)) => {
                log_debug!("Jukebox checksum: stored={}, current={}", stored_checksum, current_checksum);
                let stored: u64 = stored_checksum.parse().unwrap_or(0);
                Ok(current_checksum != stored)
            }
            _ => Ok(true) // No checksum stored, need to ingest
        }
    }

    pub fn ingest_albums(&self, module_manager: &ModuleManager) -> Result<()> {
        log_info!("🎵 Starting album ingestion...");

        // Clear existing data
        self.database.clear_albums()?;

        let album_modules = module_manager.get_modules_by_type("album");
        let mut total_tracks = 0;
        let album_count = album_modules.len(); // Store length before consuming

        for module in album_modules {
            let tracks = self.ingest_album_module(module)?;
            total_tracks += tracks;
        }

        // Store new checksum
        let checksum = self.calculate_albums_checksum()?;
        self.database.save_module_state("jukebox_checksum", &checksum.to_string())?;

        log_info!("✅ Album ingestion complete: {} tracks from {} albums", total_tracks, album_count);

        Ok(())
    }

    fn ingest_album_module(&self, module: &Module) -> Result<usize> {
        let config = &module.config;
        log_info!("  🎀 Processing: {}", config.name);

        // Extract album metadata from bindings
        let title = config.bindings.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(&config.name)
            .to_string();

        let artist = config.bindings.get("artist")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Artist")
            .to_string();

        let year = config.bindings.get("year")
            .and_then(|v| v.as_i64())
            .map(|y| y as i32);

        let genre = config.bindings.get("genre")
            .and_then(|v| v.as_str())
            .map(String::from);

        let credits = config.bindings.get("credits")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Insert album
        let album = Album {
            id: 0, // Will be assigned by DB
            module_name: config.name.clone(),
            title,
            artist,
            year,
            genre,
            credits,
        };

        let album_id = self.database.insert_album(&album)?;

        // Process tracklist
        let track_count = if let Some(tracklist) = config.bindings.get("tracklist").and_then(|v| v.as_array()) {
            let mut count = 0;

            for (idx, track_value) in tracklist.iter().enumerate() {
                if let Some(track_obj) = track_value.as_object() {
                    let track_title = track_obj.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown Track")
                        .to_string();

                    let file_name = track_obj.get("file")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let file_path = module.path.join("audio").join(file_name);

                    // Check if file exists AND is an audio file (not README.md, etc.)
                    if !file_path.exists() {
                        log_warn!("    ⚠️  Audio file not found: {:?}", file_path);
                        continue;
                    }

                    // Skip non-audio files (README.md, .txt, etc.)
                    if let Some(ext) = file_path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if !matches!(ext_str.as_str(), "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac") {
                            log_debug!("    Skipping non-audio file: {:?}", file_path);
                            continue;
                        }
                    } else {
                        // No extension, skip it
                        log_debug!("    Skipping file without extension: {:?}", file_path);
                        continue;
                    }

                    let duration = track_obj.get("duration_seconds")
                        .and_then(|v| v.as_i64())
                        .map(|d| d as i32);

                    let track_artist = track_obj.get("artist")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let track = Track {
                        id: 0,
                        album_id,
                        track_number: (idx + 1) as i32,
                        title: track_title,
                        duration_seconds: duration,
                        file_path: file_path.to_string_lossy().to_string(),
                        artist: track_artist,
                    };

                    self.database.insert_track(&track)?;
                    count += 1;
                }
            }

            count
        } else {
            0
        };

        log_info!("    ✅ {} tracks", track_count);

        Ok(track_count)
    }

    /// Calculate checksum of all album modules (like knowledge ingester)
    fn calculate_albums_checksum(&self) -> Result<u64> {
        let mut hasher = DefaultHasher::new();

        // Hash all album module directories
        for manifests_dir in ["./manifests/core", "./manifests/wasteland"] {
            let path = std::path::PathBuf::from(manifests_dir);
            if !path.exists() {
                continue;
            }

            for module_dir in fs::read_dir(&path)? {
                let module_dir = module_dir?;
                if !module_dir.file_type()?.is_dir() {
                    continue;
                }

                let config_path = module_dir.path().join("config.yml");
                if !config_path.exists() {
                    continue;
                }

                // Check if it's an album module
                let config_content = fs::read_to_string(&config_path)?;
                if config_content.contains("module_type: \"album\"") {
                    let audio_dir = module_dir.path().join("audio");
                    if audio_dir.exists() {
                        self.hash_directory_recursive(&audio_dir, &mut hasher)?;
                    }
                    // Also hash the config itself
                    config_path.hash(&mut hasher);
                }
            }
        }

        Ok(hasher.finish())
    }

    fn hash_directory_recursive(&self, dir: &Path, hasher: &mut DefaultHasher) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // Only hash audio files, skip README.md and other non-audio files
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if matches!(ext_str.as_str(), "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac") {
                        path.hash(hasher);
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                    duration.as_secs().hash(hasher);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
