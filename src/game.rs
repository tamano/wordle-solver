use std::io::{BufRead, Write};

use crate::clue::Clue;
use crate::solver::{filter_candidates, format_candidates, is_list_command, suggest_next};

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

        // Get guess
        let guess = loop {
            let line = match read_line("Enter your guess (5 letters, or 'list'): ", input, out) {
                Some(s) => s,
                None => return GameResult::InputClosed,
            };
            if is_list_command(&line) {
                writeln!(out).unwrap();
                writeln!(out, "{}", format_candidates(&candidates)).unwrap();
                writeln!(out).unwrap();
                continue;
            }
            if line.len() == 5 && line.chars().all(|c| c.is_ascii_alphabetic()) {
                break line.to_lowercase();
            }
            writeln!(out, "  Please enter exactly 5 alphabetic letters.").unwrap();
        };

        // Get feedback
        let feedback = loop {
            let line = match read_line("Enter feedback (G/Y/_ for each letter): ", input, out) {
                Some(s) => s,
                None => return GameResult::InputClosed,
            };
            let upper = line.to_uppercase();
            if upper.len() == 5
                && upper
                    .chars()
                    .all(|c| matches!(c, 'G' | 'Y' | '_'))
            {
                break upper;
            }
            writeln!(out, "  Please enter exactly 5 chars using G, Y, or _.").unwrap();
        };

        // Win check
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
