mod config;
mod database;
mod filesystem;
mod indexer;

use crate::database::connection::connect;
use crate::database::repository::search_file_entries;

fn main() -> rusqlite::Result<()> {
    //indexer::sync_filesystem()?;

    test_search()?;

    Ok(())
}

fn test_search() -> rusqlite::Result<()> {
    let conn = connect()?;

    println!("___ sqlite firefox ___");
    let entries = search_file_entries(&conn, "sqlite firefox", 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    println!("___ fire* ___");
    let entries = search_file_entries(&conn, "fire*", 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    println!("___ sqlite fire* ___");
    let entries = search_file_entries(&conn, "sqlite fire*", 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    println!("___ fire* xyzabc* ___");
    let entries = search_file_entries(&conn, "fire* xyzabc*", 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    Ok(())
}
