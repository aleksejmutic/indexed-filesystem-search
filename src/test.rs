use crate::config::loader::load_config;
use crate::database::connection::{connect, create_schema};
use crate::filesystem;
use crate::filesystem::scanner::scan;
use crate::search::query::build_fts_query;
use crate::search::search::search;

pub fn test_search() -> rusqlite::Result<()> {
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
pub fn test_sync() -> rusqlite::Result<()> {
    let conn = connect()?;

    create_schema(&conn)?;

    let path = "/home/alexei/message.txt";

    let exclusions = load_config().expect("Failed to load configuration");

    let entries = scan(&exclusions).expect("Filesystem scan failed");

    let entry = entries
        .into_iter()
        .find(|entry| entry.path.to_string_lossy() == path)
        .expect("File should exist on filesystem");

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

pub fn test_watcher() -> notify::Result<()> {
    let home = std::env::home_dir().expect("Could not find home directory");
    filesystem::watcher::watch(&home)?;
    Ok(())
}
