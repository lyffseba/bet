use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::game::{GuessError, Hangman};
use crate::lang::{Lang, Language};
use crate::tictactoe::{Cell, GameStatus, Player, TicTacToe};

fn is_utf8_supported() -> bool {
    #[cfg(windows)]
    {
        true
    }
    #[cfg(not(windows))]
    {
        for var in ["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                let val_lower = val.to_lowercase();
                if val_lower.contains("utf-8") || val_lower.contains("utf8") {
                    return true;
                }
            }
        }
        false
    }
}

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
    GameSelection,
    Playing,        // Hangman
    GameOver(bool), // Hangman true if won, false if lost
    PlayingTicTacToe,
    DiscordQr,
    EasterEgg,
}

pub struct App {
    pub state: AppState,
    pub lang: Option<Lang>,
    pub game: Option<Hangman>,
    pub tictactoe: Option<TicTacToe>,
    pub tictactoe_cursor: usize,
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
            tictactoe: None,
            tictactoe_cursor: 4, // center
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
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Press {
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('c')
                        {
                            self.should_quit = true;
                            return Ok(());
                        }

                        if !matches!(self.state, AppState::Playing)
                            && !matches!(self.state, AppState::PlayingTicTacToe)
                            && !matches!(self.state, AppState::EasterEgg)
                            && let KeyCode::Char(c) = key.code
                        {
                            self.easter_egg_buffer.push(c);
                            if self.easter_egg_buffer.len() > 50 {
                                self.easter_egg_buffer = self
                                    .easter_egg_buffer
                                    .chars()
                                    .skip(self.easter_egg_buffer.chars().count() - 20)
                                    .collect();
                            }
                            if self.easter_egg_buffer.to_lowercase().ends_with("lyff") {
                                let url = "https://lyffseba.xyz";
                                #[cfg(target_os = "linux")]
                                {
                                    if std::process::Command::new("gio")
                                        .args(["open", url])
                                        .spawn()
                                        .is_err()
                                    {
                                        let _ = open::that_detached(url);
                                    }
                                }
                                #[cfg(not(target_os = "linux"))]
                                {
                                    let _ = open::that_detached(url);
                                }

                                self.easter_egg_buffer.clear();
                                self.state = AppState::EasterEgg;
                                return Ok(());
                            }
                        }

                        match self.state {
                            AppState::LanguageSelection => match key.code {
                                KeyCode::Char('1') => self.select_language(Language::English),
                                KeyCode::Char('2') => self.select_language(Language::Spanish),
                                KeyCode::Char('3') => self.select_language(Language::Portuguese),
                                KeyCode::Char('4') => self.select_language(Language::German),
                                KeyCode::Char('5') => self.select_language(Language::Dutch),
                                KeyCode::Char('6') => self.state = AppState::DiscordQr,
                                KeyCode::Esc => self.should_quit = true,
                                _ => {}
                            },
                            AppState::GameSelection => match key.code {
                                KeyCode::Char('1') => self.start_hangman(),
                                KeyCode::Char('2') => self.start_tictactoe(),
                                KeyCode::Char('3') | KeyCode::Esc => {
                                    self.state = AppState::LanguageSelection;
                                    self.lang = None;
                                }
                                _ => {}
                            },
                            AppState::Playing => {
                                if key.code == KeyCode::Esc {
                                    self.state = AppState::GameSelection;
                                } else if let KeyCode::Char(c) = key.code
                                    && c.is_alphabetic()
                                {
                                    self.make_guess_hangman(c);
                                }
                            }
                            AppState::PlayingTicTacToe => {
                                if key.code == KeyCode::Esc {
                                    self.state = AppState::GameSelection;
                                } else if let Some(ttt) = &mut self.tictactoe {
                                    if ttt.status == GameStatus::Ongoing {
                                        match key.code {
                                            KeyCode::Up => {
                                                if self.tictactoe_cursor >= 3 {
                                                    self.tictactoe_cursor -= 3;
                                                }
                                            }
                                            KeyCode::Down => {
                                                if self.tictactoe_cursor <= 5 {
                                                    self.tictactoe_cursor += 3;
                                                }
                                            }
                                            KeyCode::Left => {
                                                if !self.tictactoe_cursor.is_multiple_of(3) {
                                                    self.tictactoe_cursor -= 1;
                                                }
                                            }
                                            KeyCode::Right => {
                                                if self.tictactoe_cursor % 3 != 2 {
                                                    self.tictactoe_cursor += 1;
                                                }
                                            }
                                            KeyCode::Enter | KeyCode::Char(' ') => {
                                                ttt.make_move(self.tictactoe_cursor);
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        if key.code == KeyCode::Enter
                                            || key.code == KeyCode::Char(' ')
                                        {
                                            ttt.reset_game();
                                        }
                                    }
                                }
                            }
                            AppState::GameOver(_) => {
                                if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                                    self.state = AppState::GameSelection;
                                    self.game = None;
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
                Event::Resize(_, _) => {
                    // ratatui handles resize gracefully on the next draw call
                }
                _ => {}
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
            if self.timer <= 0.0
                && let Some(game) = &mut self.game
            {
                game.decrease_attempts();
                self.timer = 30.0;
                if game.is_lost() {
                    self.state = AppState::GameOver(false);
                }
            }
        }
    }

    fn select_language(&mut self, language: Language) {
        self.lang = Some(Lang::from_language(language));
        self.state = AppState::GameSelection;
    }

    fn start_hangman(&mut self) {
        if let Some(lang) = &self.lang {
            let game = Hangman::random(lang.movies);
            self.game = Some(game);
            self.timer = 30.0;
            self.error_msg = None;
            self.state = AppState::Playing;
        }
    }

    fn start_tictactoe(&mut self) {
        self.tictactoe = Some(TicTacToe::new());
        self.tictactoe_cursor = 4;
        self.state = AppState::PlayingTicTacToe;
    }

    fn make_guess_hangman(&mut self, letter: char) {
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
        } else if let Some(lang) = &self.lang
            && let Some(e) = err
        {
            self.error_msg = Some(match e {
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
                    Line::from(vec![Span::styled(
                        "Select Language",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from("1. English"),
                    Line::from("2. Español"),
                    Line::from("3. Português"),
                    Line::from("4. Deutsch"),
                    Line::from("5. Nederlands"),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "6. Join our Discord! (QR)",
                        Style::default().fg(Color::LightMagenta),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Press 1-6 to select, or ESC to quit",
                        Style::default().fg(Color::DarkGray),
                    )]),
                ];
                let p = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("bet"));
                f.render_widget(Clear, rect); // Clear background
                f.render_widget(p, rect);

                let bottom_rect =
                    ratatui::layout::Rect::new(0, area.height.saturating_sub(1), area.width, 1);
                let watermark = Paragraph::new(Span::styled(
                    "lyffseba.xyz",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ))
                .alignment(Alignment::Right);
                f.render_widget(watermark, bottom_rect);
            }
            AppState::GameSelection => {
                if let Some(lang) = &self.lang {
                    let rect = centered_rect(70, 60, area);
                    let text = vec![
                        Line::from(vec![Span::styled(
                            lang.menu_game_selection,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )]),
                        Line::from(""),
                        Line::from(lang.menu_hangman),
                        Line::from(lang.menu_tictactoe),
                        Line::from(""),
                        Line::from(vec![Span::styled(
                            lang.menu_go_back,
                            Style::default().fg(Color::DarkGray),
                        )]),
                    ];
                    let p = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL).title("bet"));
                    f.render_widget(Clear, rect);
                    f.render_widget(p, rect);
                }
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

                    f.render_widget(Clear, game_area);

                    // Title
                    f.render_widget(
                        Paragraph::new(lang.title)
                            .alignment(Alignment::Center)
                            .style(
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        layout[0],
                    );

                    // Art
                    let stage = game.max_attempts() - game.attempts_left();
                    let art = HANGMAN_ART[stage.min(6)];
                    f.render_widget(
                        Paragraph::new(art)
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::Yellow)),
                        layout[1],
                    );

                    // Word
                    let word_text = vec![Line::from(vec![
                        Span::raw(lang.word_label),
                        Span::styled(
                            game.display_word(),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])];
                    f.render_widget(
                        Paragraph::new(word_text).alignment(Alignment::Center),
                        layout[2],
                    );

                    // Guessed
                    let guessed_text = vec![Line::from(vec![
                        Span::raw(lang.guessed_label),
                        Span::styled(game.display_guessed(), Style::default().fg(Color::Magenta)),
                    ])];
                    f.render_widget(
                        Paragraph::new(guessed_text).alignment(Alignment::Center),
                        layout[3],
                    );

                    // Stats (Attempts & Timer)
                    let timer_color = if self.timer <= 3.0 {
                        Color::Red
                    } else if self.timer <= 10.0 {
                        Color::Yellow
                    } else {
                        Color::White
                    };
                    let stats_text = vec![Line::from(vec![
                        Span::raw(lang.attempts_label),
                        Span::styled(
                            game.attempts_left().to_string(),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("   |   "),
                        Span::raw(lang.time_left_label),
                        Span::styled(
                            format!("{:.1}s", self.timer),
                            Style::default()
                                .fg(timer_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])];
                    f.render_widget(
                        Paragraph::new(stats_text).alignment(Alignment::Center),
                        layout[4],
                    );

                    // Error msg
                    if let Some(err) = &self.error_msg {
                        f.render_widget(
                            Paragraph::new(err.as_str())
                                .alignment(Alignment::Center)
                                .style(Style::default().fg(Color::LightRed)),
                            layout[5],
                        );
                    }

                    // Prompt
                    f.render_widget(
                        Paragraph::new(lang.prompt_guess)
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::White)),
                        layout[6],
                    );
                }
            }
            AppState::PlayingTicTacToe => {
                if let (Some(_lang), Some(ttt)) = (&self.lang, &self.tictactoe) {
                    let rect = centered_rect(70, 70, area);
                    f.render_widget(Clear, rect);

                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // Title
                            Constraint::Length(7), // Board
                            Constraint::Length(2), // Status
                            Constraint::Length(2), // Stats
                            Constraint::Min(1),    // Instructions
                        ])
                        .split(rect);

                    // Title
                    f.render_widget(
                        Paragraph::new("TIC-TAC-TOE")
                            .alignment(Alignment::Center)
                            .style(
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        layout[0],
                    );

                    // Board
                    let mut board_lines = vec![];
                    for row in 0..3 {
                        let mut line_spans = vec![];
                        for col in 0..3 {
                            let idx = row * 3 + col;
                            let cell_str = match ttt.board[idx] {
                                Cell::Empty => "   ",
                                Cell::Occupied(Player::X) => " X ",
                                Cell::Occupied(Player::O) => " O ",
                            };

                            let mut style = Style::default();
                            if ttt.board[idx] == Cell::Occupied(Player::X) {
                                style = style.fg(Color::Yellow);
                            } else if ttt.board[idx] == Cell::Occupied(Player::O) {
                                style = style.fg(Color::Magenta);
                            }

                            if ttt.status == GameStatus::Ongoing && idx == self.tictactoe_cursor {
                                style = style.bg(Color::DarkGray);
                            }

                            line_spans.push(Span::styled(cell_str, style));

                            if col < 2 {
                                line_spans.push(Span::raw("|"));
                            }
                        }
                        board_lines.push(Line::from(line_spans));
                        if row < 2 {
                            board_lines.push(Line::from("---+---+---"));
                        }
                    }

                    f.render_widget(
                        Paragraph::new(board_lines).alignment(Alignment::Center),
                        layout[1],
                    );

                    // Status
                    let status_msg = match ttt.status {
                        GameStatus::Ongoing => {
                            Span::styled("Your turn (X)", Style::default().fg(Color::White))
                        }
                        GameStatus::Win(Player::X) => Span::styled(
                            "You win!",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        GameStatus::Win(Player::O) => Span::styled(
                            "Computer wins!",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        GameStatus::Draw => Span::styled(
                            "Draw!",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    };
                    f.render_widget(
                        Paragraph::new(Line::from(status_msg)).alignment(Alignment::Center),
                        layout[2],
                    );

                    // Stats
                    let stats = format!(
                        "Wins: {} | Losses: {} | Draws: {}",
                        ttt.wins, ttt.losses, ttt.draws
                    );
                    f.render_widget(
                        Paragraph::new(stats)
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::Cyan)),
                        layout[3],
                    );

                    // Instructions
                    let instructions = if ttt.status == GameStatus::Ongoing {
                        "Arrow keys to move, Enter/Space to place X. ESC to go back."
                    } else {
                        "Press Enter/Space to play again. ESC to go back."
                    };
                    f.render_widget(
                        Paragraph::new(instructions)
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::DarkGray)),
                        layout[4],
                    );
                }
            }
            AppState::GameOver(won) => {
                if let (Some(lang), Some(game)) = (&self.lang, &self.game) {
                    let rect = centered_rect(70, 50, area);
                    let msg = if won {
                        Span::styled(
                            lang.win_msg,
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::styled(
                            lang.lose_msg,
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        )
                    };

                    let text = vec![
                        Line::from(msg),
                        Line::from(""),
                        Line::from(vec![
                            Span::raw(lang.word_label),
                            Span::styled(
                                game.word(),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(""),
                        Line::from(vec![Span::styled(
                            lang.press_enter,
                            Style::default().fg(Color::DarkGray),
                        )]),
                    ];

                    let p = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL).title(lang.title));
                    f.render_widget(Clear, rect); // Clear background
                    f.render_widget(p, rect);
                }
            }
            AppState::DiscordQr => {
                let rect = centered_rect(80, 80, area);

                let url = "https://discord.gg/MF6fMFURyC";
                let code_res = qrcode::QrCode::new(url);

                let mut qr_lines = Vec::new();

                if let Ok(code) = code_res {
                    let colors = code.to_colors();
                    let width = code.width();

                    let is_utf8 = is_utf8_supported();

                    if is_utf8 {
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
                    } else {
                        // ASCII fallback (2 chars per block to maintain roughly square aspect ratio)
                        let quiet_line = " ".repeat((width + 4) * 2);
                        qr_lines.push(quiet_line.clone());
                        for y in 0..width {
                            let mut line = String::from("    "); // Left quiet zone (4 spaces)
                            for x in 0..width {
                                let dark = colors[y * width + x] == qrcode::Color::Dark;
                                line.push_str(if dark { "##" } else { "  " });
                            }
                            line.push_str("    "); // Right quiet zone
                            qr_lines.push(line);
                        }
                        qr_lines.push(quiet_line);
                    }
                } else {
                    qr_lines.push(String::from("Error generating QR code"));
                }

                let mut lines = vec![
                    Line::from(vec![Span::styled(
                        "Join our Discord group: BET",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Hangman and more classic games incoming!",
                        Style::default().fg(Color::White),
                    )]),
                    Line::from(""),
                ];

                for line in qr_lines {
                    lines.push(Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::White).bg(Color::Black),
                    )));
                }

                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Press ESC or Enter to go back",
                    Style::default().fg(Color::DarkGray),
                )]));

                let p = Paragraph::new(lines)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("Discord"));
                f.render_widget(Clear, rect);
                f.render_widget(p, rect);
            }
            AppState::EasterEgg => {
                let color = Color::Rgb(230, 235, 240);
                let art = if is_utf8_supported() {
                    r#"
██╗     ██╗   ██╗███████╗███████╗
██║     ╚██╗ ██╔╝██╔════╝██╔════╝
██║      ╚████╔╝ █████╗  █████╗  
██║       ╚██╔╝  ██╔══╝  ██╔══╝  
███████╗   ██║   ██║     ██║     
╚══════╝   ╚═╝   ╚═╝     ╚═╝     "#
                } else {
                    r#"
L       Y   Y FFFFF FFFFF
L        Y Y  F     F    
L         Y   FFF   FFF  
L         Y   F     F    
LLLLL     Y   F     F    "#
                };

                let lines: Vec<Line> = art
                    .lines()
                    .skip(1) // Skip empty first line from raw string literal
                    .map(|l| {
                        Line::from(Span::styled(
                            l,
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ))
                    })
                    .collect();

                let mut text = lines;
                text.push(Line::from(""));
                text.push(Line::from(Span::styled(
                    "Love Yourself And Face FEAR",
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )));
                text.push(Line::from(""));
                text.push(Line::from(Span::styled(
                    "Press any key to return...",
                    Style::default().fg(Color::DarkGray),
                )));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_detection() {
        // Without mocked environment variables, this should run without panicking
        // Testing exact output would require environment manipulation which is messy in unit tests
        let _ = is_utf8_supported();
    }

    #[test]
    fn test_easter_egg_buffer() {
        let mut app = App::new();

        // Feed in some characters
        app.easter_egg_buffer.push('a');
        app.easter_egg_buffer.push('b');
        app.easter_egg_buffer.push('c');

        assert_eq!(app.easter_egg_buffer, "abc");

        // Push over 50 characters to trigger the truncation
        for _ in 0..55 {
            app.easter_egg_buffer.push('x');
        }

        // Should truncate appropriately
        app.easter_egg_buffer = app
            .easter_egg_buffer
            .chars()
            .skip(app.easter_egg_buffer.chars().count() - 20)
            .collect();
        assert_eq!(app.easter_egg_buffer.len(), 20);
    }
}
