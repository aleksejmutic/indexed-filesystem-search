mod config;
mod database;
mod filesystem;
mod indexer;
mod search;

use crate::database::connection::connect;
use crate::search::query::build_fts_query;
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
