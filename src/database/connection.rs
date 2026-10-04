use rusqlite::{Connection, Result};

//connection creation
pub fn connect() -> Result<Connection> {
    let conn = Connection::open("indexed-files.db")?;

    Ok(conn)
}

//schema creation
pub fn create_schema(conn: &Connection) -> Result<()> {
    //creates the file_entries table
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
            is_symlink INTEGER NOT NULL,
            is_executable INTEGER NOT NULL,
            is_hidden INTEGER NOT NULL,
            last_seen INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )?;

    //creates the fts5 index from file_entries table
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS file_entries_fts USING fts5(
            filename,
            extension,
            path,
            content='file_entries',
            content_rowid='id'
        )",
        (),
    )?;

    //creates a metadata table that would check whether fts5 table is filled with data or not
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        (),
    )?;

    Ok(())
}
