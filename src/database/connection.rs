use rusqlite::{Connection, Result};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

//connection creation
fn connect() -> Result<Connection> {
    let conn = Connection::open("indexed-files.db")?;

    Ok(conn)
}

//schema creation
fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "create table if not exists file_entries (
            id INTEGER PRIMARY KEY,
            device INTEGER NOT NULL,
            inode INTEGER NOT NULL,
            path TEXT NOT NULL UNIQUE,
            filename TEXT,
            extension TEXT,
            size INTEGER NOT NULL,
            modified INTEGER NOT NULL,
            is_directory INTEGER NOT NULL,
            is_hidden INTEGER NOT NULL
        )",
        (),
    )?;
    Ok(())
}

//insert a file entry row
fn insert_file_entry(conn: &Connection, entry: &FileEntry) -> Result<()> {
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
            entry.device,
            entry.inode,
            entry.path.to_string_lossy().to_string(),
            entry
                .filename
                .as_ref()
                .map(|x| x.to_string_lossy().to_string()),
            entry
                .extension
                .as_ref()
                .map(|x| x.to_string_lossy().to_string()),
            entry.size,
            entry
                .modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            entry.is_directory,
            entry.is_hidden,
        ),
    )?;

    Ok(())
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

    let entry = statement.query_row([id], |row| {
        let device: u64 = row.get(0)?;
        let inode: u64 = row.get(1)?;
        let path: String = row.get(2)?;
        let filename: Option<String> = row.get(3)?;
        let extension: Option<String> = row.get(4)?;
        let size: u64 = row.get(5)?;
        let modified: i64 = row.get(6)?;
        let is_directory: bool = row.get(7)?;
        let is_hidden: bool = row.get(8)?;

        Ok(FileEntry {
            device,
            inode,
            path: PathBuf::from(path),
            filename: filename.map(OsString::from),
            extension: extension.map(OsString::from),
            size,
            modified: std::time::UNIX_EPOCH + std::time::Duration::from_secs(modified as u64),
            is_directory,
            is_hidden,
        })
    })?;

    Ok(entry) //wrapping the entry again into a Result type, promised in the function declaration as the return type
}
