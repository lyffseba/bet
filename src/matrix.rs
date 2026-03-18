use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Clone, Copy, PartialEq)]
pub enum GameStatus {
    Ongoing,
    GameOver,
}

pub struct RainDrop {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub len: usize,
}

pub struct ActiveWord {
    pub id: u64,
    pub text: String,
    pub typed: usize,
    pub x: f64,
    pub y: f64,
    pub speed: f64,
}

pub struct MatrixGame {
    pub words: Vec<ActiveWord>,
    pub rain_drops: Vec<RainDrop>,
    pub status: GameStatus,
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub lives: u32,
    pub level: u32,
    pub target_id: Option<u64>,
    pub spawn_timer: f64,
    pub time_passed: f64,
    word_dictionary: Vec<String>,
    next_id: u64,
    pub terminal_width: f64,
}

impl MatrixGame {
    pub fn new(terminal_width: f64) -> Self {
        let dict = Self::build_dictionary();
        Self {
            words: Vec::new(),
            rain_drops: {
                let mut rain = Vec::new();
                use rand::Rng;
                let mut rng = rand::thread_rng();
                for _ in 0..40 {
                    rain.push(RainDrop {
                        x: rng.gen_range(0.0..terminal_width),
                        y: rng.gen_range(-30.0..30.0), // Some already falling
                        speed: rng.gen_range(15.0..45.0),
                        len: rng.gen_range(5..20),
                    });
                }
                rain
            },
            status: GameStatus::Ongoing,
            score: 0,
            combo: 0,
            max_combo: 0,
            lives: 3,
            level: 1,
            target_id: None,
            spawn_timer: 2.0,
            time_passed: 0.0,
            word_dictionary: dict,
            next_id: 1,
            terminal_width,
        }
    }

    fn build_dictionary() -> Vec<String> {
        let mut words = Vec::new();
        let lists = [
            crate::wordlist::POETRY_QUOTES,
            crate::wordlist::ENGLISH_MOVIES,
            crate::wordlist::MEMES,
            crate::wordlist::ASCII_MEMES,
            crate::wordlist::VIDEO_GAMES,
        ];
        for list in lists.iter() {
            for phrase in list.iter() {
                for word in phrase.split_whitespace() {
                    let clean: String = word.chars().filter(|c| c.is_alphabetic()).collect();
                    if clean.len() > 2 && clean.len() < 15 {
                        words.push(clean.to_lowercase());
                    }
                }
            }
        }
        words.sort();
        words.dedup();
        words
    }

    pub fn update(&mut self, dt: f64) -> (bool, bool) { // returns (word_completed, lost_life)
        if self.status == GameStatus::GameOver {
            return (false, false);
        }

        self.time_passed += dt;
        
        // Level progression
        self.level = 1 + (self.score / 500);

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_word();
            // Spawn rate increases with level
            let spawn_rate = (2.5 - (self.level as f64 * 0.15)).max(0.5);
            self.spawn_timer = spawn_rate;
        }

        let mut lost_life = false;
        let word_completed = false;

        // Update physics
        for w in &mut self.words {
            // Flowing physics: smooth glide downwards
            w.y += w.speed * dt;
        }
        
        // Digital Rain Physics
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for r in &mut self.rain_drops {
            r.y += r.speed * dt;
            if r.y - (r.len as f64) > 30.0 {
                // Reset drop to the top
                r.y = rng.gen_range(-10.0..0.0);
                r.x = rng.gen_range(0.0..self.terminal_width);
                r.speed = rng.gen_range(15.0..45.0);
                r.len = rng.gen_range(5..20);
            }
        }

        // Check bounds
        let bottom_limit = 22.0; // Assume height of game area is ~24, so limit is 22
        
        let mut i = 0;
        while i < self.words.len() {
            if self.words[i].y >= bottom_limit {
                // Word hit the bottom!
                if Some(self.words[i].id) == self.target_id {
                    self.target_id = None;
                }
                self.words.remove(i);
                
                self.lives = self.lives.saturating_sub(1);
                self.combo = 0;
                lost_life = true;
                
                if self.lives == 0 {
                    self.status = GameStatus::GameOver;
                }
            } else {
                i += 1;
            }
        }

        (word_completed, lost_life)
    }

    fn spawn_word(&mut self) {
        let mut rng = rand::thread_rng();
        if let Some(text) = self.word_dictionary.choose(&mut rng) {
            // Speed scales with level
            let base_speed = rng.gen_range(1.5..3.0);
            let speed = base_speed + (self.level as f64 * 0.3);
            
            // Ensure x is well within bounds to avoid edge clipping (10 char buffer)
            let buffer = 10.0;
            let max_x = (self.terminal_width - text.len() as f64 - buffer).max(buffer);
            let x = rng.gen_range(buffer..max_x);
            
            self.words.push(ActiveWord {
                id: self.next_id,
                text: text.clone(),
                typed: 0,
                x,
                y: 0.0,
                speed,
            });
            self.next_id += 1;
        }
    }

    pub fn type_char(&mut self, c: char) -> (bool, bool, Option<(f64, f64)>) {
        // returns (hit, completed, Option<coordinates of completed word>)
        if self.status == GameStatus::GameOver || self.words.is_empty() {
            return (false, false, None);
        }

        let mut hit = false;
        let mut completed = false;
        let mut explosion_coords = None;

        if let Some(tid) = self.target_id {
            // We have a target, MUST type its next letter
            if let Some(idx) = self.words.iter().position(|w| w.id == tid) {
                let w = &mut self.words[idx];
                let next_char = w.text.chars().nth(w.typed).unwrap();
                if next_char.to_lowercase().next() == Some(c.to_lowercase().next().unwrap_or(c)) {
                    w.typed += 1;
                    hit = true;
                    if w.typed >= w.text.len() {
                        // Word finished!
                        self.score += (w.text.len() as u32 * 10) * (1 + self.combo / 10);
                        self.combo += 1;
                        if self.combo > self.max_combo {
                            self.max_combo = self.combo;
                        }
                        explosion_coords = Some((w.x + w.text.len() as f64 / 2.0, w.y));
                        self.words.remove(idx);
                        self.target_id = None;
                        completed = true;
                    }
                } else {
                    // Typo on target!
                    self.combo = 0;
                }
            } else {
                self.target_id = None; // Should not happen, but safe fallback
            }
        } else {
            // No target, find the lowest word that starts with this letter
            let mut best_idx = None;
            let mut best_y = -1.0;
            
            for (i, w) in self.words.iter().enumerate() {
                if let Some(first_char) = w.text.chars().next()
                    && first_char.to_lowercase().next() == Some(c.to_lowercase().next().unwrap_or(c))
                        && w.y > best_y {
                            best_y = w.y;
                            best_idx = Some(i);
                        }
            }

            if let Some(idx) = best_idx {
                self.target_id = Some(self.words[idx].id);
                self.words[idx].typed += 1;
                hit = true;
                
                // Edge case: 1 letter word (though dictionary filters out < 3)
                if self.words[idx].typed >= self.words[idx].text.len() {
                    self.score += 10 * (1 + self.combo / 10);
                    self.combo += 1;
                    if self.combo > self.max_combo {
                        self.max_combo = self.combo;
                    }
                    explosion_coords = Some((self.words[idx].x, self.words[idx].y));
                    self.words.remove(idx);
                    self.target_id = None;
                    completed = true;
                }
            } else {
                // Typed wrong letter with no target
                self.combo = 0;
            }
        }

        (hit, completed, explosion_coords)
    }
}
