use crate::filesystem::entry::FileEntry;
use rusqlite::{Connection, Result, Row};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

//insert a file entry row
pub fn insert_file_entry(conn: &Connection, entry: &FileEntry) -> Result<()> {
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
            is_hidden
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
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
                .as_secs() as i64,
            entry.is_directory,
            entry.is_hidden,
        ),
    )?;

    Ok(())
}

fn file_entry_from_row(row: &Row) -> Result<FileEntry> {
    let device: i64 = row.get(0)?;
    let inode: i64 = row.get(1)?;
    let path: String = row.get(2)?;
    let filename: Option<String> = row.get(3)?;
    let extension: Option<String> = row.get(4)?;
    let size: i64 = row.get(5)?;
    let modified: i64 = row.get(6)?;
    let is_directory: bool = row.get(7)?;
    let is_hidden: bool = row.get(8)?;

    Ok(FileEntry {
        device: device as u64,
        inode: inode as u64,
        path: PathBuf::from(path),
        filename: filename.map(OsString::from),
        extension: extension.map(OsString::from),
        size: size as u64,
        modified: UNIX_EPOCH + Duration::from_secs(modified as u64),
        is_directory,
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
            is_hidden
        FROM file_entries
        WHERE id = ?1",
    )?;

    let entry = statement.query_row([id], file_entry_from_row)?;

    Ok(entry) //wrapping the entry again into a Result type, promised in the function declaration as the return type
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
            is_hidden
        FROM file_entries",
    )?;

    let entries = statement.query_map([], file_entry_from_row)?;

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
fn update_file_entry(conn: &Connection, id: i64, entry: &FileEntry) -> Result<()> {
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
            is_hidden = ?9
        WHERE id = ?10",
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
            entry.modified.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            entry.is_directory,
            entry.is_hidden,
            id,
        ),
    )?;

    Ok(())
}
