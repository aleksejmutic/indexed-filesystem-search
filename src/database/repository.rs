use crate::filesystem::entry::FileEntry;
use rusqlite::{Connection, OptionalExtension, Result, Row};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

//insert a file entry row, last_seen is only important in the context of deletion states, where a file entry needs to be removed from the database
//when it no longer exists in the filesystem
pub fn insert_file_entry(conn: &Connection, entry: &FileEntry, scan_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO file_entries (
            device,
            inode,
            path,
            filename,
            extension,
            size,
            modified,
            is_directory,
            is_symlink,
            is_executable,
            is_hidden,
            last_seen
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(path) DO UPDATE SET
            device = excluded.device,
            inode = excluded.inode,
            filename = excluded.filename,
            extension = excluded.extension,
            size = excluded.size,
            modified = excluded.modified,
            is_directory = excluded.is_directory,
            is_symlink = excluded.is_symlink,
            is_executable = excluded.is_executable,
            is_hidden = excluded.is_hidden,
            last_seen = excluded.last_seen",
        (
            entry.device as i64,
            entry.inode as i64,
            entry.path.to_string_lossy().to_string(),
            entry
                .filename
                .as_ref()
                .map(|x| x.to_string_lossy().to_string()),
            entry
                .extension
                .as_ref()
                .map(|x| x.to_string_lossy().to_string()),
            entry.size as i64,
            entry
                .modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64,
            entry.is_directory,
            entry.is_symlink,
            entry.is_executable,
            entry.is_hidden,
            scan_id,
        ),
    )?;

    Ok(())
}

// This function takes a row from the database and converts into a FileEntry type
// An offset is needed because some queries select file_entries.id before
// the other file_entries columns. In those queries, the FileEntry data
// starts at column 1 instead of column 0. The offset tells this function
// where the file_entries fields begin in the query result.
fn file_entry_from_row(row: &Row, offset: usize) -> Result<FileEntry> {
    let device: i64 = row.get(offset)?;
    let inode: i64 = row.get(offset + 1)?;
    let path: String = row.get(offset + 2)?;
    let filename: Option<String> = row.get(offset + 3)?;
    let extension: Option<String> = row.get(offset + 4)?;
    let size: i64 = row.get(offset + 5)?;
    let modified: i64 = row.get(offset + 6)?;
    let is_directory: bool = row.get(offset + 7)?;
    let is_symlink: bool = row.get(offset + 8)?;
    let is_executable: bool = row.get(offset + 9)?;
    let is_hidden: bool = row.get(offset + 10)?;

    Ok(FileEntry {
        device: device as u64,
        inode: inode as u64,
        path: PathBuf::from(path),
        filename: filename.map(OsString::from),
        extension: extension.map(OsString::from),
        size: size as u64,
        modified: UNIX_EPOCH + Duration::from_nanos(modified as u64),
        is_directory,
        is_symlink,
        is_executable,
        is_hidden,
    })
}

//read one file entry by id
fn get_file_entry(conn: &Connection, id: i64) -> Result<FileEntry> {
    let mut statement = conn.prepare(
        "SELECT
            device,
            inode,
            path,
            filename,
            extension,
            size,
            modified,
            is_directory,
            is_symlink,
            is_executable,
            is_hidden
        FROM file_entries
        WHERE id = ?1",
    )?;

    let entry = statement.query_row([id], |row| file_entry_from_row(row, 0))?;

    Ok(entry) //wrapping the entry again into a Result type, promised in the function declaration as the return type
}

// we get file entries by their path
pub fn get_file_entry_by_path(conn: &Connection, path: &str) -> Result<Option<(i64, FileEntry)>> {
    let mut statement = conn.prepare(
        "SELECT
            id,
            device,
            inode,
            path,
            filename,
            extension,
            size,
            modified,
            is_directory,
            is_symlink,
            is_executable,
            is_hidden
        FROM file_entries
        WHERE path = ?1",
    )?;

    let entry = statement
        .query_row([path], |row| {
            let id: i64 = row.get(0)?;
            let entry = file_entry_from_row(row, 1)?;

            Ok((id, entry))
        })
        .optional()?;

    Ok(entry)
}

//read all file entries
fn get_all_file_entries(conn: &Connection) -> Result<Vec<FileEntry>> {
    let mut statement = conn.prepare(
        "SELECT
            device,
            inode,
            path,
            filename,
            extension,
            size,
            modified,
            is_directory,
            is_symlink,
            is_executable,
            is_hidden
        FROM file_entries",
    )?;

    let entries = statement.query_map([], |row| file_entry_from_row(row, 0))?;

    let mut entries_vec = Vec::new();

    for entry in entries {
        entries_vec.push(entry?);
    }

    Ok(entries_vec)
}

//deletes a file entry
fn delete_file_entry(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM file_entries WHERE id = ?1", [id])?;

    Ok(())
}

//updates an existing file entry
pub fn update_file_entry(
    conn: &Connection,
    id: i64,
    entry: &FileEntry,
    scan_id: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE file_entries
        SET
            device = ?1,
            inode = ?2,
            path = ?3,
            filename = ?4,
            extension = ?5,
            size = ?6,
            modified = ?7,
            is_directory = ?8,
            is_symlink = ?9,
            is_executable = ?10,
            is_hidden = ?11,
            last_seen = ?12
        WHERE id = ?13",
        (
            entry.device as i64,
            entry.inode as i64,
            entry.path.to_string_lossy().to_string(),
            entry
                .filename
                .as_ref()
                .map(|x| x.to_string_lossy().to_string()),
            entry
                .extension
                .as_ref()
                .map(|x| x.to_string_lossy().to_string()),
            entry.size as i64,
            entry
                .modified
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64,
            entry.is_directory,
            entry.is_symlink,
            entry.is_executable,
            entry.is_hidden,
            scan_id,
            id,
        ),
    )?;

    Ok(())
}

// updating the last_seen field
pub fn update_last_seen(conn: &Connection, id: i64, scan_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE file_entries
         SET last_seen = ?1
         WHERE id = ?2",
        (scan_id, id),
    )?;

    Ok(())
}

//search using fts5 index
pub fn search_file_entries(conn: &Connection, query: &str, limit: i64) -> Result<Vec<FileEntry>> {
    let mut statement = conn.prepare(
        "SELECT
            device,
            inode,
            path,
            filename,
            extension,
            size,
            modified,
            is_directory,
            is_symlink,
            is_executable,
            is_hidden
        FROM file_entries
        WHERE id IN (
            SELECT rowid
            FROM file_entries_fts
            WHERE file_entries_fts MATCH ?1
        )
        LIMIT ?2",
    )?;

    let entries = statement.query_map((query, limit), |row| file_entry_from_row(row, 0))?;

    let mut entries_vec = Vec::new();

    for entry in entries {
        entries_vec.push(entry?);
    }

    Ok(entries_vec)
}
