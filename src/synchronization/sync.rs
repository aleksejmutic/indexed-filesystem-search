use crate::database::repository::{
    get_file_entry_by_path, insert_file_entry, update_file_entry, update_last_seen,
};
use crate::filesystem::entry::FileEntry;
use rusqlite::Connection;

// describes the results of synchronization operations
pub enum SyncResult {
    Unchanged,
    Inserted,
    Updated,
}

pub fn sync_file_entry(
    conn: &Connection,
    entry: &FileEntry,
    scan_id: i64,
) -> rusqlite::Result<SyncResult> {
    let path = entry.path.to_string_lossy().to_string();

    // different enum values for SyncResult are returned depending on the state of the filesystem and corresponding database, or to be precise FTS index
    match get_file_entry_by_path(conn, &path)? {
        None => {
            insert_file_entry(conn, entry, scan_id)?;
            Ok(SyncResult::Inserted)
        }

        Some((id, existing_entry)) => {
            //             println!("New entry:");
            //             println!("  path: {}", entry.path.display());
            //             println!("  size: {}", entry.size);
            //             println!("  modified: {:?}", entry.modified);
            //
            //             println!("Existing entry:");
            //             println!("  path: {}", existing_entry.path.display());
            //             println!("  size: {}", existing_entry.size);
            //             println!("  modified: {:?}", existing_entry.modified);

            if existing_entry == *entry {
                // File exists and hasn't changed.
                // We still need to mark it as seen in this scan.
                update_last_seen(conn, id, scan_id)?;

                Ok(SyncResult::Unchanged)
            } else {
                update_file_entry(conn, id, entry, scan_id)?;

                Ok(SyncResult::Updated)
            }
        }
    }
}
