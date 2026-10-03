mod config;
mod database;
mod filesystem;
mod indexer;

use crate::database::connection::connect;
use crate::database::repository::search_file_entries;

fn main() -> rusqlite::Result<()> {
    indexer::sync_filesystem()?;

    //test_search()?;

    Ok(())
}

fn test_search() -> rusqlite::Result<()> {
    let conn = connect()?;

    let entries = search_file_entries(&conn, "firefox", 10)?;

    for entry in entries {
        println!("{}", entry.path.display());
    }

    Ok(())
}
