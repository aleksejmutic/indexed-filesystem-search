mod database;
mod filesystem;

use crate::database::connection::connect;
use crate::database::connection::create_schema;
use crate::filesystem::scanner::scan;

fn main() -> rusqlite::Result<()> {
    let conn = connect()?;

    create_schema(&conn)?;

    Ok(())
}
