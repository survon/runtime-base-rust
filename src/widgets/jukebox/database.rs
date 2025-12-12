// src/widgets/jukebox/database.rs
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};
use crate::util::database::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: i64,
    pub module_name: String,
    pub title: String,
    pub artist: String,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub credits: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub album_id: i64,
    pub track_number: i32,
    pub title: String,
    pub duration_seconds: Option<i32>,
    pub file_path: String,
    pub artist: Option<String>,
}

/// Trait to add Jukebox-specific database operations to Database
pub trait JukeboxDatabase {
    fn init_jukebox_schema(&self) -> Result<()>;

    // Album operations
    fn insert_album(&self, album: &Album) -> Result<i64>;
    fn get_all_albums(&self) -> Result<Vec<Album>>;
    fn clear_albums(&self) -> Result<()>;

    // Track operations
    fn insert_track(&self, track: &Track) -> Result<()>;
    fn get_tracks_for_album(&self, album_id: i64) -> Result<Vec<Track>>;
}

impl JukeboxDatabase for Database {
    fn init_jukebox_schema(&self) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS albums (
                id INTEGER PRIMARY KEY,
                module_name TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                year INTEGER,
                genre TEXT,
                credits TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY,
                album_id INTEGER NOT NULL,
                track_number INTEGER NOT NULL,
                title TEXT NOT NULL,
                duration_seconds INTEGER,
                file_path TEXT NOT NULL,
                artist TEXT,
                FOREIGN KEY (album_id) REFERENCES albums(id)
            )",
            [],
        )?;

        Ok(())
    }

    fn insert_album(&self, album: &Album) -> Result<i64> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "INSERT INTO albums (module_name, title, artist, year, genre, credits)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                album.module_name,
                album.title,
                album.artist,
                album.year,
                album.genre,
                album.credits,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn get_all_albums(&self) -> Result<Vec<Album>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, module_name, title, artist, year, genre, credits
             FROM albums ORDER BY artist, title"
        )?;

        let albums = stmt.query_map([], |row| {
            Ok(Album {
                id: row.get(0)?,
                module_name: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                year: row.get(4)?,
                genre: row.get(5)?,
                credits: row.get(6)?,
            })
        })?;

        albums.collect()
    }

    fn clear_albums(&self) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute("DELETE FROM tracks", [])?;
        conn.execute("DELETE FROM albums", [])?;
        Ok(())
    }

    fn insert_track(&self, track: &Track) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tracks (album_id, track_number, title, duration_seconds, file_path, artist)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                track.album_id,
                track.track_number,
                track.title,
                track.duration_seconds,
                track.file_path,
                track.artist,
            ],
        )?;
        Ok(())
    }

    fn get_tracks_for_album(&self, album_id: i64) -> Result<Vec<Track>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, album_id, track_number, title, duration_seconds, file_path, artist
             FROM tracks WHERE album_id = ?1 ORDER BY track_number"
        )?;

        let tracks = stmt.query_map([album_id], |row| {
            Ok(Track {
                id: row.get(0)?,
                album_id: row.get(1)?,
                track_number: row.get(2)?,
                title: row.get(3)?,
                duration_seconds: row.get(4)?,
                file_path: row.get(5)?,
                artist: row.get(6)?,
            })
        })?;

        tracks.collect()
    }
}
