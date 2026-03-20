use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

const WORDS: &str = include_str!("data/words.txt");

fn load_words() -> Vec<String> {
    WORDS.lines().map(|w| w.trim().to_lowercase()).filter(|w| w.len() == 5).collect()
}

/// Wordle feedback for a single character position.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Hint {
    Green,  // correct letter, correct position
    Yellow, // correct letter, wrong position
    Gray,   // letter not in word
}

/// A single guess + feedback pair.
struct Clue {
    word: [char; 5],
    hints: [Hint; 5],
}

impl Clue {
    fn parse(word: &str, feedback: &str) -> Result<Self, String> {
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

/// Filter candidate words based on all accumulated clues.
fn filter_candidates(candidates: &[String], clues: &[Clue]) -> Vec<String> {
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

fn matches_clue(word: &[char], clue: &Clue) -> bool {
    // Build letter count map for the candidate word
    let mut letter_counts: HashMap<char, usize> = HashMap::new();
    for &c in word {
        *letter_counts.entry(c).or_insert(0) += 1;
    }

    // Compute per-letter constraints from the clue:
    // - green/yellow => letter appears at least N times
    // - gray mixed with green/yellow => letter appears exactly N times
    // - all gray => letter does not appear
    let mut clue_letter_info: HashMap<char, (usize, bool)> = HashMap::new(); // (min_count, has_gray)
    for i in 0..5 {
        let c = clue.word[i];
        let entry = clue_letter_info.entry(c).or_insert((0, false));
        match clue.hints[i] {
            Hint::Green | Hint::Yellow => entry.0 += 1,
            Hint::Gray => entry.1 = true,
        }
    }

    // Check exact position constraints
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

    // Check count constraints
    for (c, (min, has_gray)) in &clue_letter_info {
        let actual = *letter_counts.get(c).unwrap_or(&0);
        if *has_gray && *min == 0 {
            // Letter must not appear at all
            if actual > 0 {
                return false;
            }
        } else if *has_gray {
            // Letter appears exactly min times
            if actual != *min {
                return false;
            }
        } else {
            // Letter appears at least min times
            if actual < *min {
                return false;
            }
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

fn suggest_next(candidates: &[String], all_words: &[String]) -> Option<String> {
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

fn is_list_command(input: &str) -> bool {
    input.eq_ignore_ascii_case("list") || input == "?"
}

fn format_candidates(candidates: &[String]) -> String {
    let mut output = format!("Remaining candidates ({}):", candidates.len());
    for (i, word) in candidates.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            output.push('\n');
        }
        output.push_str(&format!("  {}", word.to_uppercase()));
    }
    output
}

fn print_banner() {
    println!("╔══════════════════════════════════╗");
    println!("║        Wordle Solver CLI         ║");
    println!("╚══════════════════════════════════╝");
    println!();
    println!("Feedback format: 5 chars, one per letter");
    println!("  G = Green  (correct letter, correct position)");
    println!("  Y = Yellow (correct letter, wrong position)");
    println!("  _ = Gray   (letter not in word)");
    println!();
    println!("Commands:");
    println!("  list  - Show all remaining candidates");
    println!();
    println!("Example: guess 'crane', feedback 'G_Y__'");
    println!("         means C=green, R=gray, A=yellow, N=gray, E=gray");
    println!();
}

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn main() {
    let all_words = load_words();
    let mut candidates = all_words.clone();

    print_banner();
    println!("Dictionary loaded: {} five-letter words", all_words.len());
    println!();

    if let Some(suggestion) = suggest_next(&candidates, &all_words) {
        println!("Suggested first guess: \"{}\"", suggestion.to_uppercase());
    }
    println!();

    let mut round = 1;
    loop {
        println!("--- Round {} ---", round);
        println!("Candidates remaining: {}", candidates.len());

        if candidates.len() <= 10 {
            let words: Vec<String> = candidates.iter().map(|w| w.to_uppercase()).collect();
            println!("Candidates: {}", words.join(", "));
        }
        println!();

        // Get guess
        let guess = loop {
            let input = read_line("Enter your guess (5 letters, or 'list'): ");
            if is_list_command(&input) {
                println!();
                println!("{}", format_candidates(&candidates));
                println!();
                continue;
            }
            if input.len() == 5 && input.chars().all(|c| c.is_ascii_alphabetic()) {
                break input.to_lowercase();
            }
            println!("  Please enter exactly 5 alphabetic letters.");
        };

        // Get feedback
        let feedback = loop {
            let input = read_line("Enter feedback (G/Y/_ for each letter): ");
            let upper = input.to_uppercase();
            if upper.len() == 5
                && upper
                    .chars()
                    .all(|c| matches!(c, 'G' | 'Y' | '_'))
            {
                break upper;
            }
            println!("  Please enter exactly 5 chars using G, Y, or _.");
        };

        // Win check
        if feedback == "GGGGG" {
            println!();
            println!(
                "Congratulations! Solved in {} guess{}!",
                round,
                if round == 1 { "" } else { "es" }
            );
            break;
        }

        match Clue::parse(&guess, &feedback) {
            Ok(clue) => {
                candidates = filter_candidates(&candidates, &[clue]);
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }

        println!();

        if candidates.is_empty() {
            println!("No candidates remaining.");
            println!("Please check that your guess and feedback are correct.");
            break;
        }

        if let Some(suggestion) = suggest_next(&candidates, &all_words) {
            println!("Suggested next guess: \"{}\"", suggestion.to_uppercase());
        }
        println!();

        round += 1;

        if round > 6 {
            println!("Reached 6 guesses. Game over.");
            if !candidates.is_empty() {
                let words: Vec<String> =
                    candidates.iter().take(5).map(|w| w.to_uppercase()).collect();
                println!("Remaining candidates: {}", words.join(", "));
            }
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_words() {
        let words = load_words();
        assert!(!words.is_empty());
        assert!(words.iter().all(|w| w.len() == 5));
    }

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
    fn test_filter_green() {
        // Guess "swxyz" G____ => 's' must be at pos 0; w,x,y,z are gray (absent)
        // Use letters w,x,y,z that don't appear in test words so only green matters
        let words = vec![
            "stone".to_string(), // s at pos 0 -> PASS
            "slick".to_string(), // s at pos 0 -> PASS
            "crane".to_string(), // c at pos 0, not 's' -> FAIL
        ];
        let clue = Clue::parse("swxyz", "G____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(result.contains(&"stone".to_string()));
        assert!(result.contains(&"slick".to_string()));
        assert!(!result.contains(&"crane".to_string()));
    }

    #[test]
    fn test_filter_gray_removes_words_with_letter() {
        // Guess "crane" _____ => c,r,a,n,e all absent from answer
        // "stole" contains 'e' -> FAIL; "swift" has none of c,r,a,n,e -> PASS
        let words = vec![
            "crane".to_string(), // contains c,r,a,n,e -> FAIL
            "swift".to_string(), // no c,r,a,n,e -> PASS
            "stole".to_string(), // contains 'e' -> FAIL
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
        // 'r' is yellow at position 0 => word contains 'r' but not at position 0
        let clue = Clue::parse("rbxxx", "Y____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(!result.contains(&"raced".to_string())); // 'r' at position 0
        assert!(result.contains(&"crane".to_string())); // 'r' at position 2
        assert!(result.contains(&"acorn".to_string())); // 'r' at position 4
    }

    #[test]
    fn test_suggest_next_single_candidate() {
        let all_words = load_words();
        let candidates = vec!["crane".to_string()];
        let suggestion = suggest_next(&candidates, &all_words);
        assert_eq!(suggestion, Some("crane".to_string()));
    }

    #[test]
    fn test_full_solve_scenario() {
        let all_words = load_words();
        let mut candidates = all_words.clone();

        // Guess "brick" when answer is "stole":
        // b,r,i,c,k are all absent from "stole" -> all gray
        let clue1 = Clue::parse("brick", "_____").unwrap();
        candidates = filter_candidates(&candidates, &[clue1]);
        assert!(!candidates.contains(&"brick".to_string()));
        assert!(candidates.contains(&"stole".to_string()));
        assert!(candidates.len() < all_words.len());
    }

    #[test]
    fn test_duplicate_letter_gray_exact_count() {
        // "eerie" _____ => e,r,i all absent from answer
        // "snowy": no e,r,i -> PASS
        // "built": has 'i' -> FAIL
        // "speed": has 'e' -> FAIL
        let words = vec!["snowy".to_string(), "built".to_string(), "speed".to_string()];
        let clue = Clue::parse("eerie", "_____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(result.contains(&"snowy".to_string()));
        assert!(!result.contains(&"built".to_string())); // has 'i'
        assert!(!result.contains(&"speed".to_string())); // has 'e'
    }

    #[test]
    fn test_duplicate_letter_green_and_gray_exact_count() {
        // Guess "erase" GG___: e=green(pos0), r=green(pos1), a=gray, s=gray, e=gray
        // => e at pos0, r at pos1, a/s absent, exactly 1 'e' (1 green + 1 gray)
        let words = vec![
            "erode".to_string(), // e@0✓ r@1✓ but 2 'e's -> FAIL
            "error".to_string(), // e@0✓ r@1✓ no a/s✓ exactly 1 'e'✓ -> PASS
            "groan".to_string(), // 'e' not at pos 0 -> FAIL
        ];
        let clue = Clue::parse("erase", "GG___").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(!result.contains(&"erode".to_string())); // 2 'e's
        assert!(result.contains(&"error".to_string()));
        assert!(!result.contains(&"groan".to_string())); // wrong pos
    }

    #[test]
    fn test_filter_yellow_letter_must_be_present() {
        // 't' yellow at pos 0 => word contains 't' but NOT at pos 0
        let words = vec![
            "stone".to_string(), // 't' at pos 1 -> PASS
            "built".to_string(), // 't' at pos 4 -> PASS
            "crane".to_string(), // no 't' -> FAIL
            "towel".to_string(), // 't' at pos 0 -> FAIL (yellow: must not be at same pos)
        ];
        let clue = Clue::parse("txxxx", "Y____").unwrap();
        let result = filter_candidates(&words, &[clue]);
        assert!(result.contains(&"stone".to_string()));
        assert!(result.contains(&"built".to_string()));
        assert!(!result.contains(&"crane".to_string())); // no 't'
        assert!(!result.contains(&"towel".to_string())); // 't' at pos 0
    }

    #[test]
    fn test_multiple_clues_accumulated() {
        let all_words = load_words();
        // Clue 1: "raise" _____ => r,a,i,s,e all absent
        // Clue 2: "clout" G____ => c at pos 0, l,o,u,t absent
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
        // Sanity: should have narrowed down meaningfully
        assert!(candidates.len() < all_words.len());
    }

    #[test]
    fn test_solve_converges() {
        // Simulate a 3-step solve for "stove":
        // Round 1: "crane" -> c=gray, r=gray, a=gray, n=gray, e=gray (none in "stove"... wait 'e' is)
        // "stove" has s,t,o,v,e
        // "crane" vs "stove": c∉stove->gray, r∉stove->gray, a∉stove->gray, n∉stove->gray, e∈stove@pos4=same pos->green
        // Round 1: "crane" "____G"
        // Round 2: "stomp" vs "stove": s=green, t=green, o=green, m∉stove->gray, p∉stove->gray
        // Round 2: "stomp" "GGG__"
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
    fn test_suggest_next_returns_candidate() {
        let all_words = load_words();
        // With many candidates, suggestion should come from all_words
        let suggestion = suggest_next(&all_words, &all_words);
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert_eq!(s.len(), 5);
        assert!(all_words.contains(&s));
    }

    #[test]
    fn test_clue_parse_case_insensitive() {
        // Both word and feedback should be accepted in any case
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
        // First line has header + 10 words, second line has remaining 2
        assert_eq!(lines.len(), 2);
    }
}
