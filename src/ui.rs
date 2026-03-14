use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};

use crate::game::{GuessError, Hangman};
use crate::lang::{Lang, Language};

// Helper function to center a rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub enum AppState {
    LanguageSelection,
    Playing,
    GameOver(bool), // true if won, false if lost
    DiscordQr,
    EasterEgg,
}

pub struct App {
    pub state: AppState,
    pub lang: Option<Lang>,
    pub game: Option<Hangman>,
    pub timer: f64,
    pub last_tick: Instant,
    pub should_quit: bool,
    pub error_msg: Option<String>,
    pub easter_egg_buffer: String,
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
            easter_egg_buffer: String::new(),
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
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        self.should_quit = true;
                        return Ok(());
                    }

                    if !matches!(self.state, AppState::Playing) && !matches!(self.state, AppState::EasterEgg) {
                        if let KeyCode::Char(c) = key.code {
                            self.easter_egg_buffer.push(c);
                            if self.easter_egg_buffer.len() > 50 {
                                self.easter_egg_buffer = self.easter_egg_buffer.chars().skip(self.easter_egg_buffer.chars().count() - 20).collect();
                            }
                            if self.easter_egg_buffer.to_lowercase().ends_with("lyff") {
                                #[cfg(target_os = "linux")]
                                {
                                    // Best effort on Linux using gio open which integrates tightly with GNOME/GTK
                                    let _ = std::process::Command::new("gio")
                                        .args(["open", "https://lyffseba.xyz"])
                                        .spawn();
                                }
                                #[cfg(not(target_os = "linux"))]
                                {
                                    let _ = open::that_detached("https://lyffseba.xyz");
                                }
                                
                                self.easter_egg_buffer.clear();
                                self.state = AppState::EasterEgg;
                                return Ok(()); // Avoid falling through to other key handlers
                            }
                        }
                    }

                    match self.state {
                        AppState::LanguageSelection => {
                            match key.code {
                                KeyCode::Char('1') => self.start_game(Language::English),
                                KeyCode::Char('2') => self.start_game(Language::Spanish),
                                KeyCode::Char('3') => self.start_game(Language::Portuguese),
                                KeyCode::Char('4') => self.start_game(Language::German),
                                KeyCode::Char('5') => self.start_game(Language::Dutch),
                                KeyCode::Char('6') => self.state = AppState::DiscordQr,
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
                        AppState::DiscordQr => {
                            if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                                self.state = AppState::LanguageSelection;
                            }
                        }
                        AppState::EasterEgg => {
                            self.state = AppState::LanguageSelection;
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
                let rect = centered_rect(70, 60, area);
                let text = vec![
                    Line::from(vec![Span::styled("Select Language", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
                    Line::from(""),
                    Line::from("1. English"),
                    Line::from("2. Español"),
                    Line::from("3. Português"),
                    Line::from("4. Deutsch"),
                    Line::from("5. Nederlands"),
                    Line::from(""),
                    Line::from(vec![Span::styled("6. Join our Discord! (QR)", Style::default().fg(Color::LightMagenta))]),
                    Line::from(""),
                    Line::from(vec![Span::styled("Press 1-6 to select, or ESC to quit", Style::default().fg(Color::DarkGray))]),
                ];
                let p = Paragraph::new(text).alignment(Alignment::Center).block(
                    Block::default().borders(Borders::ALL).title("Hangman"),
                );
                f.render_widget(Clear, rect); // Clear background
                f.render_widget(p, rect);

                // Extremely subtle cursive watermark/easter egg in the bottom right corner
                let bottom_rect = ratatui::layout::Rect::new(0, area.height.saturating_sub(1), area.width, 1);
                let watermark = Paragraph::new(Span::styled("lyffseba.xyz", Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)))
                    .alignment(Alignment::Right);
                f.render_widget(watermark, bottom_rect);
            }
            AppState::Playing => {
                if let (Some(lang), Some(game)) = (&self.lang, &self.game) {
                    let game_area = centered_rect(90, 85, area);
                    
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
                        .split(game_area);

                    f.render_widget(Clear, game_area); // Clear background

                    // Title
                    f.render_widget(
                        Paragraph::new(lang.title).alignment(Alignment::Center).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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
                        Span::styled(game.display_word(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ])];
                    f.render_widget(Paragraph::new(word_text).alignment(Alignment::Center), layout[2]);

                    // Guessed
                    let guessed_text = vec![Line::from(vec![
                        Span::raw(lang.guessed_label),
                        Span::styled(game.display_guessed(), Style::default().fg(Color::Magenta)),
                    ])];
                    f.render_widget(Paragraph::new(guessed_text).alignment(Alignment::Center), layout[3]);

                    // Stats (Attempts & Timer)
                    let timer_color = if self.timer <= 3.0 { Color::Red } else if self.timer <= 10.0 { Color::Yellow } else { Color::White };
                    let stats_text = vec![Line::from(vec![
                        Span::raw(lang.attempts_label),
                        Span::styled(game.attempts_left().to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw("   |   "),
                        Span::raw(lang.time_left_label),
                        Span::styled(format!("{:.1}s", self.timer), Style::default().fg(timer_color).add_modifier(Modifier::BOLD)),
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
                    let rect = centered_rect(70, 50, area);
                    let msg = if won {
                        Span::styled(lang.win_msg, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled(lang.lose_msg, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    };

                    let text = vec![
                        Line::from(msg),
                        Line::from(""),
                        Line::from(vec![
                            Span::raw(lang.word_label),
                            Span::styled(game.word(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        ]),
                        Line::from(""),
                        Line::from(vec![Span::styled(lang.press_enter, Style::default().fg(Color::DarkGray))]),
                    ];

                    let p = Paragraph::new(text).alignment(Alignment::Center).block(
                        Block::default().borders(Borders::ALL).title(lang.title),
                    );
                    f.render_widget(Clear, rect); // Clear background
                    f.render_widget(p, rect);
                }
            }
            AppState::DiscordQr => {
                let rect = centered_rect(80, 80, area);
                
                let url = "https://discord.gg/MF6fMFURyC";
                let code = qrcode::QrCode::new(url).unwrap();
                let colors = code.to_colors();
                let width = code.width();

                let mut qr_lines = Vec::new();
                // Add a top and bottom quiet zone using spaces
                let quiet_line = " ".repeat(width + 4);
                qr_lines.push(quiet_line.clone());
                
                for y in (0..width).step_by(2) {
                    let mut line = String::from("  "); // Left quiet zone
                    for x in 0..width {
                        let top = colors[y * width + x] == qrcode::Color::Dark;
                        let bottom = if y + 1 < width {
                            colors[(y + 1) * width + x] == qrcode::Color::Dark
                        } else {
                            false
                        };
                        let c = match (top, bottom) {
                            (true, true) => '█',
                            (true, false) => '▀',
                            (false, true) => '▄',
                            (false, false) => ' ',
                        };
                        line.push(c);
                    }
                    line.push_str("  "); // Right quiet zone
                    qr_lines.push(line);
                }
                qr_lines.push(quiet_line);

                let mut lines = vec![
                    Line::from(vec![Span::styled("Join our Discord group: BET", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
                    Line::from(""),
                    Line::from(vec![Span::styled("Hangman and more classic games incoming!", Style::default().fg(Color::White))]),
                    Line::from(""),
                ];

                for line in qr_lines {
                    // Set black background for the QR code so the blocks are always visible
                    lines.push(Line::from(Span::styled(line, Style::default().fg(Color::White).bg(Color::Black))));
                }

                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled("Press ESC or Enter to go back", Style::default().fg(Color::DarkGray))]));

                let p = Paragraph::new(lines).alignment(Alignment::Center).block(
                    Block::default().borders(Borders::ALL).title("Discord"),
                );
                f.render_widget(Clear, rect);
                f.render_widget(p, rect);
            }
            AppState::EasterEgg => {
                let color = Color::Rgb(230, 235, 240);
                let art = r#"
██╗     ██╗   ██╗███████╗███████╗
██║     ╚██╗ ██╔╝██╔════╝██╔════╝
██║      ╚████╔╝ █████╗  █████╗  
██║       ╚██╔╝  ██╔══╝  ██╔══╝  
███████╗   ██║   ██║     ██║     
╚══════╝   ╚═╝   ╚═╝     ╚═╝     "#;
                
                let lines: Vec<Line> = art.lines()
                    .skip(1) // Skip empty first line from raw string literal
                    .map(|l| Line::from(Span::styled(l, Style::default().fg(color).add_modifier(Modifier::BOLD))))
                    .collect();
                
                let mut text = lines;
                text.push(Line::from(""));
                text.push(Line::from(Span::styled("Love Yourself And Face FEAR", Style::default().fg(color).add_modifier(Modifier::BOLD))));
                text.push(Line::from(""));
                text.push(Line::from(Span::styled("Press any key to return...", Style::default().fg(Color::DarkGray))));

                let p = Paragraph::new(text).alignment(Alignment::Center);
                
                // We center it vertically, taking ~12 lines
                let rect = centered_rect(80, 50, area);
                f.render_widget(Clear, area);
                f.render_widget(p, rect);
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
