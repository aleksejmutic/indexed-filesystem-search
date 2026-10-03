use crate::config::loader::load_config;
use crate::database::connection::{connect, create_schema};
use crate::database::repository::insert_file_entry;
use crate::filesystem::scanner::scan;

pub fn index_filesystem() -> rusqlite::Result<()> {
    let mut conn = connect()?; //create a database connection

    create_schema(&conn)?; //create the schema

    let exclusions = load_config().expect("Failed to load configuration");

    println!("Starting filesystem scan...");

    let entries = scan(&exclusions).expect("Filesystem scan failed");

    println!("Scan finished. Found {} entries.", entries.len());

    println!("Starting database insertion...");

    let transaction = conn.transaction()?;

    //creates a new scan_id based on the previous one
    let scan_id: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(last_seen), 0) + 1 FROM file_entries",
        [],
        |row| row.get(0),
    )?;

    for (i, entry) in entries.iter().enumerate() {
        insert_file_entry(&transaction, entry, scan_id)?;

        if i % 1000 == 0 {
            println!("Inserted {} entries...", i);
        }
    }

    transaction.commit()?;

    println!("Database insertion finished.");

    Ok(())
}
