mod config;
mod database;
mod filesystem;
mod indexer;
mod search;
mod synchronization;

use crate::database::connection::connect;
use crate::search::query::build_fts_query;
use crate::search::search::search;

fn main() -> rusqlite::Result<()> {
    //indexer::sync_filesystem()?;

    // test_search()?;

    test_sync()?;

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

    println!("___ fire xyzabc memes .pdf___");
    for entry in search(&conn, "fire xyzabc memes .pdf", 10)? {
        println!("{}", entry.path.display());
    }

    println!("___ FTS query ___");
    println!("{}", build_fts_query("fire xyzabc memes .pdf"));

    println!("___ .pdf ___");
    for entry in search(&conn, ".pdf", 10)? {
        println!("{}", entry.path.display());
    }

    println!("{}", build_fts_query("hello (test"));
    println!("___ hello (test ___");
    for entry in search(&conn, "hello (test", 10)? {
        println!("{}", entry.path.display());
    }

    println!("___ foo:bar ___");
    for entry in search(&conn, "foo:bar", 10)? {
        println!("{}", entry.path.display());
    }

    Ok(())
}

// function to test syncing
fn test_sync() -> rusqlite::Result<()> {
    let conn = connect()?;

    let path = "/home/alexei/message.txt";

    // We need a FileEntry representing the filesystem state here.
    // For the first test, get one from the database and use it as the
    // filesystem entry.

    let (_, entry) = crate::database::repository::get_file_entry_by_path(&conn, path)?
        .expect("File should already exist in database");

    let result = crate::synchronization::sync::sync_file_entry(&conn, &entry, 999999)?;

    println!(
        "Synchronization result: {}",
        match result {
            crate::synchronization::sync::SyncResult::Unchanged => "Unchanged",
            crate::synchronization::sync::SyncResult::Inserted => "Inserted",
            crate::synchronization::sync::SyncResult::Updated => "Updated",
        }
    );

    Ok(())
}
