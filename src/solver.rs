use std::collections::{HashMap, HashSet};

use crate::clue::{matches_clue, Clue};

const WORDS: &str = include_str!("data/words.txt");

pub fn load_words() -> Vec<String> {
    WORDS.lines().map(|w| w.trim().to_lowercase()).filter(|w| w.len() == 5).collect()
}

/// Filter candidate words based on all accumulated clues.
pub fn filter_candidates(candidates: &[String], clues: &[Clue]) -> Vec<String> {
    candidates
        .iter()
        .filter(|word| matches_all_clues(word, clues))
        .cloned()
        .collect()
}

fn matches_all_clues(word: &str, clues: &[Clue]) -> bool {
    let chars: Vec<char> = word.chars().collect();
    for clue in clues {
        if !matches_clue(&chars, clue) {
            return false;
        }
    }
    true
}

/// Score a word by summing per-position letter frequencies in the candidate pool.
/// Higher score = covers more common letters = better at narrowing candidates.
fn score_word(word: &str, freq: &HashMap<char, usize>) -> usize {
    let unique: HashSet<char> = word.chars().collect();
    unique.iter().map(|c| freq.get(c).copied().unwrap_or(0)).sum()
}

pub fn suggest_next(candidates: &[String], all_words: &[String]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(candidates[0].clone());
    }

    // Build letter frequency across remaining candidates
    let mut freq: HashMap<char, usize> = HashMap::new();
    for word in candidates {
        for c in word.chars() {
            *freq.entry(c).or_insert(0) += 1;
        }
    }

    // For small candidate pools only search candidates; otherwise search all words
    // so we can find a guess that maximally splits the space even if it's not a candidate.
    let search_pool: &[String] = if candidates.len() <= 6 { candidates } else { all_words };

    search_pool
        .iter()
        .max_by_key(|w| score_word(w, &freq))
        .cloned()
}

pub fn is_list_command(input: &str) -> bool {
    input.eq_ignore_ascii_case("list") || input == "?"
}

pub fn format_candidates(candidates: &[String]) -> String {
    let mut output = format!("Remaining candidates ({}):", candidates.len());
    for (i, word) in candidates.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            output.push('\n');
        }
        output.push_str(&format!("  {}", word.to_uppercase()));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clue::Clue;

    #[test]
    fn test_load_words() {
        let words = load_words();
        assert!(!words.is_empty());
        assert!(words.iter().all(|w| w.len() == 5));
    }

    #[test]
    fn test_filter_green() {
        let words = vec![
            "stone".to_string(),
            "slick".to_string(),
            "crane".to_string(),
        ];
        let clue = Clue::parse("swxyz", "G____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(result.contains(&"stone".to_string()));
        assert!(result.contains(&"slick".to_string()));
        assert!(!result.contains(&"crane".to_string()));
    }

    #[test]
    fn test_filter_gray_removes_words_with_letter() {
        let words = vec![
            "crane".to_string(),
            "swift".to_string(),
            "stole".to_string(),
        ];
        let clue = Clue::parse("crane", "_____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(!result.contains(&"crane".to_string()));
        assert!(result.contains(&"swift".to_string()));
        assert!(!result.contains(&"stole".to_string()));
    }

    #[test]
    fn test_filter_yellow_wrong_position() {
        let words = vec![
            "crane".to_string(),
            "acorn".to_string(),
            "raced".to_string(),
        ];
        let clue = Clue::parse("rbxxx", "Y____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(!result.contains(&"raced".to_string()));
        assert!(result.contains(&"crane".to_string()));
        assert!(result.contains(&"acorn".to_string()));
    }

    #[test]
    fn test_filter_yellow_letter_must_be_present() {
        let words = vec![
            "stone".to_string(),
            "built".to_string(),
            "crane".to_string(),
            "towel".to_string(),
        ];
        let clue = Clue::parse("txxxx", "Y____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(result.contains(&"stone".to_string()));
        assert!(result.contains(&"built".to_string()));
        assert!(!result.contains(&"crane".to_string()));
        assert!(!result.contains(&"towel".to_string()));
    }

    #[test]
    fn test_duplicate_letter_gray_exact_count() {
        let words = vec!["snowy".to_string(), "built".to_string(), "speed".to_string()];
        let clue = Clue::parse("eerie", "_____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(result.contains(&"snowy".to_string()));
        assert!(!result.contains(&"built".to_string()));
        assert!(!result.contains(&"speed".to_string()));
    }

    #[test]
    fn test_duplicate_letter_green_and_gray_exact_count() {
        let words = vec![
            "erode".to_string(),
            "error".to_string(),
            "groan".to_string(),
        ];
        let clue = Clue::parse("erase", "GG___").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(!result.contains(&"erode".to_string()));
        assert!(result.contains(&"error".to_string()));
        assert!(!result.contains(&"groan".to_string()));
    }

    #[test]
    fn test_multiple_clues_accumulated() {
        let all_words = load_words();
        let clue1 = Clue::parse("raise", "_____").unwrap();
        let clue2 = Clue::parse("clout", "G____").unwrap();
        let candidates = filter_candidates(&all_words, &[clue1, clue2]);

        for word in &candidates {
            let chars: Vec<char> = word.chars().collect();
            assert_eq!(chars[0], 'c', "word '{word}' doesn't start with 'c'");
            for banned in ['r', 'a', 'i', 's', 'e', 'l', 'o', 'u', 't'] {
                assert!(!word.contains(banned), "word '{word}' contains banned letter '{banned}'");
            }
        }
        assert!(candidates.len() < all_words.len());
    }

    #[test]
    fn test_full_solve_scenario() {
        let all_words = load_words();
        let mut candidates = all_words.clone();

        let clue1 = Clue::parse("brick", "_____").unwrap();
        candidates = filter_candidates(&candidates, &[clue1]);
        assert!(!candidates.contains(&"brick".to_string()));
        assert!(candidates.contains(&"stole".to_string()));
        assert!(candidates.len() < all_words.len());
    }

    #[test]
    fn test_solve_converges() {
        let all_words = load_words();
        let mut candidates = all_words.clone();

        let c1 = Clue::parse("crane", "____G").unwrap();
        candidates = filter_candidates(&candidates, &[c1]);
        assert!(candidates.contains(&"stove".to_string()), "stove should survive round 1");
        let count1 = candidates.len();

        let c2 = Clue::parse("stomp", "GGG__").unwrap();
        candidates = filter_candidates(&candidates, &[c2]);
        assert!(candidates.contains(&"stove".to_string()), "stove should survive round 2");
        assert!(candidates.len() < count1, "candidates should decrease");
    }

    #[test]
    fn test_suggest_next_single_candidate() {
        let all_words = load_words();
        let candidates = vec!["crane".to_string()];
        let suggestion = suggest_next(&candidates, &all_words);
        assert_eq!(suggestion, Some("crane".to_string()));
    }

    #[test]
    fn test_suggest_next_returns_candidate() {
        let all_words = load_words();
        let suggestion = suggest_next(&all_words, &all_words);
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert_eq!(s.len(), 5);
        assert!(all_words.contains(&s));
    }

    #[test]
    fn test_suggest_next_empty_candidates() {
        let all_words = load_words();
        let candidates: Vec<String> = vec![];
        assert_eq!(suggest_next(&candidates, &all_words), None);
    }

    #[test]
    fn test_is_list_command() {
        assert!(is_list_command("list"));
        assert!(is_list_command("LIST"));
        assert!(is_list_command("List"));
        assert!(is_list_command("?"));
        assert!(!is_list_command("crane"));
        assert!(!is_list_command(""));
        assert!(!is_list_command("lists"));
        assert!(!is_list_command("lis"));
    }

    #[test]
    fn test_format_candidates_empty() {
        let candidates: Vec<String> = vec![];
        let output = format_candidates(&candidates);
        assert_eq!(output, "Remaining candidates (0):");
    }

    #[test]
    fn test_format_candidates_few() {
        let candidates = vec!["stone".to_string(), "stove".to_string(), "stoke".to_string()];
        let output = format_candidates(&candidates);
        assert!(output.starts_with("Remaining candidates (3):"));
        assert!(output.contains("STONE"));
        assert!(output.contains("STOVE"));
        assert!(output.contains("STOKE"));
    }

    #[test]
    fn test_format_candidates_wraps_at_10() {
        let candidates: Vec<String> = (0..12)
            .map(|i| format!("word{}", (b'a' + i as u8) as char))
            .collect();
        let output = format_candidates(&candidates);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
    }
}
