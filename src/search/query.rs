//a query builder function that basically splits at whitespaces and does matching, outputs a format which FTS5 wants to use
pub fn build_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|token| format!("{}*", token))
        .collect::<Vec<_>>()
        .join(" ")
}
// three search strategies which are used like this
// one string just searches with that given string
// two strings firstly searches for exact file that has both strings, and then falls back to showing either/or
// everything after three basically assumes no fallbacks to specific string slices, it expects a perfect intersection of string slices
pub enum SearchStrategy {
    Single,
    SoftAnd,
    HardAnd,
}
//determines the search strategy based on number of tokens
pub fn determine_strategy(query: &str) -> Option<SearchStrategy> {
    let token_count = query.split_whitespace().count();

    match token_count {
        0 => None, //in case nothing is typed at all in the search
        1 => Some(SearchStrategy::Single),
        2 => Some(SearchStrategy::SoftAnd),
        _ => Some(SearchStrategy::HardAnd),
    }
}
