use rusqlite::{Connection, Result};

//connection creation
pub fn connect() -> Result<Connection> {
    let conn = Connection::open("indexed-files.db")?;

    // Enable foreign-key enforcement for this connection
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    Ok(conn)
}
