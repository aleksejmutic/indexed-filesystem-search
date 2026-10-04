mod config;
mod database;
mod filesystem;
mod indexer;
mod search;

use crate::database::connection::connect;
use crate::database::repository::search_file_entries;
use crate::search::query::build_fts_query;

fn main() -> rusqlite::Result<()> {
    //indexer::sync_filesystem()?;

    test_search()?;

    Ok(())
}

fn test_search() -> rusqlite::Result<()> {
    let conn = connect()?;

    println!("___ sqlite firefox ___");
    let query = build_fts_query("sqlite firefox");
    let entries = search_file_entries(&conn, &query, 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    println!("___ fire* ___");
    let query = build_fts_query("fire");
    let entries = search_file_entries(&conn, &query, 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    println!("___ sqlite fire* ___");
    let query = build_fts_query("sqlite fire");
    let entries = search_file_entries(&conn, &query, 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    println!("___ fire* xyzabc* ___");
    let query = build_fts_query("fire xyzabc");
    let entries = search_file_entries(&conn, &query, 10)?;
    for entry in entries {
        println!("{}", entry.path.display());
    }

    Ok(())
}
