use bet_core::hangman::Hangman;
use bet_core::hash::fingerprint_hex;
use bet_core::rng::XorShift64;
use bet_core::tictactoe::TicTacToe;

fn main() {
    let words = ["ALPHA", "BRAVO", "CHARLIE"];
    let mut h = Hangman::from_seed(99, &words, 6);
    println!("word={}", h.word());
    println!("hash0={}", fingerprint_hex(h.state_hash()));
    println!("guess A {:?}", h.guess('A'));
    println!("hashA={}", fingerprint_hex(h.state_hash()));
    println!("guess X {:?}", h.guess('X'));
    println!("hashAX={}", fingerprint_hex(h.state_hash()));
    println!("display={}", h.display_word());
    println!("guessed={:?}", h.guessed_letters());

    let mut t = TicTacToe::new();
    let mut rng = XorShift64::new(7);
    t.make_move_vs_ai(4, &mut rng);
    println!("ttt board={}", {
        t.board.iter().map(|c| match c {
            bet_core::tictactoe::Cell::Empty => '.',
            bet_core::tictactoe::Cell::Occupied(bet_core::tictactoe::Player::X) => 'X',
            bet_core::tictactoe::Cell::Occupied(bet_core::tictactoe::Player::O) => 'O',
        }).collect::<String>()
    });
    println!("ttt {}", fingerprint_hex(t.state_hash()));
}
