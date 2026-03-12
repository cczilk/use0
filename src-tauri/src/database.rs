use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use crate::commands::{TrackRow, PlaylistRow};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS tracks (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                title          TEXT    NOT NULL,
                artist         TEXT,
                album          TEXT,
                duration       INTEGER,
                file_path      TEXT    NOT NULL UNIQUE,
                thumbnail_path TEXT,
                created_at     INTEGER DEFAULT (strftime('%s','now'))
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s','now'))
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
                track_id    INTEGER NOT NULL REFERENCES tracks(id)    ON DELETE CASCADE,
                position    INTEGER DEFAULT 0,
                PRIMARY KEY (playlist_id, track_id)
            );

            CREATE INDEX IF NOT EXISTS idx_tracks_title  ON tracks(title);
            CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
        ")?;
        Ok(())
    }

    pub fn get_all_tracks(&self) -> Result<Vec<TrackRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, artist, album, duration, file_path, thumbnail_path
             FROM tracks ORDER BY title COLLATE NOCASE"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TrackRow {
                id:             row.get(0)?,
                title:          row.get(1)?,
                artist:         row.get(2)?,
                album:          row.get(3)?,
                duration:       row.get(4)?,
                file_path:      row.get(5)?,
                thumbnail_path: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_track_by_id(&self, id: i64) -> Result<Option<TrackRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, artist, album, duration, file_path, thumbnail_path
             FROM tracks WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(TrackRow {
                id:             row.get(0)?,
                title:          row.get(1)?,
                artist:         row.get(2)?,
                album:          row.get(3)?,
                duration:       row.get(4)?,
                file_path:      row.get(5)?,
                thumbnail_path: row.get(6)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn insert_track(
        &self,
        title: &str,
        artist: Option<&str>,
        album: Option<&str>,
        duration: Option<i64>,
        file_path: &str,
        thumbnail_path: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tracks (title, artist, album, duration, file_path, thumbnail_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![title, artist, album, duration, file_path, thumbnail_path],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_track(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_track_metadata(
        &self,
        id: i64,
        title: &str,
        artist: Option<&str>,
        album: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE tracks SET title=?1, artist=?2, album=?3 WHERE id=?4",
            params![title, artist, album, id],
        )?;
        Ok(())
    }

    // UPDATED: This allows external commands to update the thumbnail safely
    pub fn update_track_thumbnail(&self, id: i64, thumbnail_path: Option<String>) -> Result<()> {
        self.conn.execute(
            "UPDATE tracks SET thumbnail_path=?1 WHERE id=?2",
            params![thumbnail_path, id],
        )?;
        Ok(())
    }

    // ── Playlists ─────────────────────────────────────────────────────────────

    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO playlists (name) VALUES (?1)",
            params![name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_playlists(&self) -> Result<Vec<PlaylistRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name,
                    COUNT(pt.track_id) as track_count
             FROM playlists p
             LEFT JOIN playlist_tracks pt ON p.id = pt.playlist_id
             GROUP BY p.id, p.name
             ORDER BY p.name COLLATE NOCASE"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PlaylistRow {
                id:          row.get(0)?,
                name:        row.get(1)?,
                track_count: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn add_track_to_playlist(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        let pos: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_tracks WHERE playlist_id=?1",
            params![playlist_id],
            |row| row.get(0),
        ).unwrap_or(0);
        self.conn.execute(
            "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position)
             VALUES (?1, ?2, ?3)",
            params![playlist_id, track_id, pos],
        )?;
        Ok(())
    }

    pub fn get_playlist_tracks(&self, playlist_id: i64) -> Result<Vec<TrackRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.title, t.artist, t.album, t.duration, t.file_path, t.thumbnail_path
             FROM tracks t
             JOIN playlist_tracks pt ON t.id = pt.track_id
             WHERE pt.playlist_id = ?1
             ORDER BY pt.position"
        )?;
        let rows = stmt.query_map(params![playlist_id], |row| {
            Ok(TrackRow {
                id:             row.get(0)?,
                title:          row.get(1)?,
                artist:         row.get(2)?,
                album:          row.get(3)?,
                duration:       row.get(4)?,
                file_path:      row.get(5)?,
                thumbnail_path: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_track_by_path(&self, path: &str) -> rusqlite::Result<Option<crate::commands::TrackRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,title,artist,album,duration,file_path,thumbnail_path FROM tracks WHERE file_path=?1 LIMIT 1"
        )?;
        let mut rows = stmt.query_map([path], |row| Ok(crate::commands::TrackRow {
            id: row.get(0)?, title: row.get(1)?, artist: row.get(2)?,
            album: row.get(3)?, duration: row.get(4)?, file_path: row.get(5)?,
            thumbnail_path: row.get(6)?,
        }))?;
        Ok(rows.next().transpose()?)
    }


    pub fn remove_from_playlist(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id=?1 AND track_id=?2",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

}