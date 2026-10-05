mod config;
mod database;
mod filesystem;
mod indexer;
mod search;
mod synchronization;
mod test;
use crate::test::{test_search, test_sync};

fn main() -> rusqlite::Result<()> {
    indexer::sync_filesystem()?;

    //test_search()?;

    //test_sync()?;

    Ok(())
}
