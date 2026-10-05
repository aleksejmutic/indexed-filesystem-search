use crate::test::test_watcher;

mod config;
mod database;
mod filesystem;
mod indexer;
mod search;
mod synchronization;
mod test;

fn main() -> rusqlite::Result<()> {
    //indexer::sync_filesystem()?;

    //test_search()?;

    //test_sync()?;

    test_watcher();

    Ok(())
}
