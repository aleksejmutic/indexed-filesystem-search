use crate::test::test_sync;
use crate::test::test_watcher;

mod config;
mod database;
mod filesystem;
mod indexer;
mod search;
mod synchronization;
mod test;

// as of now the return type is a Result from notify, not a Result from rusqlite, important!!!
// I am changing this depending on the call, sync_filesystem() expects a rusqlite Result, while test_watcher() expects a notify Result
fn main() -> notify::Result<()> {
    // this is important to see how performance has changed, at least the prints do xd
    //indexer::sync_filesystem()?;

    //test_search()?;

    //test_sync()?;

    test_watcher()?;

    Ok(())
}
