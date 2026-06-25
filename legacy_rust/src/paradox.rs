use std::f64::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParadoxStage {
    TeleportingSelector = 0,
    LiarsGate = 1,
    SchrodingersWave = 2,
    ParadoxQuiz = 3,
    ReverseTuring = 4,
    Complete = 5,
}

impl ParadoxStage {
    pub fn next(self) -> Self {
        match self {
            Self::TeleportingSelector => Self::LiarsGate,
            Self::LiarsGate => Self::SchrodingersWave,
            Self::SchrodingersWave => Self::ParadoxQuiz,
            Self::ParadoxQuiz => Self::ReverseTuring,
            Self::ReverseTuring => Self::Complete,
            Self::Complete => Self::Complete,
        }
    }

    pub fn index(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StagePhase {
    Active,
    Success,
    Bypassed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParadoxAction {
    None,
    AdvanceStage,
    ShowBypass,
    GameComplete,
}

pub struct ParadoxGame {
    pub stage: ParadoxStage,
    pub phase: StagePhase,
    pub time: f64,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub btn_x: u16,
    pub btn_y: u16,
    pub gate_cursor: u8,
    pub quiz_cursor: u8,
    pub quiz_index: u8,
    pub wave_collapsed: bool,
    pub turing_round: u8,
    pub turing_a: i32,
    pub turing_b: i32,
    pub turing_op: u8,
    pub turing_answer: String,
    pub turing_timer: f64,
    pub status_msg: String,
    pub field_w: u16,
    pub field_h: u16,
    rng_seed: u64,
}

const FREEZE_CYCLE: f64 = 4.0;
const FREEZE_START: f64 = 0.55;
const FREEZE_END: f64 = 0.85;
const WAVE_SPEED: f64 = 2.5;
const PEAK_THRESHOLD: f64 = 0.92;
const TURING_TIME: f64 = 6.0;
const TURING_ROUNDS: u8 = 3;
const LIAR_CORRECT: u8 = 0;

pub const QUIZ_COUNT: u8 = 3;

pub fn quiz_correct_answer(quiz_index: u8) -> u8 {
    match quiz_index {
        0 => 2,
        1 => 1,
        _ => 3,
    }
}

impl ParadoxGame {
    pub fn new(field_w: u16, field_h: u16) -> Self {
        let mut g = Self {
            stage: ParadoxStage::TeleportingSelector,
            phase: StagePhase::Active,
            time: 0.0,
            cursor_x: field_w / 2,
            cursor_y: field_h / 2,
            btn_x: field_w / 2,
            btn_y: field_h / 3,
            gate_cursor: 0,
            quiz_cursor: 0,
            quiz_index: 0,
            wave_collapsed: false,
            turing_round: 0,
            turing_a: 7,
            turing_b: 3,
            turing_op: 0,
            turing_answer: String::new(),
            turing_timer: TURING_TIME,
            status_msg: String::new(),
            field_w,
            field_h,
            rng_seed: 0xDEAD_BEEF_CAFE_BABE,
        };
        g.scatter_button();
        g
    }

    fn rng_next(&mut self) -> u64 {
        self.rng_seed = self.rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.rng_seed
    }

    fn rng_range(&mut self, max: u16) -> u16 {
        if max == 0 {
            return 0;
        }
        (self.rng_next() % max as u64) as u16
    }

    pub fn freeze_active(&self) -> bool {
        let t = (self.time % FREEZE_CYCLE) / FREEZE_CYCLE;
        t >= FREEZE_START && t <= FREEZE_END
    }

    pub fn wave_value(&self) -> f64 {
        (self.time * WAVE_SPEED).sin()
    }

    pub fn wave_at_peak(&self) -> bool {
        self.wave_value() >= PEAK_THRESHOLD
    }

    pub fn turing_expected(&self) -> i32 {
        match self.turing_op {
            0 => self.turing_a + self.turing_b,
            1 => self.turing_a - self.turing_b,
            _ => self.turing_a * self.turing_b,
        }
    }

    pub fn turing_label(&self) -> char {
        match self.turing_op {
            0 => '+',
            1 => '-',
            _ => '×',
        }
    }

    fn scatter_button(&mut self) {
        let margin = 2;
        let max_x = self.field_w.saturating_sub(margin + 4);
        let max_y = self.field_h.saturating_sub(margin + 1);
        if max_x > margin {
            self.btn_x = margin + self.rng_range(max_x - margin);
        }
        if max_y > margin {
            self.btn_y = margin + self.rng_range(max_y - margin);
        }
    }

    fn cursor_on_button(&self) -> bool {
        self.cursor_x.abs_diff(self.btn_x) <= 1 && self.cursor_y.abs_diff(self.btn_y) <= 0
    }

    pub fn update(&mut self, dt: f64) -> ParadoxAction {
        if self.phase != StagePhase::Active {
            return ParadoxAction::None;
        }
        self.time += dt;
        if self.stage == ParadoxStage::SchrodingersWave {
            self.wave_collapsed = false;
        }
        if self.stage == ParadoxStage::ReverseTuring {
            self.turing_timer -= dt;
            if self.turing_timer <= 0.0 {
                self.status_msg.clear();
                self.next_turing_round(false);
            }
        }
        ParadoxAction::None
    }

    pub fn move_cursor(&mut self, dx: i16, dy: i16) {
        if self.stage != ParadoxStage::TeleportingSelector || self.phase != StagePhase::Active {
            return;
        }
        let nx = (self.cursor_x as i32 + dx as i32).clamp(1, (self.field_w - 2) as i32) as u16;
        let ny = (self.cursor_y as i32 + dy as i32).clamp(1, (self.field_h - 2) as i32) as u16;
        self.cursor_x = nx;
        self.cursor_y = ny;
    }

    pub fn try_select_teleport(&mut self) -> bool {
        if self.freeze_active() && self.cursor_on_button() {
            self.complete_stage();
            true
        } else {
            self.scatter_button();
            false
        }
    }

    pub fn gate_choice(&mut self, choice: u8) -> bool {
        if choice == LIAR_CORRECT {
            self.complete_stage();
            true
        } else {
            false
        }
    }

    pub fn try_wave_collapse(&mut self) -> bool {
        if self.wave_at_peak() {
            self.wave_collapsed = true;
            self.complete_stage();
            true
        } else {
            self.wave_collapsed = true;
            false
        }
    }

    pub fn quiz_choice(&mut self, choice: u8) -> bool {
        if choice == quiz_correct_answer(self.quiz_index) {
            if self.quiz_index + 1 >= QUIZ_COUNT {
                self.complete_stage();
            } else {
                self.quiz_index += 1;
                self.quiz_cursor = 0;
            }
            true
        } else {
            false
        }
    }

    pub fn submit_turing(&mut self) -> bool {
        let Ok(val) = self.turing_answer.trim().parse::<i32>() else {
            return false;
        };
        let ok = val == self.turing_expected();
        self.turing_answer.clear();
        self.next_turing_round(ok)
    }

    fn next_turing_round(&mut self, ok: bool) -> bool {
        if !ok {
            self.turing_timer = TURING_TIME;
            self.spawn_turing();
            return false;
        }
        self.turing_round += 1;
        if self.turing_round >= TURING_ROUNDS {
            self.complete_stage();
            return true;
        }
        self.turing_timer = TURING_TIME;
        self.spawn_turing();
        true
    }

    fn spawn_turing(&mut self) {
        self.turing_a = (self.rng_range(18) as i32) + 2;
        self.turing_b = (self.rng_range(12) as i32) + 1;
        self.turing_op = (self.rng_range(3) as u8).min(2);
    }

    pub fn logic_bypass(&mut self) -> ParadoxAction {
        self.phase = StagePhase::Bypassed;
        ParadoxAction::ShowBypass
    }

    pub fn acknowledge(&mut self) -> ParadoxAction {
        match self.phase {
            StagePhase::Success | StagePhase::Bypassed => {
                let next = self.stage.next();
                if next == ParadoxStage::Complete {
                    self.stage = ParadoxStage::Complete;
                    self.phase = StagePhase::Success;
                    ParadoxAction::GameComplete
                } else {
                    self.stage = next;
                    self.phase = StagePhase::Active;
                    self.reset_stage_state();
                    ParadoxAction::AdvanceStage
                }
            }
            _ => ParadoxAction::None,
        }
    }

    fn complete_stage(&mut self) {
        self.phase = StagePhase::Success;
    }

    fn reset_stage_state(&mut self) {
        self.status_msg.clear();
        match self.stage {
            ParadoxStage::TeleportingSelector => {
                self.cursor_x = self.field_w / 2;
                self.cursor_y = self.field_h / 2;
                self.scatter_button();
            }
            ParadoxStage::LiarsGate => self.gate_cursor = 0,
            ParadoxStage::SchrodingersWave => {
                self.wave_collapsed = false;
                self.time = 0.0;
            }
            ParadoxStage::ParadoxQuiz => {
                self.quiz_index = 0;
                self.quiz_cursor = 0;
            }
            ParadoxStage::ReverseTuring => {
                self.turing_round = 0;
                self.turing_timer = TURING_TIME;
                self.turing_answer.clear();
                self.spawn_turing();
            }
            ParadoxStage::Complete => {}
        }
    }

    #[allow(dead_code)]
    pub fn render_wave(width: usize) -> String {
        let w = width.max(20);
        let mut out = String::with_capacity(w + 2);
        for x in 0..w {
            let t = x as f64 / w as f64 * 2.0 * PI;
            let y = t.sin();
            let ch = if y > 0.85 {
                '█'
            } else if y > 0.4 {
                '▓'
            } else if y > 0.0 {
                '▒'
            } else if y > -0.4 {
                '░'
            } else {
                ' '
            };
            out.push(ch);
        }
        out
    }

    pub fn render_wave_at(time: f64, width: usize) -> String {
        let w = width.max(20);
        let mut out = String::with_capacity(w + 2);
        for x in 0..w {
            let t = x as f64 / w as f64 * 2.0 * PI + time * WAVE_SPEED;
            let y = t.sin();
            let ch = if y > 0.85 {
                '█'
            } else if y > 0.4 {
                '▓'
            } else if y > 0.0 {
                '▒'
            } else if y > -0.4 {
                '░'
            } else {
                ' '
            };
            out.push(ch);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_starts_at_stage_one() {
        let g = ParadoxGame::new(40, 12);
        assert_eq!(g.stage, ParadoxStage::TeleportingSelector);
        assert_eq!(g.phase, StagePhase::Active);
    }

    #[test]
    fn stage_progression_order() {
        let mut s = ParadoxStage::TeleportingSelector;
        let expected = [
            ParadoxStage::LiarsGate,
            ParadoxStage::SchrodingersWave,
            ParadoxStage::ParadoxQuiz,
            ParadoxStage::ReverseTuring,
            ParadoxStage::Complete,
        ];
        for e in expected {
            s = s.next();
            assert_eq!(s, e);
        }
        assert_eq!(s.next(), ParadoxStage::Complete);
    }

    #[test]
    fn freeze_window_cycles() {
        let mut g = ParadoxGame::new(40, 12);
        g.time = 0.0;
        assert!(!g.freeze_active());
        g.time = FREEZE_CYCLE * FREEZE_START + 0.1;
        assert!(g.freeze_active());
        g.time = FREEZE_CYCLE * FREEZE_END + 0.1;
        assert!(!g.freeze_active());
    }

    #[test]
    fn teleport_fails_outside_freeze() {
        let mut g = ParadoxGame::new(40, 12);
        g.time = 0.0;
        g.cursor_x = g.btn_x;
        g.cursor_y = g.btn_y;
        let bx = g.btn_x;
        assert!(!g.try_select_teleport());
        assert_eq!(g.phase, StagePhase::Active);
        assert_ne!(g.btn_x, bx);
    }

    #[test]
    fn teleport_succeeds_in_freeze() {
        let mut g = ParadoxGame::new(40, 12);
        g.time = FREEZE_CYCLE * 0.7;
        g.cursor_x = g.btn_x;
        g.cursor_y = g.btn_y;
        assert!(g.try_select_teleport());
        assert_eq!(g.phase, StagePhase::Success);
    }

    #[test]
    fn bypass_then_acknowledge_advances() {
        let mut g = ParadoxGame::new(40, 12);
        assert_eq!(g.logic_bypass(), ParadoxAction::ShowBypass);
        assert_eq!(g.phase, StagePhase::Bypassed);
        assert_eq!(g.acknowledge(), ParadoxAction::AdvanceStage);
        assert_eq!(g.stage, ParadoxStage::LiarsGate);
        assert_eq!(g.phase, StagePhase::Active);
    }

    #[test]
    fn liar_gate_correct_answer() {
        let mut g = ParadoxGame::new(40, 12);
        g.stage = ParadoxStage::LiarsGate;
        assert!(g.gate_choice(LIAR_CORRECT));
        assert_eq!(g.phase, StagePhase::Success);
        assert!(!g.gate_choice(1));
    }

    #[test]
    fn wave_peak_detection() {
        let mut g = ParadoxGame::new(40, 12);
        g.stage = ParadoxStage::SchrodingersWave;
        g.time = (PI / 2.0) / WAVE_SPEED;
        assert!(g.wave_at_peak());
        assert!(g.try_wave_collapse());
        assert_eq!(g.phase, StagePhase::Success);
    }

    #[test]
    fn wave_miss_stays_active() {
        let mut g = ParadoxGame::new(40, 12);
        g.stage = ParadoxStage::SchrodingersWave;
        g.time = PI / WAVE_SPEED;
        assert!(!g.wave_at_peak());
        assert!(!g.try_wave_collapse());
        assert_eq!(g.phase, StagePhase::Active);
    }

    #[test]
    fn quiz_advances_through_all_questions() {
        let mut g = ParadoxGame::new(40, 12);
        g.stage = ParadoxStage::ParadoxQuiz;
        for i in 0..QUIZ_COUNT {
            assert_eq!(g.quiz_index, i);
            let ans = quiz_correct_answer(i);
            assert!(g.quiz_choice(ans));
        }
        assert_eq!(g.phase, StagePhase::Success);
    }

    #[test]
    fn quiz_wrong_answer_stays() {
        let mut g = ParadoxGame::new(40, 12);
        g.stage = ParadoxStage::ParadoxQuiz;
        let wrong = if quiz_correct_answer(0) == 0 { 1 } else { 0 };
        assert!(!g.quiz_choice(wrong));
        assert_eq!(g.quiz_index, 0);
        assert_eq!(g.phase, StagePhase::Active);
    }

    #[test]
    fn turing_three_correct_completes() {
        let mut g = ParadoxGame::new(40, 12);
        g.stage = ParadoxStage::ReverseTuring;
        g.turing_round = 0;
        for _ in 0..TURING_ROUNDS {
            g.turing_answer = g.turing_expected().to_string();
            assert!(g.submit_turing());
        }
        assert_eq!(g.phase, StagePhase::Success);
    }

    #[test]
    fn turing_wrong_resets_timer() {
        let mut g = ParadoxGame::new(40, 12);
        g.stage = ParadoxStage::ReverseTuring;
        g.turing_timer = 1.0;
        g.turing_answer = "99999".to_string();
        assert!(!g.submit_turing());
        assert_eq!(g.turing_round, 0);
        assert!(g.turing_timer > 1.0);
    }

    #[test]
    fn full_bypass_path_to_complete() {
        let mut g = ParadoxGame::new(40, 12);
        for _ in 0..5 {
            g.logic_bypass();
            let action = g.acknowledge();
            if g.stage == ParadoxStage::Complete {
                assert_eq!(action, ParadoxAction::GameComplete);
                break;
            }
            assert_eq!(action, ParadoxAction::AdvanceStage);
        }
        assert_eq!(g.stage, ParadoxStage::Complete);
    }

    #[test]
    fn render_wave_has_correct_width() {
        let s = ParadoxGame::render_wave_at(1.0, 50);
        assert_eq!(s.chars().count(), 50);
    }
}