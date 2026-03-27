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
        let mut w = ['a'; 5];
        for (i, c) in word.to_lowercase().chars().enumerate() {
            if !c.is_ascii_alphabetic() {
                return Err(format!("Word contains non-alphabetic character '{}'", c));
            }
            w[i] = c;
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

pub fn matches_clue(word: &[char], clue: &Clue, constraints: &LetterConstraints) -> bool {
    let mut letter_counts: HashMap<char, usize> = HashMap::new();
    for &c in word {
        *letter_counts.entry(c).or_insert(0) += 1;
    }

    matches_positions(word, clue) && matches_counts(&letter_counts, constraints)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clue_parse_valid() {
        let clue = Clue::parse("crane", "G_Y__").unwrap();
        assert_eq!(clue.word, ['c', 'r', 'a', 'n', 'e']);
        assert_eq!(
            clue.hints,
            [Hint::Green, Hint::Gray, Hint::Yellow, Hint::Gray, Hint::Gray]
        );
    }

    #[test]
    fn test_clue_parse_invalid_length() {
        assert!(Clue::parse("cran", "G___").is_err());
        assert!(Clue::parse("crane", "G___").is_err());
    }

    #[test]
    fn test_clue_parse_case_insensitive() {
        let clue = Clue::parse("CRANE", "g_y__").unwrap();
        assert_eq!(clue.word, ['c', 'r', 'a', 'n', 'e']);
        assert_eq!(clue.hints[0], Hint::Green);
        assert_eq!(clue.hints[2], Hint::Yellow);
    }

    #[test]
    fn test_clue_parse_invalid_char_in_word() {
        assert!(Clue::parse("cr4ne", "G____").is_err());
        assert!(Clue::parse("cr ne", "G____").is_err());
    }

    #[test]
    fn test_clue_parse_invalid_feedback_char() {
        assert!(Clue::parse("crane", "G_X__").is_err());
        assert!(Clue::parse("crane", "G_1__").is_err());
    }
}
