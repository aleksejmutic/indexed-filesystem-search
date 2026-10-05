use crate::test::test_watcher;

mod config;
mod database;
mod filesystem;
mod indexer;
mod search;
mod synchronization;
mod test;

// as of now the return type is a Result from notify, not a Result from rusqlite, important!!!
fn main() -> notify::Result<()> {
    //indexer::sync_filesystem()?;

    //test_search()?;

    //test_sync()?;

    test_watcher()?;

    Ok(())
}
