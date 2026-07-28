//! Pure tic-tac-toe rules. Multiplayer uses `place`; SP AI uses seeded RNG.

use crate::hash::Hasher;
use crate::rng::XorShift64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Cell {
    Empty,
    Occupied(Player),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameStatus {
    Ongoing,
    Win(Player),
    Draw,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TicTacToe {
    pub board: [Cell; 9],
    pub status: GameStatus,
    pub current: Player,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub winning_line: Option<[usize; 3]>,
}

const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

impl Default for TicTacToe {
    fn default() -> Self {
        Self::new()
    }
}

impl TicTacToe {
    pub fn new() -> Self {
        Self {
            board: [Cell::Empty; 9],
            status: GameStatus::Ongoing,
            current: Player::X,
            wins: 0,
            losses: 0,
            draws: 0,
            winning_line: None,
        }
    }

    pub fn reset_board(&mut self) {
        self.board = [Cell::Empty; 9];
        self.status = GameStatus::Ongoing;
        self.current = Player::X;
        self.winning_line = None;
    }

    /// Alias used by the TUI layer.
    pub fn reset_game(&mut self) {
        self.reset_board();
    }

    /// Pure multiplayer / engine move: place `player` at `index`.
    pub fn place(&mut self, player: Player, index: usize) -> bool {
        if index >= 9
            || self.board[index] != Cell::Empty
            || self.status != GameStatus::Ongoing
            || player != self.current
        {
            return false;
        }
        self.board[index] = Cell::Occupied(player);
        self.refresh_status(player);
        if self.status == GameStatus::Ongoing {
            self.current = player.opponent();
        }
        true
    }

    /// Single-player convenience: human (X) move then seeded AI (O) if ongoing.
    pub fn make_move_vs_ai(&mut self, index: usize, rng: &mut XorShift64) -> bool {
        if !self.place(Player::X, index) {
            return false;
        }
        if self.status == GameStatus::Ongoing {
            self.computer_move(rng);
        }
        true
    }

    /// Back-compat: SP move with fixed seed path via ephemeral RNG from board hash.
    pub fn make_move(&mut self, index: usize) -> bool {
        let seed = self.state_hash();
        let mut rng = XorShift64::new(seed ^ 0xC0FF_EE00_D15E_A5E);
        self.make_move_vs_ai(index, &mut rng)
    }

    pub fn computer_move(&mut self, rng: &mut XorShift64) {
        if self.status != GameStatus::Ongoing || self.current != Player::O {
            return;
        }
        if self.try_line_move(Player::O) {
            return;
        }
        if self.try_line_move(Player::X) {
            // blocked as O
            return;
        }
        let empty: Vec<usize> = self
            .board
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if *c == Cell::Empty { Some(i) } else { None })
            .collect();
        if let Some(&idx) = rng.choose(&empty) {
            let _ = self.place(Player::O, idx);
        }
    }

    fn try_line_move(&mut self, look_for: Player) -> bool {
        for line in WIN_LINES {
            let cells = [
                self.board[line[0]],
                self.board[line[1]],
                self.board[line[2]],
            ];
            let mut count = 0;
            let mut empty_idx = None;
            for (i, &cell) in cells.iter().enumerate() {
                if cell == Cell::Occupied(look_for) {
                    count += 1;
                } else if cell == Cell::Empty {
                    empty_idx = Some(line[i]);
                }
            }
            if count == 2
                && let Some(idx) = empty_idx
            {
                return self.place(Player::O, idx);
            }
        }
        false
    }

    fn refresh_status(&mut self, last_player: Player) {
        for line in WIN_LINES {
            let c0 = self.board[line[0]];
            if let Cell::Occupied(p) = c0
                && self.board[line[1]] == c0
                && self.board[line[2]] == c0
            {
                self.status = GameStatus::Win(p);
                self.winning_line = Some(line);
                // Session stats: X is "human" in SP; for pure MP both can win.
                if p == Player::X {
                    self.wins += 1;
                } else if last_player == Player::O || p == Player::O {
                    self.losses += 1;
                }
                return;
            }
        }
        if !self.board.contains(&Cell::Empty) {
            self.status = GameStatus::Draw;
            self.draws += 1;
        }
    }

    /// Used by tests / legacy code that sets board manually.
    pub fn update_status(&mut self) {
        self.winning_line = None;
        self.status = GameStatus::Ongoing;
        for line in WIN_LINES {
            let c0 = self.board[line[0]];
            if let Cell::Occupied(p) = c0
                && self.board[line[1]] == c0
                && self.board[line[2]] == c0
            {
                self.status = GameStatus::Win(p);
                self.winning_line = Some(line);
                return;
            }
        }
        if !self.board.contains(&Cell::Empty) {
            self.status = GameStatus::Draw;
        }
    }

    pub fn state_hash(&self) -> u64 {
        let mut h = Hasher::new();
        for cell in &self.board {
            let v = match cell {
                Cell::Empty => 0u8,
                Cell::Occupied(Player::X) => 1,
                Cell::Occupied(Player::O) => 2,
            };
            h.write_u8(v);
        }
        h.write_u8(match self.current {
            Player::X => 0,
            Player::O => 1,
        });
        h.write_u8(match self.status {
            GameStatus::Ongoing => 0,
            GameStatus::Win(Player::X) => 1,
            GameStatus::Win(Player::O) => 2,
            GameStatus::Draw => 3,
        });
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let game = TicTacToe::new();
        assert_eq!(game.status, GameStatus::Ongoing);
        assert!(game.board.iter().all(|c| *c == Cell::Empty));
        assert_eq!(game.current, Player::X);
    }

    #[test]
    fn pure_place_alternates() {
        let mut g = TicTacToe::new();
        assert!(g.place(Player::X, 0));
        assert_eq!(g.current, Player::O);
        assert!(g.place(Player::O, 1));
        assert_eq!(g.current, Player::X);
        assert!(!g.place(Player::X, 0)); // occupied
        assert!(!g.place(Player::O, 2)); // wrong turn
    }

    #[test]
    fn win_rows() {
        let mut game = TicTacToe::new();
        game.board[3] = Cell::Occupied(Player::X);
        game.board[4] = Cell::Occupied(Player::X);
        game.board[5] = Cell::Occupied(Player::X);
        game.update_status();
        assert_eq!(game.status, GameStatus::Win(Player::X));
    }

    #[test]
    fn ai_blocks() {
        let mut game = TicTacToe::new();
        game.board[0] = Cell::Occupied(Player::X);
        game.board[3] = Cell::Occupied(Player::X);
        game.current = Player::O;
        let mut rng = XorShift64::new(1);
        game.computer_move(&mut rng);
        assert_eq!(game.board[6], Cell::Occupied(Player::O));
    }

    #[test]
    fn deterministic_ai_from_seed() {
        let mut a = TicTacToe::new();
        let mut b = TicTacToe::new();
        let mut ra = XorShift64::new(7);
        let mut rb = XorShift64::new(7);
        a.make_move_vs_ai(4, &mut ra);
        b.make_move_vs_ai(4, &mut rb);
        assert_eq!(a.board, b.board);
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn draw_detection() {
        let mut game = TicTacToe::new();
        let layout = [
            Player::X, Player::O, Player::X,
            Player::X, Player::O, Player::O,
            Player::O, Player::X, Player::X,
        ];
        for (i, p) in layout.into_iter().enumerate() {
            game.board[i] = Cell::Occupied(p);
        }
        game.update_status();
        assert_eq!(game.status, GameStatus::Draw);
    }
}
