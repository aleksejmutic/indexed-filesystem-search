use rusqlite::{Connection, Result};

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

    // Stores usage statistics for files that have been opened, used in the ranking system
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_usage (
            file_entry_id INTEGER PRIMARY KEY,
            use_count INTEGER NOT NULL DEFAULT 0,
            last_used INTEGER NOT NULL,

            FOREIGN KEY (file_entry_id)
                REFERENCES file_entries(id)
                ON DELETE CASCADE
        )",
        (),
    )?;

    // Automatically add new file entries to the FTS5 index
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS file_entries_ai
             AFTER INSERT ON file_entries
             BEGIN
                 INSERT INTO file_entries_fts(rowid, filename, extension, path)
                 VALUES (new.id, new.filename, new.extension, new.path);
             END",
        (),
    )?;

    // Automatically remove deleted file entries from the FTS5 index
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS file_entries_ad
             AFTER DELETE ON file_entries
             BEGIN
                 INSERT INTO file_entries_fts(
                     file_entries_fts, rowid, filename, extension, path
                 )
                 VALUES (
                     'delete', old.id, old.filename, old.extension, old.path
                 );
             END",
        (),
    )?;

    // Automatically update searchable fields in the FTS5 index
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS file_entries_au
             AFTER UPDATE OF filename, extension, path ON file_entries
             BEGIN
                 INSERT INTO file_entries_fts(
                     file_entries_fts, rowid, filename, extension, path
                 )
                 VALUES (
                     'delete', old.id, old.filename, old.extension, old.path
                 );

                 INSERT INTO file_entries_fts(rowid, filename, extension, path)
                 VALUES (new.id, new.filename, new.extension, new.path);
             END",
        (),
    )?;

    Ok(())
}
