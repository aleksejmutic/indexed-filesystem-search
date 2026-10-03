use rusqlite::{Connection, Result};

//connection creation
pub fn connect() -> Result<Connection> {
    let conn = Connection::open("indexed-files.db")?;

    Ok(conn)
}

//schema creation
pub fn create_schema(conn: &Connection) -> Result<()> {
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
    Ok(())
}
