use std::cmp::min;

pub mod package_manager;

pub fn fuzz<I>(word_list: I, query: String, threshold: usize) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    word_list
        .into_iter()
        .filter(|word| levenshtein_distance(word, &query) < threshold)
        .collect()
}

fn levenshtein_distance(s: &str, t: &str) -> usize {
    if s.is_empty() {
        return t.len();
    } else if t.is_empty() {
        return s.len();
    }
    let mut cost = 1;

    if s.chars().nth(s.len() - 1) == t.chars().nth(t.len() - 1) {
        cost = 0
    }

    // let s_last = match s.chars().nth(s.len() - 1) {
    //     Some(last) => last,
    //     None => return t.len(),
    // };
    // let t_last = match t.chars().nth(t.len() - 1) {
    //     Some(last) => last,
    //     None => return s.len(),
    // };
    //
    // if s_last == t_last {
    //     cost = 0;
    // }
    min(
        min(
            levenshtein_distance(&s[..s.len() - 1], t) + 1,
            levenshtein_distance(s, &t[..t.len() - 1]) + 1,
        ),
        levenshtein_distance(&s[..s.len() - 1], &t[..t.len() - 1]) + cost,
    )
}
