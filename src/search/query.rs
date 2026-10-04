//a query builder function that basically splits at whitespaces and does matching, outputs a format which FTS5 wants to use
pub fn build_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|token| format!("{}*", token))
        .collect::<Vec<_>>()
        .join(" ")
}
