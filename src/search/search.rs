use crate::database::repository::search_file_entries;
use crate::filesystem::entry::FileEntry;
use crate::search::query::{SearchStrategy, build_fts_query, determine_strategy};
use rusqlite::Connection;

//search function that creates a query and passes it to SQLite search, Single and HardAnd actually do the same thing looking at how the database is searched
pub fn search(conn: &Connection, query: &str, limit: i64) -> rusqlite::Result<Vec<FileEntry>> {
    let strategy = determine_strategy(query);

    match strategy {
        None => Ok(Vec::new()),

        Some(SearchStrategy::Single) => {
            let fts_query = build_fts_query(query);
            search_file_entries(conn, &fts_query, limit)
        }
        // this strategy basically takes two tokens from the search and tried a HardAnd implementation, and if nothing is found,
        // it falls back to searching each token separately, first token, then second token, with also checking for duplicates in the search
        Some(SearchStrategy::SoftAnd) => {
            let tokens: Vec<&str> = query.split_whitespace().collect();

            let exact_query = format!("{}* {}*", tokens[0], tokens[1]);

            let mut results = search_file_entries(conn, &exact_query, limit)?;

            if results.len() < limit as usize {
                let first_query = format!("{}*", tokens[0]);
                let second_query = format!("{}*", tokens[1]);

                let first_results = search_file_entries(conn, &first_query, limit)?;
                let second_results = search_file_entries(conn, &second_query, limit)?;

                let mut first_index = 0;
                let mut second_index = 0;

                while results.len() < limit as usize
                    && (first_index < first_results.len() || second_index < second_results.len())
                {
                    if first_index < first_results.len() {
                        let entry = &first_results[first_index];

                        if !results.iter().any(|existing| existing.path == entry.path) {
                            results.push(entry.clone()); //I am cloning since FileEntry is already owned, but I can own a clone
                        }

                        first_index += 1;
                    }

                    if results.len() >= limit as usize {
                        break;
                    }

                    if second_index < second_results.len() {
                        let entry = &second_results[second_index];

                        if !results.iter().any(|existing| existing.path == entry.path) {
                            results.push(entry.clone());
                        }

                        second_index += 1;
                    }
                }
            }

            Ok(results)
        }

        Some(SearchStrategy::HardAnd) => {
            let fts_query = build_fts_query(query);
            search_file_entries(conn, &fts_query, limit)
        }
    }
}
