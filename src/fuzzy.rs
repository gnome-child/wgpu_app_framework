#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Score(i32);

impl Score {
    pub(crate) fn get(self) -> i32 {
        self.0
    }
}

pub(crate) fn score(query: &str, candidate: &str) -> Option<Score> {
    let query = query.trim();
    if query.is_empty() {
        return Some(Score(0));
    }

    let query_lower = query.to_lowercase();
    let candidate_lower = candidate.to_lowercase();
    if candidate_lower == query_lower {
        return Some(Score(1_000_000 - candidate_lower.chars().count() as i32));
    }

    if let Some(start) = candidate_lower.find(&query_lower) {
        return Some(Score(
            900_000 - start as i32 * 20 - candidate_lower.chars().count() as i32,
        ));
    }

    subsequence_score(&query_lower, candidate)
}

fn subsequence_score(query: &str, candidate: &str) -> Option<Score> {
    let candidate_lower = candidate.to_lowercase();
    let candidate_chars = candidate_lower.chars().collect::<Vec<_>>();
    let original_chars = candidate.chars().collect::<Vec<_>>();
    let query_chars = query.chars().collect::<Vec<_>>();
    let mut candidate_index = 0;
    let mut positions = Vec::with_capacity(query_chars.len());

    for query_char in query_chars {
        let found = candidate_chars
            .iter()
            .enumerate()
            .skip(candidate_index)
            .find_map(|(index, candidate_char)| (*candidate_char == query_char).then_some(index))?;
        positions.push(found);
        candidate_index = found + 1;
    }

    let mut score = 100_000;
    let mut previous = None;
    for position in positions {
        if let Some(previous) = previous {
            let gap = position.saturating_sub(previous + 1) as i32;
            if gap == 0 {
                score += 4_000;
            } else {
                score -= gap * 80;
            }
        }

        if is_word_start(&original_chars, position) {
            score += 2_000;
        }
        score -= position as i32 * 4;
        previous = Some(position);
    }

    score -= candidate_chars.len() as i32;
    Some(Score(score))
}

fn is_word_start(chars: &[char], index: usize) -> bool {
    if index == 0 {
        return true;
    }

    let previous = chars[index - 1];
    let current = chars[index];
    !previous.is_alphanumeric()
        || (previous.is_lowercase() && current.is_uppercase())
        || matches!(previous, '_' | '-' | '.' | '/' | '\\')
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn exact_matches_beat_substrings() {
        assert!(score("undo", "undo") > score("undo", "undo history"));
    }

    #[test]
    fn substrings_beat_sparse_subsequences() {
        assert!(score("und", "Undo") > score("und", "Update Node"));
    }

    #[test]
    fn word_starts_are_rewarded() {
        assert!(score("sf", "Save File") > score("sf", "surface"));
    }

    #[test]
    fn command_name_fallback_can_match_paths() {
        assert!(score("eo", "edit.open").is_some());
    }

    #[test]
    fn rejects_non_matches() {
        assert!(score("xyz", "Undo").is_none());
    }
}
