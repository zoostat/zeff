//! "Did you mean ...?": the candidate closest to a word that was not found.

/// The distance between two words: the fewest single-character insertions, deletions,
/// substitutions and swaps of neighbouring characters that turn one into the other.
fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous_previous: Vec<usize> = vec![0; b.len() + 1];
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut current = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            current[j] = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                current[j] = current[j].min(previous_previous[j - 2] + 1);
            }
        }
        previous_previous = previous;
        previous = current;
    }
    previous[b.len()]
}

/// The candidate that is the best guess for what `word` was meant to be, if any is close enough:
/// the same letters in another case first, then the nearest by distance, within a third of the
/// word's length (at least one change). Ties go to the candidate listed first.
pub fn did_you_mean<'a>(
    word: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let candidates: Vec<&str> = candidates.into_iter().collect();
    if let Some(same) = candidates
        .iter()
        .find(|c| **c != word && c.eq_ignore_ascii_case(word))
    {
        return Some(same);
    }
    let limit = (word.chars().count().max(3)) / 3;
    candidates
        .into_iter()
        .filter(|c| *c != word)
        .map(|c| (distance(word, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_counts_edits_and_swaps() {
        assert_eq!(distance("income", "income"), 0);
        assert_eq!(distance("incme", "income"), 1);
        assert_eq!(distance("incoem", "income"), 1);
        assert_eq!(distance("netwrk", "network"), 1);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("kitten", "sitting"), 3);
    }

    #[test]
    fn the_nearest_candidate_within_reach_is_suggested() {
        let columns = ["income", "age", "wage"];
        assert_eq!(did_you_mean("incme", columns), Some("income"));
        assert_eq!(did_you_mean("Wage", columns), Some("wage"));
        assert_eq!(did_you_mean("xyz", columns), None);
        assert_eq!(
            did_you_mean("income", columns),
            None,
            "the word itself is not a suggestion"
        );
    }

    #[test]
    fn the_same_letters_in_another_case_win() {
        assert_eq!(
            did_you_mean("Edition", ["editions", "edition"]),
            Some("edition")
        );
    }

    #[test]
    fn a_short_word_needs_a_close_match() {
        assert_eq!(did_you_mean("ab", ["abc", "xyz"]), Some("abc"));
        assert_eq!(did_you_mean("ab", ["xyz"]), None);
    }
}
