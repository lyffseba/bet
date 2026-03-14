#[derive(PartialEq, Debug)]
pub enum GameStatus {
    Ongoing,
    PlayerWins,
    ComputerWins,
}

pub struct PongGame {
    pub ball_x: f64,
    pub ball_y: f64,
    pub ball_dx: f64,
    pub ball_dy: f64,
    pub player_y: f64,
    pub computer_y: f64,
    pub player_score: u8,
    pub computer_score: u8,
    pub status: GameStatus,
}

impl PongGame {
    pub const WIDTH: f64 = 100.0;
    pub const HEIGHT: f64 = 100.0;
    pub const PADDLE_H: f64 = 20.0;
    pub const PADDLE_W: f64 = 2.0;
    pub const BALL_R: f64 = 1.0;
    pub const WIN_SCORE: u8 = 5;

    pub fn new() -> Self {
        let mut game = Self {
            ball_x: 50.0,
            ball_y: 50.0,
            ball_dx: 40.0,
            ball_dy: 30.0,
            player_y: 50.0,
            computer_y: 50.0,
            player_score: 0,
            computer_score: 0,
            status: GameStatus::Ongoing,
        };
        game.reset_ball(true);
        game
    }

    pub fn reset_ball(&mut self, towards_player: bool) {
        self.ball_x = Self::WIDTH / 2.0;
        self.ball_y = Self::HEIGHT / 2.0;
        self.ball_dx = if towards_player { -50.0 } else { 50.0 };
        // Randomize dy slightly
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.ball_dy = rng.gen_range(-30.0..30.0);
    }

    pub fn update(&mut self, dt: f64) {
        if self.status != GameStatus::Ongoing {
            return;
        }

        // Store previous position for Continuous Collision Detection (CCD)
        let prev_x = self.ball_x;
        let _prev_y = self.ball_y;

        // Move ball
        self.ball_x += self.ball_dx * dt;
        self.ball_y += self.ball_dy * dt;

        // Top/Bottom collisions
        if self.ball_y - Self::BALL_R < 0.0 {
            self.ball_y = Self::BALL_R;
            self.ball_dy *= -1.0;
        } else if self.ball_y + Self::BALL_R > Self::HEIGHT {
            self.ball_y = Self::HEIGHT - Self::BALL_R;
            self.ball_dy *= -1.0;
        }

        // Paddle collisions
        let player_x = 5.0;
        let computer_x = 95.0;

        // Player collision (CCD: if it was in front of paddle, and now is behind it, AND y is within paddle)
        // Also keep standard bounding box check for edge cases
        let crossed_player = prev_x - Self::BALL_R >= player_x + Self::PADDLE_W / 2.0 && self.ball_x - Self::BALL_R <= player_x + Self::PADDLE_W / 2.0;
        let inside_player_y = self.ball_y > self.player_y - Self::PADDLE_H / 2.0 && self.ball_y < self.player_y + Self::PADDLE_H / 2.0;
        let bounding_player = self.ball_x - Self::BALL_R < player_x + Self::PADDLE_W / 2.0
            && self.ball_x + Self::BALL_R > player_x - Self::PADDLE_W / 2.0
            && inside_player_y;

        if self.ball_dx < 0.0 && (crossed_player || bounding_player) && inside_player_y {
            self.ball_x = player_x + Self::PADDLE_W / 2.0 + Self::BALL_R;
            self.ball_dx *= -1.1; // Speed up slightly
            // Cap maximum speed to prevent chaotic physics
            self.ball_dx = self.ball_dx.clamp(-150.0, 150.0); 
            
            let diff = self.ball_y - self.player_y;
            self.ball_dy += diff * 2.0; // English (spin)
        }

        // Computer collision
        let crossed_computer = prev_x + Self::BALL_R <= computer_x - Self::PADDLE_W / 2.0 && self.ball_x + Self::BALL_R >= computer_x - Self::PADDLE_W / 2.0;
        let inside_computer_y = self.ball_y > self.computer_y - Self::PADDLE_H / 2.0 && self.ball_y < self.computer_y + Self::PADDLE_H / 2.0;
        let bounding_computer = self.ball_x + Self::BALL_R > computer_x - Self::PADDLE_W / 2.0
            && self.ball_x - Self::BALL_R < computer_x + Self::PADDLE_W / 2.0
            && inside_computer_y;

        if self.ball_dx > 0.0 && (crossed_computer || bounding_computer) && inside_computer_y {
            self.ball_x = computer_x - Self::PADDLE_W / 2.0 - Self::BALL_R;
            self.ball_dx *= -1.1;
            // Cap maximum speed
            self.ball_dx = self.ball_dx.clamp(-150.0, 150.0);
            
            let diff = self.ball_y - self.computer_y;
            self.ball_dy += diff * 2.0;
        }

        // Scoring
        if self.ball_x < 0.0 {
            self.computer_score += 1;
            if self.computer_score >= Self::WIN_SCORE {
                self.status = GameStatus::ComputerWins;
            } else {
                self.reset_ball(true);
            }
        } else if self.ball_x > Self::WIDTH {
            self.player_score += 1;
            if self.player_score >= Self::WIN_SCORE {
                self.status = GameStatus::PlayerWins;
            } else {
                self.reset_ball(false);
            }
        }

        // Computer AI (moves towards ball, speed limited)
        let ai_speed = 35.0;
        if self.computer_y < self.ball_y - 2.0 {
            self.computer_y += ai_speed * dt;
        } else if self.computer_y > self.ball_y + 2.0 {
            self.computer_y -= ai_speed * dt;
        }

        // Clamp paddles
        self.computer_y = self.computer_y.clamp(Self::PADDLE_H / 2.0, Self::HEIGHT - Self::PADDLE_H / 2.0);
    }

    pub fn move_player(&mut self, up: bool) {
        let speed = 8.0;
        if up {
            self.player_y += speed;
        } else {
            self.player_y -= speed;
        }
        self.player_y = self.player_y.clamp(Self::PADDLE_H / 2.0, Self::HEIGHT - Self::PADDLE_H / 2.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let game = PongGame::new();
        assert_eq!(game.status, GameStatus::Ongoing);
        assert_eq!(game.player_score, 0);
        assert_eq!(game.computer_score, 0);
    }

    #[test]
    fn test_paddle_movement() {
        let mut game = PongGame::new();
        let initial_y = game.player_y;
        
        game.move_player(true); // move up
        assert!(game.player_y > initial_y);
        
        let new_y = game.player_y;
        game.move_player(false); // move down
        assert!(game.player_y < new_y);
    }

    #[test]
    fn test_scoring() {
        let mut game = PongGame::new();
        game.ball_x = -10.0; // Ball went past player
        game.update(0.1);
        
        assert_eq!(game.computer_score, 1);
        assert_eq!(game.player_score, 0);
        assert_eq!(game.ball_x, PongGame::WIDTH / 2.0); // Reset
    }
}
