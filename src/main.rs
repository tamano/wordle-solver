mod clue;
mod game;
mod solver;

use std::io;

fn main() {
    let all_words = solver::load_words();
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut out = io::stdout();
    game::run_game(&all_words, &mut input, &mut out);
}

#[cfg(test)]
mod tests {
    use crate::clue::{Clue, Hint};
    use crate::solver::*;

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

        let clue1 = Clue::parse("brick", "_____").unwrap();
        candidates = filter_candidates(&candidates, &[clue1]);
        assert!(!candidates.contains(&"brick".to_string()));
        assert!(candidates.contains(&"stole".to_string()));
        assert!(candidates.len() < all_words.len());
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
    fn test_suggest_next_returns_candidate() {
        let all_words = load_words();
        let suggestion = suggest_next(&all_words, &all_words);
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert_eq!(s.len(), 5);
        assert!(all_words.contains(&s));
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

    #[test]
    fn test_suggest_next_empty_candidates() {
        let all_words = load_words();
        let candidates: Vec<String> = vec![];
        assert_eq!(suggest_next(&candidates, &all_words), None);
    }
}

#[cfg(test)]
mod game_tests {
    use std::io::Cursor;
    use crate::game::{run_game, GameResult};

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
        // Guess correctly on round 1
        let words = words();
        let input = "crane\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 1 });
        assert!(output.contains("Solved in 1 guess!"));
    }

    #[test]
    fn test_game_solved_round_2() {
        let words = vec!["abcde".to_string(), "fghij".to_string()];
        // Round 1: guess "abcde" feedback "_____" → eliminates "abcde", keeps "fghij"
        // Round 2: guess "fghij" feedback "GGGGG"
        let input = "abcde\n_____\nfghij\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 2 });
        assert!(output.contains("Solved in 2 guesses!"));
    }

    #[test]
    fn test_game_no_candidates() {
        // Contradictory feedback eliminates all candidates
        let words = vec!["apple".to_string()];
        let input = "apple\n_____\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::NoCandidates);
        assert!(output.contains("No candidates remaining"));
    }

    #[test]
    fn test_game_input_closed_at_guess() {
        let words = words();
        let input = "";  // EOF immediately
        let (result, _) = run(&words, input);
        assert_eq!(result, GameResult::InputClosed);
    }

    #[test]
    fn test_game_input_closed_at_feedback() {
        let words = words();
        let input = "crane\n";  // EOF after guess, before feedback
        let (result, _) = run(&words, input);
        assert_eq!(result, GameResult::InputClosed);
    }

    #[test]
    fn test_game_invalid_guess_retry() {
        let words = words();
        // "hi" is too short, "123456" is wrong, then valid guess + win
        let input = "hi\n12345\ncrane\nGGGGG\n";
        let (result, output) = run(&words, input);
        assert_eq!(result, GameResult::Solved { round: 1 });
        assert!(output.contains("Please enter exactly 5 alphabetic letters"));
    }

    #[test]
    fn test_game_invalid_feedback_retry() {
        let words = words();
        // Valid guess, bad feedback "XXXXX", then correct feedback
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
        // guess "abcdx" with "GGGG_": pos0-3 correct, pos4 gray. Both words survive.
        for _ in 0..6 {
            input.push_str("abcdx\nGGGG_\n");
        }

        let (result, output) = run(&words, &input);
        assert!(matches!(result, GameResult::GameOver { .. }));
        assert!(output.contains("Reached 6 guesses. Game over."));
    }

    #[test]
    fn test_game_shows_candidates_when_few() {
        // With <= 10 candidates, they should be listed
        let words = vec!["crane".to_string(), "stone".to_string()];
        let input = "crane\nGGGGG\n";
        let (_, output) = run(&words, input);
        assert!(output.contains("Candidates:"));
        assert!(output.contains("CRANE"));
        assert!(output.contains("STONE"));
    }
}
