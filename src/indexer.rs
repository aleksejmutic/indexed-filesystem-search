use crate::config::loader::load_config;
use crate::database::connection::{connect, create_schema};
use crate::database::repository::insert_file_entry;
use crate::filesystem::scanner::scan;

pub fn index_filesystem() -> rusqlite::Result<()> {
    let conn = connect()?; //create a database connection

    create_schema(&conn)?; //create the schema

    let exclusions = load_config().expect("Failed to load configuration");

    let entries = scan(&exclusions).expect("Filesystem scan failed");

    for entry in entries {
        insert_file_entry(&conn, &entry)?;
    }

    Ok(())
}
