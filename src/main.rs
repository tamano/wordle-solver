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
