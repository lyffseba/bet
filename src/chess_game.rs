use shakmaty::{Chess, Position, Square, Move, Color};
use rand::seq::SliceRandom;

#[derive(PartialEq, Debug)]
pub enum GameStatus {
    Ongoing,
    Win(Color),
    Stalemate,
    Draw,
}

pub struct ChessGame {
    pub pos: Chess,
    pub status: GameStatus,
    pub is_pvp: bool,
    pub player_color: Color,
}

impl ChessGame {
    pub fn new(is_pvp: bool) -> Self {
        Self {
            pos: Chess::default(),
            status: GameStatus::Ongoing,
            is_pvp,
            player_color: Color::White,
        }
    }

    pub fn get_moves_from(&self, sq: Square) -> Vec<Move> {
        let moves = self.pos.legal_moves();
        moves.into_iter().filter(|m| m.from() == Some(sq)).collect()
    }

    pub fn make_move(&mut self, m: Move) -> bool {
        if self.status != GameStatus::Ongoing {
            return false;
        }

        if let Ok(new_pos) = self.pos.clone().play(m) {
            self.pos = new_pos;
            self.update_status();
            
            if self.status == GameStatus::Ongoing && !self.is_pvp && self.pos.turn() != self.player_color {
                self.computer_move();
                self.update_status();
            }
            return true;
        }
        false
    }

    fn computer_move(&mut self) {
        let moves = self.pos.legal_moves();
        if !moves.is_empty() {
            let mut rng = rand::thread_rng();
            
            let captures: Vec<Move> = moves.iter().filter(|m| m.is_capture()).cloned().collect();
            let chosen = if !captures.is_empty() {
                *captures.choose(&mut rng).unwrap()
            } else {
                let moves_vec: Vec<Move> = moves.into_iter().collect();
                *moves_vec.choose(&mut rng).unwrap()
            };
            
            self.pos = self.pos.clone().play(chosen).unwrap();
        }
    }

    pub fn update_status(&mut self) {
        if self.pos.is_checkmate() {
            self.status = GameStatus::Win(!self.pos.turn());
        } else if self.pos.is_stalemate() {
            self.status = GameStatus::Stalemate;
        } else if self.pos.is_insufficient_material() {
            self.status = GameStatus::Draw;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let game = ChessGame::new(false);
        assert_eq!(game.status, GameStatus::Ongoing);
        assert_eq!(game.pos.turn(), Color::White);
    }

    #[test]
    fn test_valid_move_and_ai_response() {
        let mut game = ChessGame::new(false);
        let moves = game.get_moves_from(Square::E2); // White pawn
        assert!(!moves.is_empty());
        let m = moves.into_iter().find(|m| m.to() == Square::E4).unwrap();
        
        let success = game.make_move(m);
        assert!(success);
        
        // AI should have played instantly because it's false for PVP
        // So it should be White's turn again
        assert_eq!(game.pos.turn(), Color::White);
    }

    #[test]
    fn test_pvp_move() {
        let mut game = ChessGame::new(true); // PvP mode
        let moves = game.get_moves_from(Square::E2); 
        let m = moves.into_iter().find(|m| m.to() == Square::E4).unwrap();
        
        game.make_move(m);
        
        // PvP means it's now Black's turn (AI didn't move)
        assert_eq!(game.pos.turn(), Color::Black);
    }
}
