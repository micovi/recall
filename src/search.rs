pub fn fuzzy_match(query: &str, target: &str) -> Option<i64> {
    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();

    if query_lower.is_empty() {
        return Some(0);
    }

    let mut score: i64 = 0;
    let mut qi = 0;
    let mut last_match: Option<usize> = None;

    for (ti, &tc) in target_lower.iter().enumerate() {
        if qi < query_lower.len() && tc == query_lower[qi] {
            qi += 1;
            score += 1;

            // Consecutive match bonus
            if ti > 0 && last_match == Some(ti - 1) {
                score += 2;
            }

            // Word boundary bonus
            if ti == 0 || !target_lower[ti - 1].is_alphanumeric() {
                score += 3;
            }

            // Exact prefix bonus
            if ti + 1 == qi {
                score += 2;
            }

            last_match = Some(ti);
        }
    }

    if qi == query_lower.len() {
        Some(score)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_scores_highest() {
        let exact = fuzzy_match("ls", "ls");
        let partial = fuzzy_match("ls", "list");
        assert!(exact > partial);
    }

    #[test]
    fn no_match_returns_none() {
        assert!(fuzzy_match("xyz", "ls").is_none());
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(fuzzy_match("", "anything").is_some());
    }
}
