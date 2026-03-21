use std::collections::HashMap;

/// Wordle feedback for a single character position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hint {
    Green,  // correct letter, correct position
    Yellow, // correct letter, wrong position
    Gray,   // letter not in word
}

/// A single guess + feedback pair.
pub struct Clue {
    pub word: [char; 5],
    pub hints: [Hint; 5],
}

impl Clue {
    pub fn parse(word: &str, feedback: &str) -> Result<Self, String> {
        if word.len() != 5 {
            return Err(format!("Word must be 5 letters, got '{}'", word));
        }
        if feedback.len() != 5 {
            return Err(format!("Feedback must be 5 chars (G/Y/_), got '{}'", feedback));
        }
        let word_chars: Vec<char> = word.to_lowercase().chars().collect();
        let mut w = ['a'; 5];
        for (i, c) in word_chars.iter().enumerate() {
            if !c.is_ascii_alphabetic() {
                return Err(format!("Word contains non-alphabetic character '{}'", c));
            }
            w[i] = *c;
        }
        let mut hints = [Hint::Gray; 5];
        for (i, c) in feedback.to_uppercase().chars().enumerate() {
            hints[i] = match c {
                'G' => Hint::Green,
                'Y' => Hint::Yellow,
                '_' => Hint::Gray,
                other => return Err(format!("Invalid feedback char '{}', use G/Y/_", other)),
            };
        }
        Ok(Clue { word: w, hints })
    }
}

/// Per-letter constraint derived from a clue: (min_count, has_gray).
pub type LetterConstraints = HashMap<char, (usize, bool)>;

/// Build per-letter constraints from a clue.
/// - green/yellow => letter appears at least N times
/// - gray mixed with green/yellow => letter appears exactly N times
/// - all gray => letter does not appear
pub fn build_letter_constraints(clue: &Clue) -> LetterConstraints {
    let mut info: LetterConstraints = HashMap::new();
    for i in 0..5 {
        let c = clue.word[i];
        let entry = info.entry(c).or_insert((0, false));
        match clue.hints[i] {
            Hint::Green | Hint::Yellow => entry.0 += 1,
            Hint::Gray => entry.1 = true,
        }
    }
    info
}

/// Check that each letter is (or is not) at the exact position the clue requires.
pub fn matches_positions(word: &[char], clue: &Clue) -> bool {
    for i in 0..5 {
        let c = clue.word[i];
        match clue.hints[i] {
            Hint::Green => {
                if word[i] != c {
                    return false;
                }
            }
            Hint::Yellow => {
                if word[i] == c {
                    return false; // must NOT be at this position
                }
            }
            Hint::Gray => {}
        }
    }
    true
}

/// Check that letter counts in the word satisfy the constraints.
pub fn matches_counts(letter_counts: &HashMap<char, usize>, constraints: &LetterConstraints) -> bool {
    for (c, (min, has_gray)) in constraints {
        let actual = *letter_counts.get(c).unwrap_or(&0);
        if *has_gray && *min == 0 {
            if actual > 0 {
                return false;
            }
        } else if *has_gray {
            if actual != *min {
                return false;
            }
        } else {
            if actual < *min {
                return false;
            }
        }
    }
    true
}

pub fn matches_clue(word: &[char], clue: &Clue) -> bool {
    let mut letter_counts: HashMap<char, usize> = HashMap::new();
    for &c in word {
        *letter_counts.entry(c).or_insert(0) += 1;
    }

    let constraints = build_letter_constraints(clue);
    matches_positions(word, clue) && matches_counts(&letter_counts, &constraints)
}
