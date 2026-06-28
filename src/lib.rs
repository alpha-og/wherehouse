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
    let s_vec: Vec<_> = s.chars().collect();
    let t_vec: Vec<_> = t.chars().collect();

    let s_len = s_vec.len();
    let t_len = t_vec.len();

    let mut dp: Vec<usize> = vec![0; s_len + 1];

    dp[0] = 0;

    for i in 1..s_len + 1 {
        dp[i] = dp[i - 1] + 1;
    }

    for i in 1..t_len + 1 {
        let mut diag = dp[0];
        dp[0] = i;
        for j in 1..s_len + 1 {
            let mut diag_cost: usize = 0;
            if s_vec[j - 1] != t_vec[i - 1] {
                diag_cost = 1;
            }
            let next_diag = dp[j];
            dp[j] = min(min(dp[j - 1] + 1, dp[j] + 1), diag + diag_cost);
            diag = next_diag;
        }
    }

    dp[s_len]
}
