use rand::Rng;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Player {
    X,
    O,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Occupied(Player),
}

#[derive(PartialEq, Debug)]
pub enum GameStatus {
    Ongoing,
    Win(Player),
    Draw,
}

pub struct TicTacToe {
    pub board: [Cell; 9],
    pub status: GameStatus,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

impl TicTacToe {
    pub fn new() -> Self {
        Self {
            board: [Cell::Empty; 9],
            status: GameStatus::Ongoing,
            wins: 0,
            losses: 0,
            draws: 0,
        }
    }

    pub fn reset_game(&mut self) {
        self.board = [Cell::Empty; 9];
        self.status = GameStatus::Ongoing;
    }

    pub fn make_move(&mut self, index: usize) -> bool {
        if index >= 9 || self.board[index] != Cell::Empty || self.status != GameStatus::Ongoing {
            return false;
        }

        self.board[index] = Cell::Occupied(Player::X);
        self.update_status();

        if self.status == GameStatus::Ongoing {
            self.computer_move();
            self.update_status();
        }

        true
    }

    fn computer_move(&mut self) {
        // Simple AI: Try to win, then try to block, otherwise random
        if self.try_winning_move(Player::O) {
            return;
        }
        if self.try_winning_move(Player::X) {
            return;
        }

        // Random move
        let mut empty_cells = Vec::new();
        for (i, cell) in self.board.iter().enumerate() {
            if *cell == Cell::Empty {
                empty_cells.push(i);
            }
        }

        if !empty_cells.is_empty() {
            let mut rng = rand::thread_rng();
            let idx = empty_cells[rng.gen_range(0..empty_cells.len())];
            self.board[idx] = Cell::Occupied(Player::O);
        }
    }

    fn try_winning_move(&mut self, player: Player) -> bool {
        let winning_lines = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8], // rows
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8], // cols
            [0, 4, 8],
            [2, 4, 6], // diagonals
        ];

        for line in winning_lines.iter() {
            let cells = [
                self.board[line[0]],
                self.board[line[1]],
                self.board[line[2]],
            ];
            let mut count = 0;
            let mut empty_idx = None;

            for (i, &cell) in cells.iter().enumerate() {
                if cell == Cell::Occupied(player) {
                    count += 1;
                } else if cell == Cell::Empty {
                    empty_idx = Some(line[i]);
                }
            }

            if count == 2
                && let Some(idx) = empty_idx
            {
                self.board[idx] = Cell::Occupied(Player::O);
                return true;
            }
        }

        false
    }

    fn update_status(&mut self) {
        let winning_lines = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8], // rows
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8], // cols
            [0, 4, 8],
            [2, 4, 6], // diagonals
        ];

        for line in winning_lines.iter() {
            if let Cell::Occupied(p1) = self.board[line[0]]
                && self.board[line[1]] == Cell::Occupied(p1)
                && self.board[line[2]] == Cell::Occupied(p1)
            {
                self.status = GameStatus::Win(p1);
                if p1 == Player::X {
                    self.wins += 1;
                } else {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let game = TicTacToe::new();
        assert_eq!(game.status, GameStatus::Ongoing);
        assert!(game.board.iter().all(|c| *c == Cell::Empty));
    }

    #[test]
    fn test_valid_move() {
        let mut game = TicTacToe::new();
        assert!(game.make_move(0)); // X moves at 0
        assert_eq!(game.board[0], Cell::Occupied(Player::X));
        // The AI should have made a move
        assert_eq!(
            game.board
                .iter()
                .filter(|&&c| c == Cell::Occupied(Player::O))
                .count(),
            1
        );
    }

    #[test]
    fn test_invalid_move_occupied() {
        let mut game = TicTacToe::new();
        game.make_move(0); // X moves at 0

        let mut ai_pos = 0;
        for (i, cell) in game.board.iter().enumerate() {
            if *cell == Cell::Occupied(Player::O) {
                ai_pos = i;
                break;
            }
        }

        // Try to move on the AI's position or our own
        assert!(!game.make_move(0));
        assert!(!game.make_move(ai_pos));
    }

    #[test]
    fn test_invalid_move_out_of_bounds() {
        let mut game = TicTacToe::new();
        assert!(!game.make_move(9)); // Out of bounds
        assert!(!game.make_move(10));
    }

    #[test]
    fn test_ai_winning_move() {
        let mut game = TicTacToe::new();
        // Setup a scenario where O can win on the top row
        game.board[0] = Cell::Occupied(Player::O);
        game.board[1] = Cell::Occupied(Player::O);
        game.board[2] = Cell::Empty;

        // AI should take position 2
        game.computer_move();
        assert_eq!(game.board[2], Cell::Occupied(Player::O));
    }

    #[test]
    fn test_ai_blocking_move() {
        let mut game = TicTacToe::new();
        // Setup a scenario where X is about to win on the left column
        game.board[0] = Cell::Occupied(Player::X);
        game.board[3] = Cell::Occupied(Player::X);
        game.board[6] = Cell::Empty;

        // AI should block position 6
        game.computer_move();
        assert_eq!(game.board[6], Cell::Occupied(Player::O));
    }

    #[test]
    fn test_win_detection_rows() {
        let mut game = TicTacToe::new();
        game.board[3] = Cell::Occupied(Player::X);
        game.board[4] = Cell::Occupied(Player::X);
        game.board[5] = Cell::Occupied(Player::X);
        game.update_status();
        assert_eq!(game.status, GameStatus::Win(Player::X));
    }

    #[test]
    fn test_win_detection_cols() {
        let mut game = TicTacToe::new();
        game.board[1] = Cell::Occupied(Player::O);
        game.board[4] = Cell::Occupied(Player::O);
        game.board[7] = Cell::Occupied(Player::O);
        game.update_status();
        assert_eq!(game.status, GameStatus::Win(Player::O));
    }

    #[test]
    fn test_win_detection_diagonals() {
        let mut game = TicTacToe::new();
        game.board[0] = Cell::Occupied(Player::X);
        game.board[4] = Cell::Occupied(Player::X);
        game.board[8] = Cell::Occupied(Player::X);
        game.update_status();
        assert_eq!(game.status, GameStatus::Win(Player::X));

        let mut game2 = TicTacToe::new();
        game2.board[2] = Cell::Occupied(Player::O);
        game2.board[4] = Cell::Occupied(Player::O);
        game2.board[6] = Cell::Occupied(Player::O);
        game2.update_status();
        assert_eq!(game2.status, GameStatus::Win(Player::O));
    }

    #[test]
    fn test_draw_detection() {
        let mut game = TicTacToe::new();
        // Fill the board without any winning lines
        game.board[0] = Cell::Occupied(Player::X);
        game.board[1] = Cell::Occupied(Player::O);
        game.board[2] = Cell::Occupied(Player::X);

        game.board[3] = Cell::Occupied(Player::X);
        game.board[4] = Cell::Occupied(Player::O);
        game.board[5] = Cell::Occupied(Player::O);

        game.board[6] = Cell::Occupied(Player::O);
        game.board[7] = Cell::Occupied(Player::X);
        game.board[8] = Cell::Occupied(Player::X);

        game.update_status();
        assert_eq!(game.status, GameStatus::Draw);
    }
}
