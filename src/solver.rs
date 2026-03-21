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
