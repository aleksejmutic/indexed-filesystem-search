use crate::database::connection::{connect, create_schema};
use crate::database::repository::insert_file_entry;
use crate::filesystem::scanner::scan;

pub fn index_filesystem() -> rusqlite::Result<()> {
    let conn = connect()?; //create a database connection

    create_schema(&conn)?; //create the schema

    let entries = scan().expect("Filesystem scan failed");
    /*
        expect uses Debug to format errors, It gives useful information when the Result
        contains an error, which is why ScanError needs to implement Debug.
    */
    for entry in entries {
        insert_file_entry(&conn, &entry)?;
    }

    Ok(())
}
