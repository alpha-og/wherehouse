mod search_pane;
mod search_results_pane;

pub enum Widget {
    SearchPane(search_pane::SearchPane),
    SearchResultsPane(search_results_pane::ListPane),
}
