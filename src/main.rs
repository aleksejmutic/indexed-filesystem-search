mod config;
mod database;
mod filesystem;
mod indexer;

fn main() -> rusqlite::Result<()> {
    indexer::index_filesystem()?;

    Ok(())
}
