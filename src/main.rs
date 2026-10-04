mod config;
mod database;
mod filesystem;
mod indexer;
mod search;

use crate::database::connection::connect;
use crate::search::search::search;

fn main() -> rusqlite::Result<()> {
    //indexer::sync_filesystem()?;

    test_search()?;

    Ok(())
}

fn test_search() -> rusqlite::Result<()> {
    let conn = connect()?;

    println!("___ sqlite firefox ___");
    for entry in search(&conn, "sqlite firefox", 10)? {
        println!("{}", entry.path.display());
    }

    println!("___ fire ___");
    for entry in search(&conn, "fire", 10)? {
        println!("{}", entry.path.display());
    }

    println!("___ sqlite fire ___");
    for entry in search(&conn, "sqlite fire", 10)? {
        println!("{}", entry.path.display());
    }

    println!("___ fire xyzabc ___");
    for entry in search(&conn, "fire xyzabc", 10)? {
        println!("{}", entry.path.display());
    }

    Ok(())
}
