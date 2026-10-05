use crate::config::loader::load_config;
use crate::database::connection::{connect, create_schema};
use crate::database::fts::{is_fts_populated, mark_fts_populated, populate_fts};
use crate::filesystem::scanner::scan;
use crate::synchronization::sync::sync_file_entry;
use std::time::Instant;

pub fn sync_filesystem() -> rusqlite::Result<()> {
    let mut conn = connect()?; //create a database connection

    create_schema(&conn)?; //create the schema

    let exclusions = load_config().expect("Failed to load configuration");

    println!("Starting filesystem scan...");

    let scan_start = Instant::now();

    let entries = scan(&exclusions).expect("Filesystem scan failed");

    println!("Scan finished. Found {} entries.", entries.len());

    println!("Filesystem scan took: {:?}", scan_start.elapsed());

    println!("Starting database insertion...");

    let transaction = conn.transaction()?;

    //creates a new scan_id based on the previous one
    let scan_id: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(last_seen), 0) + 1 FROM file_entries",
        [],
        |row| row.get(0),
    )?;

    let sync_start = Instant::now();

    for (i, entry) in entries.iter().enumerate() {
        sync_file_entry(&transaction, entry, scan_id)?;

        if i % 1000 == 0 {
            println!("Synchronized {} entries...", i);
        }
    }

    println!("Synchronization took: {:?}", sync_start.elapsed());

    let delete_start = Instant::now();

    // Remove files that were not encountered during this scan
    transaction.execute("DELETE FROM file_entries WHERE last_seen != ?1", [scan_id])?;

    println!("Deletion took: {:?}", delete_start.elapsed());

    transaction.commit()?;

    println!("Database insertion finished.");

    //populating the fts5 index, done after the transaction, as file_entries table needs to finish its work, and then fts5 index comes into play
    if !is_fts_populated(&conn)? {
        println!("Populating FTS5 index...");
        populate_fts(&conn)?;
        mark_fts_populated(&conn)?;
        println!("FTS5 population finished.");
    }

    Ok(())
}
