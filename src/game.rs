use std::io::{BufRead, Write};

use crate::clue::Clue;
use crate::solver::{filter_candidates, suggest_next};

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

fn print_banner(out: &mut dyn Write) {
    writeln!(out, "╔══════════════════════════════════╗").unwrap();
    writeln!(out, "║        Wordle Solver CLI         ║").unwrap();
    writeln!(out, "╚══════════════════════════════════╝").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Feedback format: 5 chars, one per letter").unwrap();
    writeln!(out, "  G = Green  (correct letter, correct position)").unwrap();
    writeln!(out, "  Y = Yellow (correct letter, wrong position)").unwrap();
    writeln!(out, "  _ = Gray   (letter not in word)").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Commands:").unwrap();
    writeln!(out, "  list  - Show all remaining candidates").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Example: guess 'crane', feedback 'G_Y__'").unwrap();
    writeln!(out, "         means C=green, R=gray, A=yellow, N=gray, E=gray").unwrap();
    writeln!(out).unwrap();
}

fn read_line(prompt: &str, input: &mut dyn BufRead, out: &mut dyn Write) -> Option<String> {
    write!(out, "{}", prompt).unwrap();
    out.flush().unwrap();
    let mut buf = String::new();
    match input.read_line(&mut buf) {
        Ok(0) => None,
        Ok(_) => Some(buf.trim().to_string()),
        Err(_) => None,
    }
}

/// Game result returned by run_game.
#[derive(Debug, PartialEq)]
pub enum GameResult {
    Solved { round: usize },
    NoCandidates,
    GameOver { remaining: Vec<String> },
    InputClosed,
}

/// Prompt for a valid 5-letter guess, handling list commands. Returns None on EOF.
fn read_guess(
    candidates: &[String],
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> Option<String> {
    loop {
        let line = read_line("Enter your guess (5 letters, or 'list'): ", input, out)?;
        if is_list_command(&line) {
            writeln!(out).unwrap();
            writeln!(out, "{}", format_candidates(candidates)).unwrap();
            writeln!(out).unwrap();
            continue;
        }
        if line.len() == 5 && line.chars().all(|c| c.is_ascii_alphabetic()) {
            return Some(line.to_lowercase());
        }
        writeln!(out, "  Please enter exactly 5 alphabetic letters.").unwrap();
    }
}

/// Prompt for valid feedback (5 chars of G/Y/_). Returns None on EOF.
fn read_feedback(input: &mut dyn BufRead, out: &mut dyn Write) -> Option<String> {
    loop {
        let line = read_line("Enter feedback (G/Y/_ for each letter): ", input, out)?;
        let upper = line.to_uppercase();
        if upper.len() == 5 && upper.chars().all(|c| matches!(c, 'G' | 'Y' | '_')) {
            return Some(upper);
        }
        writeln!(out, "  Please enter exactly 5 chars using G, Y, or _.").unwrap();
    }
}

pub fn run_game(
    all_words: &[String],
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> GameResult {
    let mut candidates = all_words.to_vec();

    print_banner(out);
    writeln!(out, "Dictionary loaded: {} five-letter words", all_words.len()).unwrap();
    writeln!(out).unwrap();

    if let Some(suggestion) = suggest_next(&candidates, all_words) {
        writeln!(out, "Suggested first guess: \"{}\"", suggestion.to_uppercase()).unwrap();
    }
    writeln!(out).unwrap();

    let mut round = 1;
    loop {
        writeln!(out, "--- Round {} ---", round).unwrap();
        writeln!(out, "Candidates remaining: {}", candidates.len()).unwrap();

        if candidates.len() <= 10 {
            let words: Vec<String> = candidates.iter().map(|w| w.to_uppercase()).collect();
            writeln!(out, "Candidates: {}", words.join(", ")).unwrap();
        }
        writeln!(out).unwrap();

        let guess = match read_guess(&candidates, input, out) {
            Some(g) => g,
            None => return GameResult::InputClosed,
        };

        let feedback = match read_feedback(input, out) {
            Some(f) => f,
            None => return GameResult::InputClosed,
        };

        if feedback == "GGGGG" {
            writeln!(out).unwrap();
            writeln!(
                out,
                "Congratulations! Solved in {} guess{}!",
                round,
                if round == 1 { "" } else { "es" }
            ).unwrap();
            return GameResult::Solved { round };
        }

        match Clue::parse(&guess, &feedback) {
            Ok(clue) => {
                candidates = filter_candidates(&candidates, &[clue]);
            }
            Err(e) => {
                writeln!(out, "Error: {}", e).unwrap();
                continue;
            }
        }

        writeln!(out).unwrap();

        if candidates.is_empty() {
            writeln!(out, "No candidates remaining.").unwrap();
            writeln!(out, "Please check that your guess and feedback are correct.").unwrap();
            return GameResult::NoCandidates;
        }

        if let Some(suggestion) = suggest_next(&candidates, all_words) {
            writeln!(out, "Suggested next guess: \"{}\"", suggestion.to_uppercase()).unwrap();
        }
        writeln!(out).unwrap();

        round += 1;

        if round > 6 {
            writeln!(out, "Reached 6 guesses. Game over.").unwrap();
            let remaining: Vec<String> = candidates.iter().take(5).cloned().collect();
            if !remaining.is_empty() {
                let words: Vec<String> = remaining.iter().map(|w| w.to_uppercase()).collect();
                writeln!(out, "Remaining candidates: {}", words.join(", ")).unwrap();
            }
            return GameResult::GameOver { remaining };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    fn words() -> Vec<String> {
        vec![
            "apple".to_string(),
            "grape".to_string(),
            "crane".to_string(),
            "stone".to_string(),
            "stove".to_string(),
        ]
    }

    fn run(words: &[String], input: &str) -> (GameResult, String) {
        let mut cursor = Cursor::new(input.as_bytes().to_vec());
        let mut out = Vec::new();
        let result = run_game(words, &mut cursor, &mut out);
        (result, String::from_utf8(out).unwrap())
    }

    #[test]
    fn test_game_solved_round_1() {
        let words = words();
        let input = "crane\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 1 });
        assert!(output.contains("Solved in 1 guess!"));
    }

    #[test]
    fn test_game_solved_round_2() {
        let words = vec!["abcde".to_string(), "fghij".to_string()];
        let input = "abcde\n_____\nfghij\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 2 });
        assert!(output.contains("Solved in 2 guesses!"));
    }

    #[test]
    fn test_game_no_candidates() {
        let words = vec!["apple".to_string()];
        let input = "apple\n_____\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::NoCandidates);
        assert!(output.contains("No candidates remaining"));
    }

    #[test]
    fn test_game_input_closed_at_guess() {
        let words = words();
        let input = "";
        let (result, _) = run(&words, input);
        assert_eq!(result, GameResult::InputClosed);
    }

    #[test]
    fn test_game_input_closed_at_feedback() {
        let words = words();
        let input = "crane\n";
        let (result, _) = run(&words, input);
        assert_eq!(result, GameResult::InputClosed);
    }

    #[test]
    fn test_game_invalid_guess_retry() {
        let words = words();
        let input = "hi\n12345\ncrane\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 1 });
        assert!(output.contains("Please enter exactly 5 alphabetic letters"));
    }

    #[test]
    fn test_game_invalid_feedback_retry() {
        let words = words();
        let input = "crane\nXXXXX\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 1 });
        assert!(output.contains("Please enter exactly 5 chars using G, Y, or _"));
    }

    #[test]
    fn test_game_list_command() {
        let words = words();
        let input = "list\ncrane\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 1 });
        assert!(output.contains("Remaining candidates (5):"));
    }

    #[test]
    fn test_game_banner_and_suggestion() {
        let words = words();
        let input = "crane\nGGGGG\n";
        let (_, output) = run(&words, input);
        assert!(output.contains("Wordle Solver CLI"));
        assert!(output.contains("Dictionary loaded: 5 five-letter words"));
        assert!(output.contains("Suggested first guess:"));
    }

    #[test]
    fn test_game_over_after_6_rounds() {
        let words = vec!["abcdf".to_string(), "abcdg".to_string()];
        let mut input = String::new();
        for _ in 0..6 {
            input.push_str("abcdx\nGGGG_\n");
        }

        let (result, output) = run(&words, &input);
        assert!(matches!(result, GameResult::GameOver { .. }));
        assert!(output.contains("Reached 6 guesses. Game over."));
    }

    #[test]
    fn test_game_shows_candidates_when_few() {
        let words = vec!["crane".to_string(), "stone".to_string()];
        let input = "crane\nGGGGG\n";
        let (_, output) = run(&words, input);
        assert!(output.contains("Candidates:"));
        assert!(output.contains("CRANE"));
        assert!(output.contains("STONE"));
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
