use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
pub enum Player {
    X,
    O,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Occupied(Player),
}

#[derive(PartialEq)]
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
            [0, 1, 2], [3, 4, 5], [6, 7, 8], // rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8], // cols
            [0, 4, 8], [2, 4, 6],            // diagonals
        ];

        for line in winning_lines.iter() {
            let cells = [self.board[line[0]], self.board[line[1]], self.board[line[2]]];
            let mut count = 0;
            let mut empty_idx = None;

            for (i, &cell) in cells.iter().enumerate() {
                if cell == Cell::Occupied(player) {
                    count += 1;
                } else if cell == Cell::Empty {
                    empty_idx = Some(line[i]);
                }
            }

            if count == 2 && empty_idx.is_some() {
                self.board[empty_idx.unwrap()] = Cell::Occupied(Player::O);
                return true;
            }
        }

        false
    }

    fn update_status(&mut self) {
        let winning_lines = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8], // rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8], // cols
            [0, 4, 8], [2, 4, 6],            // diagonals
        ];

        for line in winning_lines.iter() {
            if let Cell::Occupied(p1) = self.board[line[0]] {
                if self.board[line[1]] == Cell::Occupied(p1) && self.board[line[2]] == Cell::Occupied(p1) {
                    self.status = GameStatus::Win(p1);
                    if p1 == Player::X {
                        self.wins += 1;
                    } else {
                        self.losses += 1;
                    }
                    return;
                }
            }
        }

        if !self.board.contains(&Cell::Empty) {
            self.status = GameStatus::Draw;
            self.draws += 1;
        }
    }
}
