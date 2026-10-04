use rusqlite::{Connection, OptionalExtension, Result};

//populating the fts5 index
pub fn populate_fts(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO file_entries_fts(rowid, filename, extension, path)
         SELECT id, filename, extension, path
         FROM file_entries",
        (),
    )?;

    Ok(())
}

////checks whether the fts5 index has been initially populated
pub fn is_fts_populated(conn: &Connection) -> Result<bool> {
    let populated: Option<String> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'fts_populated'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    Ok(populated.as_deref() == Some("true"))
}

//marks the fts5 table as populated
pub fn mark_fts_populated(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO metadata (key, value)
         VALUES ('fts_populated', 'true')
         ON CONFLICT(key) DO UPDATE SET value = 'true'",
        (),
    )?;

    Ok(())
}
