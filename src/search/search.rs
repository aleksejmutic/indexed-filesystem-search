use crate::database::repository::search_file_entries;
use crate::filesystem::entry::FileEntry;
use crate::search::query::build_fts_query;
use rusqlite::Connection;

//search function that creates a query and passes it to SQLite search
pub fn search(conn: &Connection, query: &str, limit: i64) -> rusqlite::Result<Vec<FileEntry>> {
    let fts_query = build_fts_query(query);

    search_file_entries(conn, &fts_query, limit)
}
