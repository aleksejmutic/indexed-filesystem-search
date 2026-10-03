use rusqlite::{Connection, Result};

fn connect() -> Result<Connection> {
    let conn = Connection::open("indexed-files.db")?; //connection creation

    Ok(conn)
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "create table if not exists file_entries (
            id integer primary key,
            path text not null unique,
            filename text,
            extension text,
            size integer not null,
            modified integer not null,
            is_directory integer not null,
            is_hidden integer not null
        )",
        (),
    )?; //schema creation

    Ok(())
}
