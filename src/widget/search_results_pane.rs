pub struct SearchResultsPane {
    id: u8,
    items: Vec<SearchResult>,
    selected: usize,
    active: bool,
}

struct SearchResult {
    display_text: String,
}
