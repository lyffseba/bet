use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::game::{GuessError, Hangman};
use crate::lang::{Lang, Language};

pub enum AppState {
    LanguageSelection,
    Playing,
    GameOver(bool), // true if won, false if lost
}

pub struct App {
    pub state: AppState,
    pub lang: Option<Lang>,
    pub game: Option<Hangman>,
    pub timer: f64,
    pub last_tick: Instant,
    pub should_quit: bool,
    pub error_msg: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::LanguageSelection,
            lang: None,
            game: None,
            timer: 30.0,
            last_tick: Instant::now(),
            should_quit: false,
            error_msg: None,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> io::Result<()> {
        self.last_tick = Instant::now();
        while !self.should_quit {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
            self.tick();
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let timeout = Duration::from_millis(50);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match self.state {
                        AppState::LanguageSelection => {
                            match key.code {
                                KeyCode::Char('1') => self.start_game(Language::English),
                                KeyCode::Char('2') => self.start_game(Language::Spanish),
                                KeyCode::Char('3') => self.start_game(Language::Portuguese),
                                KeyCode::Char('4') => self.start_game(Language::German),
                                KeyCode::Char('5') => self.start_game(Language::Dutch),
                                KeyCode::Esc => self.should_quit = true,
                                _ => {}
                            }
                        }
                        AppState::Playing => {
                            if key.code == KeyCode::Esc {
                                self.should_quit = true;
                            } else if let KeyCode::Char(c) = key.code {
                                if c.is_alphabetic() {
                                    self.make_guess(c);
                                }
                            }
                        }
                        AppState::GameOver(_) => {
                            if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                                self.state = AppState::LanguageSelection;
                                self.game = None;
                                self.lang = None;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        self.last_tick = now;

        if let AppState::Playing = self.state {
            self.timer -= dt;
            if self.timer <= 0.0 {
                if let Some(game) = &mut self.game {
                    game.decrease_attempts();
                    self.timer = 30.0;
                    if game.is_lost() {
                        self.state = AppState::GameOver(false);
                    }
                }
            }
        }
    }

    fn start_game(&mut self, language: Language) {
        let lang = Lang::from_language(language);
        let game = Hangman::random(lang.movies);
        self.lang = Some(lang);
        self.game = Some(game);
        self.timer = 30.0;
        self.error_msg = None;
        self.state = AppState::Playing;
    }

    fn make_guess(&mut self, letter: char) {
        let mut won = false;
        let mut lost = false;
        let mut err = None;

        if let Some(game) = &mut self.game {
            match game.guess(letter) {
                Ok(_) => {
                    won = game.is_won();
                    lost = game.is_lost();
                }
                Err(e) => err = Some(e),
            }
        }

        if err.is_none() {
            self.timer = 30.0;
            self.error_msg = None;
            if won {
                self.state = AppState::GameOver(true);
            } else if lost {
                self.state = AppState::GameOver(false);
            }
        } else {
            let lang = self.lang.as_ref().unwrap();
            self.error_msg = Some(match err.unwrap() {
                GuessError::NotLetter => lang.error_not_letter.to_string(),
                GuessError::AlreadyGuessed => lang.error_already_guessed.to_string(),
            });
        }
    }

    fn draw(&self, f: &mut Frame) {
        let area = f.size();

        match self.state {
            AppState::LanguageSelection => {
                let text = vec![
                    Line::from(vec![Span::styled("Select Language", Style::default().fg(Color::Cyan))]),
                    Line::from(""),
                    Line::from("1. English"),
                    Line::from("2. Español"),
                    Line::from("3. Português"),
                    Line::from("4. Deutsch"),
                    Line::from("5. Nederlands"),
                    Line::from(""),
                    Line::from(vec![Span::styled("Press 1-5 to select, or ESC to quit", Style::default().fg(Color::DarkGray))]),
                ];
                let p = Paragraph::new(text).alignment(Alignment::Center).block(
                    Block::default().borders(Borders::ALL).title("Hangman"),
                );
                f.render_widget(p, area);
            }
            AppState::Playing => {
                if let (Some(lang), Some(game)) = (&self.lang, &self.game) {
                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),  // Title
                            Constraint::Length(10), // Hangman art
                            Constraint::Length(2),  // Word
                            Constraint::Length(2),  // Guessed
                            Constraint::Length(2),  // Attempts & Time
                            Constraint::Length(2),  // Error msg
                            Constraint::Min(1),     // Prompt
                        ])
                        .split(area);

                    // Title
                    f.render_widget(
                        Paragraph::new(lang.title).alignment(Alignment::Center).style(Style::default().fg(Color::Cyan)),
                        layout[0],
                    );

                    // Art
                    let stage = game.max_attempts() - game.attempts_left();
                    let art = HANGMAN_ART[stage.min(6)];
                    f.render_widget(
                        Paragraph::new(art).alignment(Alignment::Center).style(Style::default().fg(Color::Yellow)),
                        layout[1],
                    );

                    // Word
                    let word_text = vec![Line::from(vec![
                        Span::raw(lang.word_label),
                        Span::styled(game.display_word(), Style::default().fg(Color::Green)),
                    ])];
                    f.render_widget(Paragraph::new(word_text).alignment(Alignment::Center), layout[2]);

                    // Guessed
                    let guessed_text = vec![Line::from(vec![
                        Span::raw(lang.guessed_label),
                        Span::styled(game.display_guessed(), Style::default().fg(Color::Magenta)),
                    ])];
                    f.render_widget(Paragraph::new(guessed_text).alignment(Alignment::Center), layout[3]);

                    // Stats (Attempts & Timer)
                    let stats_text = vec![Line::from(vec![
                        Span::raw(lang.attempts_label),
                        Span::styled(game.attempts_left().to_string(), Style::default().fg(Color::Red)),
                        Span::raw(" | "),
                        Span::raw(lang.time_left_label),
                        Span::styled(format!("{:.1}s", self.timer), Style::default().fg(Color::LightCyan)),
                    ])];
                    f.render_widget(Paragraph::new(stats_text).alignment(Alignment::Center), layout[4]);

                    // Error msg
                    if let Some(err) = &self.error_msg {
                        f.render_widget(
                            Paragraph::new(err.as_str()).alignment(Alignment::Center).style(Style::default().fg(Color::LightRed)),
                            layout[5],
                        );
                    }

                    // Prompt
                    f.render_widget(
                        Paragraph::new(lang.prompt_guess).alignment(Alignment::Center).style(Style::default().fg(Color::White)),
                        layout[6],
                    );
                }
            }
            AppState::GameOver(won) => {
                if let (Some(lang), Some(game)) = (&self.lang, &self.game) {
                    let msg = if won {
                        Span::styled(lang.win_msg, Style::default().fg(Color::Green))
                    } else {
                        Span::styled(lang.lose_msg, Style::default().fg(Color::Red))
                    };

                    let text = vec![
                        Line::from(msg),
                        Line::from(""),
                        Line::from(vec![
                            Span::raw("Word: "),
                            Span::styled(game.word(), Style::default().fg(Color::Yellow)),
                        ]),
                        Line::from(""),
                        Line::from(vec![Span::styled(lang.press_enter, Style::default().fg(Color::DarkGray))]),
                    ];

                    let p = Paragraph::new(text).alignment(Alignment::Center).block(
                        Block::default().borders(Borders::ALL).title(lang.title),
                    );
                    f.render_widget(p, area);
                }
            }
        }
    }
}

const HANGMAN_ART: [&str; 7] = [
    // Stage 0
    "  +---+\n  |   |\n      |\n      |\n      |\n      |\n=========",
    // Stage 1
    "  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=========",
    // Stage 2
    "  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=========",
    // Stage 3
    "  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=========",
    // Stage 4
    "  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=========",
    // Stage 5
    "  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=========",
    // Stage 6
    "  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=========",
];
