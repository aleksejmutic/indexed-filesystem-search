use rusqlite::{Connection, Result};

//connection creation
pub fn connect() -> Result<Connection> {
    let conn = Connection::open("indexed-files.db")?;

    Ok(conn)
}
