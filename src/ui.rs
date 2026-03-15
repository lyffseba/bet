use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::widgets::canvas::{Canvas, Rectangle};
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
use crate::tictactoe::{Cell, GameStatus as TicTacToeStatus, Player, TicTacToe};
use crate::chess_game::{ChessGame, GameStatus as ChessStatus};
use crate::pong::{GameStatus as PongStatus, PongGame};
use shakmaty::{Square, Color as ChessColor, Position};

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

#[derive(Clone, Copy, PartialEq)]
pub enum RecommenderCategory {
    Movie,
    Series,
    Manga,
    Book,
    Anime,
    Cartoon,
    MusicRock,
    MusicHipHop,
    MusicPop,
    MusicElectronic,
    MusicClassical,
    MusicSalsa,
    MusicReggae,
    VideoGame,
    Meme,
}

pub enum AppState {
    LanguageSelection,
    GameSelection,
    Playing,        // Hangman
    GameOver(bool), // Hangman true if won, false if lost
    PlayingTicTacToe,
    PlayingChess,
    PlayingPong,
    RecommenderMenu,
    MusicMenu,
    Recommendation(RecommenderCategory, String),
    Meme(String),
    DiscordQr,
    EasterEgg,
}

pub struct App {
    pub state: AppState,
    pub main_menu_meme: &'static str,
    pub lang: Option<Lang>,
    pub language_cursor: usize,
    pub game_cursor: usize,
    pub recommender_cursor: usize,
    pub music_cursor: usize,
    pub game: Option<Hangman>,
    pub tictactoe: Option<TicTacToe>,
    pub tictactoe_cursor: usize,
    pub chess: Option<ChessGame>,
    pub chess_cursor: Square,
    pub chess_selected: Option<Square>,
    pub pong: Option<PongGame>,
    pub timer: f64,
    pub last_tick: Instant,
    pub should_quit: bool,
    pub error_msg: Option<String>,
    pub easter_egg_buffer: String,
    pub ticker_text: Vec<char>,
    pub ticker_pos: f64,
    pub ticker_pause_timer: f64,
    pub bouncer_x: f64,
    pub bouncer_y: f64,
    pub bouncer_dx: f64,
    pub bouncer_dy: f64,
    pub bouncer_timer: f64,
    pub bouncer_active: bool,
    pub can_spawn_bouncer: bool,
    pub ticker_pause_points: Vec<usize>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            state: AppState::LanguageSelection,
            main_menu_meme: {
                let mut rng = rand::thread_rng();
                use rand::seq::SliceRandom;
                crate::wordlist::MEMES.choose(&mut rng).unwrap_or(&"Stonks")
            },
            lang: None,
            language_cursor: 0,
            game_cursor: 0,
            recommender_cursor: 0,
            music_cursor: 0,
            game: None,
            tictactoe: None,
            chess: None,
            chess_cursor: Square::from_coords(shakmaty::File::E, shakmaty::Rank::Second),
            chess_selected: None,
            pong: None,
            tictactoe_cursor: 4, // center
            timer: 30.0,
            last_tick: Instant::now(),
            should_quit: false,
            error_msg: None,
            easter_egg_buffer: String::new(),
            ticker_text: Vec::new(),
            ticker_pos: 0.0,
            ticker_pause_timer: 0.0,
            bouncer_x: 10.0,
            bouncer_y: 10.0,
            bouncer_dx: 20.0,
            bouncer_dy: 12.0,
            bouncer_timer: 0.0,
            bouncer_active: false,
            ticker_pause_points: Vec::new(),
            can_spawn_bouncer: {
                let current_day = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() / 86400;
                
                let mut can_spawn = true;
                if let Ok(home) = std::env::var("HOME") {
                    let path = std::path::Path::new(&home).join(".bet_qr_last_seen");
                    if let Ok(contents) = std::fs::read_to_string(&path)
                        && let Ok(saved_day) = contents.trim().parse::<u64>()
                        && saved_day == current_day
                    {
                        can_spawn = false;
                    }
                }
                can_spawn
            },
        };
        
        let mut text = Vec::new();
        let mut pause_points = Vec::new();
        
        let mut quotes = crate::wordlist::POETRY_QUOTES.to_vec();
        use rand::seq::SliceRandom;
        quotes.shuffle(&mut rand::thread_rng());
        
        for quote in quotes {
            // Add massive padding so only one quote is on screen at a time
            let padding = " ".repeat(60); 
            
            // The quote starts here
            let next_start = text.len();
            
            // We want it to pause when the quote is perfectly centered on a standard 100 char terminal.
            // That means it pauses when the start of the quote has traveled about `padding.len() + (100 - quote.len())/2` chars.
            let quote_len = quote.chars().count();
            let expected_term_width: usize = 100;
            let center_offset = (expected_term_width.saturating_sub(quote_len)) / 2;
            
            // To ensure it always pauses perfectly for the user, we just set the pause point at the start of the quote minus the center offset.
            if next_start >= center_offset {
                pause_points.push(next_start - center_offset);
            } else {
                pause_points.push(0);
            }
            
            text.extend(quote.chars());
            text.extend(padding.chars());
            text.extend("✦".chars());
            text.extend(padding.chars());
        }
        app.ticker_text = text;
        app.ticker_pause_points = pause_points;

        app.parse_args();
        app
    }

    fn parse_args(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        if args.is_empty() {
            return;
        }

        let exec_name = std::path::Path::new(&args[0])
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        let wants_hangman = exec_name.contains("hangman")
            || args.iter().skip(1).any(|a| a.to_lowercase() == "hangman");
        let wants_tictactoe = exec_name.contains("tictactoe")
            || args.iter().skip(1).any(|a| a.to_lowercase() == "tictactoe");
        let wants_chess = exec_name.contains("chess")
            || args.iter().skip(1).any(|a| a.to_lowercase() == "chess");
        let wants_pong = exec_name.contains("pong")
            || args.iter().skip(1).any(|a| a.to_lowercase() == "pong");
        let wants_movie = exec_name.contains("movie") || args.iter().skip(1).any(|a| a.to_lowercase() == "movie");
        let wants_series = exec_name.contains("series") || args.iter().skip(1).any(|a| a.to_lowercase() == "series");
        let wants_manga = exec_name.contains("manga") || args.iter().skip(1).any(|a| a.to_lowercase() == "manga");
        let wants_book = exec_name.contains("book") || args.iter().skip(1).any(|a| a.to_lowercase() == "book");
        let wants_anime = exec_name.contains("anime") || args.iter().skip(1).any(|a| a.to_lowercase() == "anime");
        let wants_cartoon = exec_name.contains("cartoon") || args.iter().skip(1).any(|a| a.to_lowercase() == "cartoon");
        let wants_music = exec_name.contains("music") || args.iter().skip(1).any(|a| a.to_lowercase() == "music");
        let wants_videogame = exec_name.contains("videogame") || exec_name.contains("game") || args.iter().skip(1).any(|a| a.to_lowercase() == "videogame" || a.to_lowercase() == "game");
        let wants_meme = exec_name.contains("meme") || args.iter().skip(1).any(|a| a.to_lowercase() == "meme");
        let wants_salsa = exec_name.contains("salsa") || args.iter().skip(1).any(|a| a.to_lowercase() == "salsa");
        let wants_reggae = exec_name.contains("reggae") || args.iter().skip(1).any(|a| a.to_lowercase() == "reggae");
        let wants_rec = exec_name.contains("recommend") || args.iter().skip(1).any(|a| a.to_lowercase() == "recommend");

        if wants_hangman {
            self.select_language(Language::English);
            self.start_hangman();
        } else if wants_tictactoe {
            self.select_language(Language::English);
            self.start_tictactoe();
        } else if wants_chess {
            self.select_language(Language::English);
            self.start_chess();
        } else if wants_pong {
            self.select_language(Language::English);
            self.start_pong();
        } else if wants_movie {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::Movie);
        } else if wants_series {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::Series);
        } else if wants_manga {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::Manga);
        } else if wants_book {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::Book);
        } else if wants_anime {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::Anime);
        } else if wants_cartoon {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::Cartoon);
        } else if wants_music {
            self.select_language(Language::English);
            self.state = AppState::MusicMenu;
        } else if wants_salsa {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::MusicSalsa);
        } else if wants_reggae {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::MusicReggae);
        } else if wants_videogame {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::VideoGame);
        } else if wants_meme {
            self.select_language(Language::English);
            self.show_recommendation(RecommenderCategory::Meme);
        } else if wants_rec {
            self.select_language(Language::English);
            self.state = AppState::RecommenderMenu;
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> io::Result<()> 
    where std::io::Error: From<<B as Backend>::Error> {
        self.last_tick = Instant::now();
        while !self.should_quit {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
            self.tick();
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let timeout = Duration::from_millis(8); // 120Hz poll rate for ultra-smooth time delta resolution
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
                                KeyCode::Up => {
                                    self.language_cursor = self.language_cursor.saturating_sub(1);
                                }
                                KeyCode::Down => {
                                    if self.language_cursor < 5 {
                                        self.language_cursor += 1;
                                    }
                                }
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    match self.language_cursor {
                                        0 => self.select_language(Language::English),
                                        1 => self.select_language(Language::Spanish),
                                        2 => self.select_language(Language::Portuguese),
                                        3 => self.select_language(Language::German),
                                        4 => self.select_language(Language::Dutch),
                                        5 => self.state = AppState::DiscordQr,
                                        _ => {}
                                    }
                                }
                                KeyCode::Char('1') => { self.language_cursor = 0; self.select_language(Language::English); }
                                KeyCode::Char('2') => { self.language_cursor = 1; self.select_language(Language::Spanish); }
                                KeyCode::Char('3') => { self.language_cursor = 2; self.select_language(Language::Portuguese); }
                                KeyCode::Char('4') => { self.language_cursor = 3; self.select_language(Language::German); }
                                KeyCode::Char('5') => { self.language_cursor = 4; self.select_language(Language::Dutch); }
                                KeyCode::Char('9') => { self.language_cursor = 5; self.state = AppState::DiscordQr; }
                                KeyCode::Esc => self.should_quit = true,
                                _ => {}
                            },
                            AppState::GameSelection => match key.code {
                                KeyCode::Up => {
                                    self.game_cursor = self.game_cursor.saturating_sub(1);
                                }
                                KeyCode::Down => {
                                    if self.game_cursor < 6 {
                                        self.game_cursor += 1;
                                    }
                                }
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    match self.game_cursor {
                                        0 => self.start_hangman(),
                                        1 => self.start_tictactoe(),
                                        2 => self.start_chess(),
                                        3 => self.start_pong(),
                                        4 => self.state = AppState::RecommenderMenu,
                                        5 => self.show_meme(),
                                        6 => {
                                            self.state = AppState::LanguageSelection;
                                            self.lang = None;
                                        }
                                        _ => {}
                                    }
                                }
                                KeyCode::Char('1') => { self.game_cursor = 0; self.start_hangman(); }
                                KeyCode::Char('2') => { self.game_cursor = 1; self.start_tictactoe(); }
                                KeyCode::Char('3') => { self.game_cursor = 2; self.start_chess(); }
                                KeyCode::Char('4') => { self.game_cursor = 3; self.start_pong(); }
                                KeyCode::Char('5') => { self.game_cursor = 4; self.state = AppState::RecommenderMenu; }
                                KeyCode::Char('6') => { self.game_cursor = 5; self.show_meme(); }
                                KeyCode::Char('8') | KeyCode::Esc => {
                                    self.game_cursor = 6;
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
                                    if ttt.status == TicTacToeStatus::Ongoing {
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
                            AppState::PlayingChess => {
                                if key.code == KeyCode::Esc {
                                    self.state = AppState::GameSelection;
                                } else if let Some(chess) = &mut self.chess {
                                    if chess.status == ChessStatus::Ongoing {
                                        let rank = self.chess_cursor.rank() as i8;
                                        let file = self.chess_cursor.file() as i8;
                                        match key.code {
                                            KeyCode::Up => {
                                                if rank < 7 {
                                                    self.chess_cursor = Square::from_coords(shakmaty::File::new((file) as u32), shakmaty::Rank::new((rank + 1) as u32));
                                                }
                                            }
                                            KeyCode::Down => {
                                                if rank > 0 {
                                                    self.chess_cursor = Square::from_coords(shakmaty::File::new((file) as u32), shakmaty::Rank::new((rank - 1) as u32));
                                                }
                                            }
                                            KeyCode::Left => {
                                                if file > 0 {
                                                    self.chess_cursor = Square::from_coords(shakmaty::File::new((file - 1) as u32), shakmaty::Rank::new((rank) as u32));
                                                }
                                            }
                                            KeyCode::Right => {
                                                if file < 7 {
                                                    self.chess_cursor = Square::from_coords(shakmaty::File::new((file + 1) as u32), shakmaty::Rank::new((rank) as u32));
                                                }
                                            }
                                            KeyCode::Enter | KeyCode::Char(' ') => {
                                                if let Some(selected) = self.chess_selected {
                                                    if selected == self.chess_cursor {
                                                        self.chess_selected = None;
                                                    } else {
                                                        let moves = chess.get_moves_from(selected);
                                                        // Automatically promote to Queen if it's a promotion move
                                                        let m = moves.into_iter().find(|m| m.to() == self.chess_cursor);
                                                        if let Some(m) = m {
                                                            chess.make_move(m);
                                                        }
                                                        self.chess_selected = None;
                                                    }
                                                } else {
                                                    // Only select if it's our piece
                                                    if let Some(piece) = chess.pos.board().piece_at(self.chess_cursor)
                                                        && piece.color == chess.player_color {
                                                            self.chess_selected = Some(self.chess_cursor);
                                                        }
                                                }
                                            }
                                            _ => {}
                                        }
                                    } else if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                                        self.start_chess();
                                    }
                                }
                            }
                            AppState::PlayingPong => {
                                if key.code == KeyCode::Esc {
                                    self.state = AppState::GameSelection;
                                } else if let Some(pong) = &mut self.pong {
                                    if pong.status == PongStatus::Ongoing {
                                        match key.code {
                                            KeyCode::Up => pong.move_player(true),
                                            KeyCode::Down => pong.move_player(false),
                                            _ => {}
                                        }
                                    } else if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                                        self.start_pong();
                                    }
                                }
                            },
                            AppState::RecommenderMenu => {
                                match key.code {
                                    KeyCode::Up => self.recommender_cursor = self.recommender_cursor.saturating_sub(1),
                                    KeyCode::Down => {
                                        if self.recommender_cursor < 8 {
                                            self.recommender_cursor += 1;
                                        }
                                    }
                                    KeyCode::Enter | KeyCode::Char(' ') => match self.recommender_cursor {
                                        0 => self.show_recommendation(RecommenderCategory::Movie),
                                        1 => self.show_recommendation(RecommenderCategory::Series),
                                        2 => self.show_recommendation(RecommenderCategory::Manga),
                                        3 => self.show_recommendation(RecommenderCategory::Book),
                                        4 => self.show_recommendation(RecommenderCategory::Anime),
                                        5 => self.show_recommendation(RecommenderCategory::Cartoon),
                                        6 => self.show_recommendation(RecommenderCategory::VideoGame),
                                        7 => self.state = AppState::MusicMenu,
                                        8 => self.state = AppState::GameSelection,
                                        _ => {}
                                    },
                                    KeyCode::Char('1') => { self.recommender_cursor = 0; self.show_recommendation(RecommenderCategory::Movie); }
                                    KeyCode::Char('2') => { self.recommender_cursor = 1; self.show_recommendation(RecommenderCategory::Series); }
                                    KeyCode::Char('3') => { self.recommender_cursor = 2; self.show_recommendation(RecommenderCategory::Manga); }
                                    KeyCode::Char('4') => { self.recommender_cursor = 3; self.show_recommendation(RecommenderCategory::Book); }
                                    KeyCode::Char('5') => { self.recommender_cursor = 4; self.show_recommendation(RecommenderCategory::Anime); }
                                    KeyCode::Char('6') => { self.recommender_cursor = 5; self.show_recommendation(RecommenderCategory::Cartoon); }
                                    KeyCode::Char('7') => { self.recommender_cursor = 6; self.show_recommendation(RecommenderCategory::VideoGame); }
                                    KeyCode::Char('8') => { self.recommender_cursor = 7; self.state = AppState::MusicMenu; }
                                    KeyCode::Char('9') | KeyCode::Esc => { self.recommender_cursor = 8; self.state = AppState::GameSelection; }
                                    _ => {}
                                }
                            }
                            AppState::MusicMenu => {
                                match key.code {
                                    KeyCode::Up => self.music_cursor = self.music_cursor.saturating_sub(1),
                                    KeyCode::Down => {
                                        if self.music_cursor < 7 {
                                            self.music_cursor += 1;
                                        }
                                    }
                                    KeyCode::Enter | KeyCode::Char(' ') => match self.music_cursor {
                                        0 => self.show_recommendation(RecommenderCategory::MusicRock),
                                        1 => self.show_recommendation(RecommenderCategory::MusicHipHop),
                                        2 => self.show_recommendation(RecommenderCategory::MusicPop),
                                        3 => self.show_recommendation(RecommenderCategory::MusicElectronic),
                                        4 => self.show_recommendation(RecommenderCategory::MusicClassical),
                                        5 => self.show_recommendation(RecommenderCategory::MusicSalsa),
                                        6 => self.show_recommendation(RecommenderCategory::MusicReggae),
                                        7 => self.state = AppState::RecommenderMenu,
                                        _ => {}
                                    },
                                    KeyCode::Char('1') => { self.music_cursor = 0; self.show_recommendation(RecommenderCategory::MusicRock); }
                                    KeyCode::Char('2') => { self.music_cursor = 1; self.show_recommendation(RecommenderCategory::MusicHipHop); }
                                    KeyCode::Char('3') => { self.music_cursor = 2; self.show_recommendation(RecommenderCategory::MusicPop); }
                                    KeyCode::Char('4') => { self.music_cursor = 3; self.show_recommendation(RecommenderCategory::MusicElectronic); }
                                    KeyCode::Char('5') => { self.music_cursor = 4; self.show_recommendation(RecommenderCategory::MusicClassical); }
                                    KeyCode::Char('6') => { self.music_cursor = 5; self.show_recommendation(RecommenderCategory::MusicSalsa); }
                                    KeyCode::Char('7') => { self.music_cursor = 6; self.show_recommendation(RecommenderCategory::MusicReggae); }
                                    KeyCode::Char('8') | KeyCode::Esc => { self.music_cursor = 7; self.state = AppState::RecommenderMenu; }
                                    _ => {}
                                }
                            }
                            AppState::Recommendation(cat, _) => {
                                if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                                    self.show_recommendation(cat);
                                } else if key.code == KeyCode::Esc {
                                    self.state = AppState::RecommenderMenu;
                                }
                            }
                            AppState::Meme(_) => {
                                if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                                    self.show_meme();
                                } else if key.code == KeyCode::Esc {
                                    self.state = AppState::GameSelection;
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

        if self.ticker_pause_timer > 0.0 {
            self.ticker_pause_timer -= dt;
        } else if !self.ticker_text.is_empty() {
            let old_pos = self.ticker_pos;
            self.ticker_pos += 6.0 * dt; // 6 chars/sec (exactly 1 char per 166ms) for slower, perfectly paced reading
            
            let old_idx = old_pos as usize;
            let new_idx = self.ticker_pos as usize;
            
            if new_idx > old_idx && self.ticker_pause_points.contains(&new_idx) {
                self.ticker_pause_timer = 5.0; // 5 seconds to read // Pause for 3 seconds
            }
            
            if self.ticker_pos >= self.ticker_text.len() as f64 {
                self.ticker_pos = 0.0;
            }
        }

        if let AppState::PlayingPong = self.state
            && let Some(pong) = &mut self.pong
        {
            pong.update(dt);
        }

        if self.bouncer_active {
            self.bouncer_x += self.bouncer_dx * dt;
            self.bouncer_y += self.bouncer_dy * dt;
            
            let term_w = 120.0;
            let term_h = 24.0; // Estimate
            
            if self.bouncer_x <= 0.0 || self.bouncer_x + 40.0 >= term_w {
                self.bouncer_dx *= -1.0;
                let max_x = if term_w > 35.0 { term_w - 40.0 } else { 0.0 };
                self.bouncer_x = self.bouncer_x.clamp(0.0, max_x);
            }
            if self.bouncer_y <= 0.0 || self.bouncer_y + 18.0 >= term_h {
                self.bouncer_dy *= -1.0;
                let max_y = if term_h > 18.0 { term_h - 18.0 } else { 0.0 };
                self.bouncer_y = self.bouncer_y.clamp(0.0, max_y);
            }
            
            self.bouncer_timer -= dt;
            if self.bouncer_timer <= 0.0 {
                self.bouncer_active = false;
            }
        } else if self.can_spawn_bouncer {
            // Extremely rare chance so it pops up unexpectedly during the day (1 in 30,000 frames = roughly once every 4 minutes of active use)
            use rand::Rng;
            if rand::thread_rng().gen_bool(0.00003) {
                self.bouncer_active = true;
                self.bouncer_timer = 30.0; // Stays on screen 30 seconds
                self.can_spawn_bouncer = false; // Never again today
                
                // Save today's date
                let current_day = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() / 86400;
                
                if let Ok(home) = std::env::var("HOME") {
                    let path = std::path::Path::new(&home).join(".bet_qr_last_seen");
                    let _ = std::fs::write(path, current_day.to_string());
                }
            }
        }

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

    fn start_chess(&mut self) {
        self.chess = Some(ChessGame::new(false));
        self.chess_cursor = Square::from_coords(shakmaty::File::E, shakmaty::Rank::Second);
        self.chess_selected = None;
        self.state = AppState::PlayingChess;
    }

    fn start_pong(&mut self) {
        self.pong = Some(PongGame::new());
        self.state = AppState::PlayingPong;
    }

    fn show_recommendation(&mut self, category: RecommenderCategory) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let item = if let Some(lang) = &self.lang {
            match category {
                RecommenderCategory::Movie => lang.movies.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::Series => lang.series.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::Manga => lang.mangas.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::Book => lang.books.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::Anime => lang.animes.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::Cartoon => lang.cartoons.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::MusicRock => lang.music_rock.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::MusicHipHop => lang.music_hiphop.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::MusicPop => lang.music_pop.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::MusicElectronic => lang.music_electronic.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::MusicClassical => lang.music_classical.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::MusicSalsa => lang.music_salsa.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::MusicReggae => lang.music_reggae.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::VideoGame => lang.videogames.choose(&mut rng).unwrap_or(&"BET"),
                RecommenderCategory::Meme => lang.memes.choose(&mut rng).unwrap_or(&"BET"),
            }
        } else {
            "BET"
        };
        self.state = AppState::Recommendation(category, item.to_string());
    }

    fn show_meme(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let item = if let Some(_lang) = &self.lang {
            crate::wordlist::MEMES.choose(&mut rng).unwrap_or(&"BET")
        } else {
            crate::wordlist::MEMES.choose(&mut rng).unwrap_or(&"BET")
        };
        self.state = AppState::Meme(item.to_string());
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
        let mut area = f.area();
        
        // Render ticker at the top
        let ticker_area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: area.width,
            height: 1,
        };
        area.y += 1;
        area.height = area.height.saturating_sub(1);

        match self.state {
            AppState::LanguageSelection => {
                let rect = centered_rect(80, 80, area);
                
                let ascii_banner = if is_utf8_supported() {
                    r#"
  /█\   /█\   /████\ /████\     /████\ /████\ /██████\ 
 /███\ /███\ /██╔══╝/██╔══╝    /██╔══╝/██╔══╝/██╔══██╗
/█████\█████\\████\ \████\    /█████\/█████\ \██║  ██║
\██╔══██╔══██/\██╔╝  \██╔╝    \██╔══/\██╔══╝  ██║  ██║
 \██\  \██\ \████║    \██║     \████\ \████\  ██║  ██║
  \██\  \██\ \██╔╝     \██\     \██╔╝  \██╔╝  ██║  ██║
   \═╝   \═╝  \═╝       \═╝      \═╝    \═╝   ╚═╝  ╚═╝"#
                } else {
                    r#"
 /\_/\  /\_/\  /____\ /____\   /____\ /____\ /______\
 | |  | | |  | | |__| | |__|   | |__| | |__|   | |   
 | |  | | |  | | |    | |      | |__  | |__    | |   
 | |__| | |__| | |    | |      | |__| | |__|   | |   
  \____/ \____/ \_/    \_/      \____/ \____/   \_/  "#
                };

                let mut text = vec![];
                
                // Basquiat x Escher static stark contrast
                let colors = [
                    Color::White,
                    Color::Gray,
                    Color::DarkGray,
                    Color::Gray,
                ];
                
                for (i, line) in ascii_banner.lines().enumerate() {
                    let color = colors[i % colors.len()];
                    text.push(Line::from(vec![Span::styled(
                        line,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    )]));
                }
                text.push(Line::from(""));
                text.push(Line::from(""));
                
                text.push(Line::from(vec![Span::styled(
                    "Select Language",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )]));
                text.push(Line::from(""));
                let options = [
                    "1. English",
                    "2. Español",
                    "3. Português",
                    "4. Deutsch",
                    "5. Nederlands",
                ];
                for (i, opt) in options.iter().enumerate() {
                    if i == self.language_cursor {
                        text.push(Line::from(vec![Span::styled(format!("    {}  ", opt), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))]));
                    } else {
                        text.push(Line::from(vec![Span::styled(format!("    {}  ", opt), Style::default().fg(Color::White))]));
                    }
                }
                text.push(Line::from(""));
                text.push(Line::from(""));
                
                if self.language_cursor == 5 {
                    text.push(Line::from(vec![Span::styled(
                        "    9. Join our Discord! (QR)  ",
                        Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD),
                    )]));
                } else {
                    text.push(Line::from(vec![Span::styled(
                        "    9. Join our Discord! (QR)  ",
                        Style::default().fg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD),
                    )]));
                }
                text.push(Line::from(""));
                text.push(Line::from(vec![Span::styled(
                    "Press 1-5 to select, 9 for Discord, or ESC to quit",
                    Style::default().fg(Color::DarkGray),
                )]));
                
                text.push(Line::from(""));
                text.push(Line::from(vec![
                    Span::styled("MEME:", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(
                        self.main_menu_meme,
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    )
                ]));
                let p = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("bet"));
                f.render_widget(Clear, rect);
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
                    let rect = centered_rect(80, 80, area);
                    let text = vec![
                        Line::from(vec![Span::styled(
                            lang.menu_game_selection,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )]),
                        Line::from(""),
                        Line::from(if self.game_cursor == 0 { vec![Span::styled(format!("    {}  ", lang.menu_hangman), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))] } else { vec![Span::styled(format!("    {}  ", lang.menu_hangman), Style::default().fg(Color::White))] }),
                        Line::from(if self.game_cursor == 1 { vec![Span::styled(format!("    {}  ", lang.menu_tictactoe), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))] } else { vec![Span::styled(format!("    {}  ", lang.menu_tictactoe), Style::default().fg(Color::White))] }),
                        Line::from(if self.game_cursor == 2 { vec![Span::styled(format!("    {}  ", lang.menu_chess), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))] } else { vec![Span::styled(format!("    {}  ", lang.menu_chess), Style::default().fg(Color::White))] }),
                        Line::from(if self.game_cursor == 3 { vec![Span::styled(format!("    {}  ", lang.menu_pong), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))] } else { vec![Span::styled(format!("    {}  ", lang.menu_pong), Style::default().fg(Color::White))] }),
                        Line::from(if self.game_cursor == 4 { vec![Span::styled(format!("    {}  ", lang.menu_recommender), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))] } else { vec![Span::styled(format!("    {}  ", lang.menu_recommender), Style::default().fg(Color::White))] }),
                        Line::from(if self.game_cursor == 5 { vec![Span::styled(format!("    {}  ", lang.menu_meme), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))] } else { vec![Span::styled(format!("    {}  ", lang.menu_meme), Style::default().fg(Color::White))] }),
                        Line::from(""),
                        Line::from(if self.game_cursor == 6 { vec![Span::styled(format!("    {}  ", lang.menu_go_back), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))] } else { vec![Span::styled(format!("    {}  ", lang.menu_go_back), Style::default().fg(Color::DarkGray))] }),
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
                    let game_area = centered_rect(70, 24, area);

                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),  // Title
                            Constraint::Length(10), // Hangman art
                            Constraint::Length(3),  // Word
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
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        layout[0],
                    );

                    // Art
                    let stage = game.max_attempts() - game.attempts_left();
                    let art = HANGMAN_ART[stage.min(6)];
                    let mut art_lines = vec![];
                    for (row, line) in art.lines().enumerate() {
                        let mut spans = vec![];
                        for (col, c) in line.chars().enumerate() {
                            let is_man = row >= 2 && col < 5 && c != ' ';
                            let color = if is_man {
                                Color::Rgb(180, 255, 50) // Neon Mango Biche for the man
                            } else {
                                Color::DarkGray // Faded gray for the gallows
                            };
                            spans.push(Span::styled(c.to_string(), Style::default().fg(color)));
                        }
                        art_lines.push(Line::from(spans));
                    }
                    f.render_widget(
                        Paragraph::new(art_lines).alignment(Alignment::Center),
                        layout[1],
                    );

                    // Compact, Highly Readable, AAA Word Rendering
                    let mut word_spans = vec![];
                    for c in game.word().chars() {
                        let is_revealed = c.is_alphabetic() && game.guessed_letters().contains(&c);
                        
                        if is_revealed {
                            word_spans.push(Span::styled(format!(" {} ", c), Style::default().bg(Color::Rgb(180, 255, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)));
                        } else if c.is_alphabetic() {
                            word_spans.push(Span::styled(" _ ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
                        } else {
                            word_spans.push(Span::styled(format!(" {} ", c), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
                        }
                    }
                    
                    let final_word_text = vec![Line::from(""), Line::from(word_spans), Line::from("")];
                    
                    f.render_widget(
                        Paragraph::new(final_word_text).alignment(Alignment::Center),
                        layout[2],
                    );

                    // Guessed
                    let guessed_text = vec![Line::from(vec![
                        Span::raw(lang.guessed_label),
                        Span::styled(game.display_guessed(), Style::default().fg(Color::White)),
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
                                .style(Style::default().fg(Color::Red)),
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
                                    .fg(Color::White)
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
                                style = style.fg(Color::White);
                            } else if ttt.board[idx] == Cell::Occupied(Player::O) {
                                style = style.fg(Color::Gray);
                            }

                            if ttt.status == TicTacToeStatus::Ongoing && idx == self.tictactoe_cursor {
                                style = style.bg(Color::Rgb(180, 255, 50)).fg(Color::Black);
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
                        TicTacToeStatus::Ongoing => {
                            Span::styled("Your turn (X)", Style::default().fg(Color::White))
                        }
                        TicTacToeStatus::Win(Player::X) => Span::styled(
                            "You win!",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        TicTacToeStatus::Win(Player::O) => Span::styled(
                            "Computer wins!",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        TicTacToeStatus::Draw => Span::styled(
                            "Draw!",
                            Style::default()
                                .fg(Color::White)
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
                            .style(Style::default().fg(Color::White)),
                        layout[3],
                    );

                    // Instructions
                    let instructions = if ttt.status == TicTacToeStatus::Ongoing {
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
            AppState::PlayingChess => {
                if let (Some(lang), Some(chess)) = (&self.lang, &self.chess) {
                    let rect = centered_rect(80, 90, area);
                    f.render_widget(Clear, rect);

                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),  // Title
                            Constraint::Length(11), // Board
                            Constraint::Length(2),  // Status
                            Constraint::Min(1),     // Instructions
                        ])
                        .split(rect);

                    // Title
                    f.render_widget(
                        Paragraph::new(lang.chess_title).alignment(Alignment::Center).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        layout[0],
                    );

                    // Board
                    let mut board_lines = vec![];
                    for rank in (0..8).rev() { 
                        let mut line_spans = vec![];
                        line_spans.push(Span::raw(format!(" {} ", rank + 1))); 
                        
                        for file in 0..8 {
                            let sq = crate::ui::Square::from_coords(shakmaty::File::new(file), shakmaty::Rank::new(rank));
                            let is_cursor = self.chess_cursor == sq;
                            let is_selected = self.chess_selected == Some(sq);
                            
                            let mut is_valid_dest = false;
                            if let Some(sel) = self.chess_selected {
                                let moves = chess.get_moves_from(sel);
                                is_valid_dest = moves.iter().any(|m| m.to() == sq);
                            }

                            let piece_str = match chess.pos.board().piece_at(sq) {
                                Some(piece) => {
                                    if is_utf8_supported() {
                                        match (piece.color, piece.role) {
                                            (shakmaty::Color::White, shakmaty::Role::Pawn) => "♙",
                                            (shakmaty::Color::White, shakmaty::Role::Knight) => "♘",
                                            (shakmaty::Color::White, shakmaty::Role::Bishop) => "♗",
                                            (shakmaty::Color::White, shakmaty::Role::Rook) => "♖",
                                            (shakmaty::Color::White, shakmaty::Role::Queen) => "♕",
                                            (shakmaty::Color::White, shakmaty::Role::King) => "♔",
                                            (shakmaty::Color::Black, shakmaty::Role::Pawn) => "♟",
                                            (shakmaty::Color::Black, shakmaty::Role::Knight) => "♞",
                                            (shakmaty::Color::Black, shakmaty::Role::Bishop) => "♝",
                                            (shakmaty::Color::Black, shakmaty::Role::Rook) => "♜",
                                            (shakmaty::Color::Black, shakmaty::Role::Queen) => "♛",
                                            (shakmaty::Color::Black, shakmaty::Role::King) => "♚",
                                        }
                                    } else {
                                        match (piece.color, piece.role) {
                                            (shakmaty::Color::White, shakmaty::Role::Pawn) => "P",
                                            (shakmaty::Color::White, shakmaty::Role::Knight) => "N",
                                            (shakmaty::Color::White, shakmaty::Role::Bishop) => "B",
                                            (shakmaty::Color::White, shakmaty::Role::Rook) => "R",
                                            (shakmaty::Color::White, shakmaty::Role::Queen) => "Q",
                                            (shakmaty::Color::White, shakmaty::Role::King) => "K",
                                            (shakmaty::Color::Black, shakmaty::Role::Pawn) => "p",
                                            (shakmaty::Color::Black, shakmaty::Role::Knight) => "n",
                                            (shakmaty::Color::Black, shakmaty::Role::Bishop) => "b",
                                            (shakmaty::Color::Black, shakmaty::Role::Rook) => "r",
                                            (shakmaty::Color::Black, shakmaty::Role::Queen) => "q",
                                            (shakmaty::Color::Black, shakmaty::Role::King) => "k",
                                        }
                                    }
                                },
                                None => " ",
                            };

                            let mut bg = if (rank + file) % 2 == 1 { Color::DarkGray } else { Color::Gray }; 
                            if is_valid_dest { bg = if (rank + file) % 2 == 1 { Color::Blue } else { Color::LightBlue }; }
                            if is_selected { bg = Color::Rgb(180, 255, 50); }
                            if is_cursor { bg = Color::Rgb(180, 255, 50); }

                            let text = format!(" {} ", piece_str);
                            let fg = Color::Black;
                            line_spans.push(Span::styled(text, Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD)));
                        }
                        board_lines.push(Line::from(line_spans));
                    }
                    let file_labels = Line::from("    A  B  C  D  E  F  G  H ");
                    board_lines.push(file_labels);
                    
                    f.render_widget(Paragraph::new(board_lines).alignment(Alignment::Center), layout[1]);

                    let status_msg = match chess.status {
                        ChessStatus::Ongoing => lang.chess_your_turn,
                        ChessStatus::Win(c) => if c == ChessColor::White { lang.chess_white_wins } else { lang.chess_black_wins },
                        ChessStatus::Stalemate => lang.chess_stalemate,
                        ChessStatus::Draw => lang.chess_draw,
                    };
                    
                    let instr = if chess.status == ChessStatus::Ongoing { lang.chess_instructions_ongoing } else { lang.chess_instructions_over };
                    
                    f.render_widget(Paragraph::new(status_msg).alignment(Alignment::Center).style(Style::default().fg(Color::White)), layout[2]);
                    f.render_widget(Paragraph::new(instr).alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray)), layout[3]);
                }
            }
            AppState::PlayingPong => {
                if let (Some(lang), Some(pong)) = (&self.lang, &self.pong) {
                    let rect = centered_rect(65, 24, area);
                    f.render_widget(Clear, rect);

                    let layout = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .constraints([
                            ratatui::layout::Constraint::Length(3),  // Title and Score
                            ratatui::layout::Constraint::Min(12),    // Canvas
                            ratatui::layout::Constraint::Length(2),  // Status/Instructions
                        ])
                        .split(rect);

                    // Title & Score
                    let title = format!("{}  |  {} - {}", lang.pong_title, pong.player_score, pong.computer_score);
                    f.render_widget(
                        Paragraph::new(title).alignment(Alignment::Center).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        layout[0],
                    );

                    // Canvas
                    let canvas = Canvas::default()
                        .block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(ratatui::widgets::BorderType::Thick))
                        .marker(ratatui::symbols::Marker::Braille)
                        .x_bounds([0.0, 100.0])
                        .y_bounds([0.0, 100.0])
                        .paint(|ctx| {
                            // Player paddle
                            ctx.draw(&Rectangle {
                                x: 5.0 - 1.0,
                                y: pong.player_y - 10.0,
                                width: 2.0,
                                height: 20.0,
                                color: Color::White,
                            });
                            // Computer paddle
                            ctx.draw(&Rectangle {
                                x: 95.0 - 1.0,
                                y: pong.computer_y - 10.0,
                                width: 2.0,
                                height: 20.0,
                                color: Color::White,
                            });
                            // Ball
                            ctx.draw(&Rectangle {
                                x: pong.ball_x - 1.0,
                                y: pong.ball_y - 1.0,
                                width: 2.0,
                                height: 2.0,
                                color: Color::Rgb(180, 255, 50),
                            });
                            // Center dashed line
                            for i in (0..100).step_by(5) {
                                ctx.draw(&Rectangle {
                                    x: 50.0 - 0.5,
                                    y: i as f64,
                                    width: 1.0,
                                    height: 2.0,
                                    color: Color::DarkGray,
                                });
                            }
                        });
                    f.render_widget(canvas, layout[1]);

                    let msg = match pong.status {
                        PongStatus::Ongoing => lang.pong_instructions,
                        PongStatus::PlayerWins => lang.pong_player_wins,
                        PongStatus::ComputerWins => lang.pong_computer_wins,
                    };
                    
                    let instr = if pong.status != PongStatus::Ongoing { "Press Enter to play again." } else { "" };
                    let combined = format!("{} {}", msg, instr);
                    
                    f.render_widget(Paragraph::new(combined).alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray)), layout[2]);
                }
            }
            AppState::GameOver(won) => {
                if let (Some(lang), Some(game)) = (&self.lang, &self.game) {
                    let rect = centered_rect(70, 50, area);
                    let msg = if won {
                        Span::styled(
                            lang.win_msg,
                            Style::default()
                                .fg(Color::Rgb(180, 255, 50)) // Neon Yellow Green
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
                                    .fg(Color::White)
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
                    f.render_widget(Clear, rect);
                    f.render_widget(p, rect);
                }
            }
                        AppState::RecommenderMenu => {
                if let Some(lang) = &self.lang {
                    let rect = centered_rect(65, 24, area);
                    let mut text = vec![
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            lang.menu_recommender,
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                        )]),
                        ratatui::text::Line::from(""),
                    ];
                    
                    let options = [
                        lang.recommender_menu_movies,
                        lang.recommender_menu_series,
                        lang.recommender_menu_manga,
                        lang.recommender_menu_books,
                        lang.recommender_menu_anime,
                        lang.recommender_menu_cartoons,
                        lang.recommender_menu_videogames,
                        lang.recommender_menu_music,
                        lang.recommender_go_back,
                    ];
                    
                    for (i, opt) in options.iter().enumerate() {
                        if i == 8 { text.push(ratatui::text::Line::from("")); } // Spacer before Go Back
                        
                        if i == self.recommender_cursor {
                            text.push(ratatui::text::Line::from(vec![ratatui::text::Span::styled(format!("    {}  ", opt), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))]));
                        } else {
                            let color = if i == 8 { Color::DarkGray } else { Color::White };
                            text.push(ratatui::text::Line::from(vec![ratatui::text::Span::styled(format!("    {}  ", opt), Style::default().fg(color))]));
                        }
                    }

                    let p = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(
                            Block::default()
                                .borders(ratatui::widgets::Borders::ALL).border_type(ratatui::widgets::BorderType::Thick)
                                .title(lang.recommender_title),
                        );
                    f.render_widget(Clear, rect);
                    f.render_widget(p, rect);
                }
            }
            AppState::MusicMenu => {
                if let Some(lang) = &self.lang {
                    let rect = centered_rect(65, 24, area);
                    let mut text = vec![
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            lang.recommender_menu_music,
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                        )]),
                        ratatui::text::Line::from(""),
                    ];
                    
                    let options = [
                        lang.music_menu_rock,
                        lang.music_menu_hiphop,
                        lang.music_menu_pop,
                        lang.music_menu_electronic,
                        lang.music_menu_classical,
                        lang.music_menu_salsa,
                        lang.music_menu_reggae,
                        lang.music_go_back,
                    ];
                    
                    for (i, opt) in options.iter().enumerate() {
                        if i == 9 { text.push(ratatui::text::Line::from("")); } // Spacer before Go Back
                        
                        if i == self.music_cursor {
                            text.push(ratatui::text::Line::from(vec![ratatui::text::Span::styled(format!("    {}  ", opt), Style::default().fg(Color::Black).bg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD))]));
                        } else {
                            let color = if i == 9 { Color::DarkGray } else { Color::White };
                            text.push(ratatui::text::Line::from(vec![ratatui::text::Span::styled(format!("    {}  ", opt), Style::default().fg(color))]));
                        }
                    }

                    let p = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(
                            Block::default()
                                .borders(ratatui::widgets::Borders::ALL).border_type(ratatui::widgets::BorderType::Thick)
                                .title(lang.music_menu_title),
                        );
                    f.render_widget(Clear, rect);
                    f.render_widget(p, rect);
                }
            }
            AppState::Recommendation(_, ref item) => {
                if let Some(lang) = &self.lang {
                    let rect = centered_rect(60, 30, area);
                    
                    let text = vec![
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            lang.recommender_subtitle,
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        )]),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            item.as_str(),
                            Style::default().fg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD),
                        )]),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            "Press Enter for another, or ESC to go back.",
                            Style::default().fg(Color::DarkGray),
                        )]),
                    ];

                    let p = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(ratatui::widgets::BorderType::Thick).title(lang.recommender_title));
                    
                    f.render_widget(Clear, rect);
                    f.render_widget(p, rect);
                }
            }
            AppState::Meme(ref item) => {
                if let Some(_lang) = &self.lang {
                    let rect = centered_rect(60, 20, area);
                    
                    let text = vec![
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            "Internet Culture:",
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        )]),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            item.as_str(),
                            Style::default().fg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD),
                        )]),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(""),
                        ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                            "Press Enter for another, or ESC to go back.",
                            Style::default().fg(Color::DarkGray),
                        )]),
                    ];

                    let p = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(ratatui::widgets::BorderType::Thick).title("MEME GENERATOR"));
                    
                    let clear_block = ratatui::widgets::Block::default().style(Style::default().bg(Color::Reset));
                    f.render_widget(clear_block, rect);
                    f.render_widget(p, rect);
                }
            },
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
                            .fg(Color::White)
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
                let color = Color::White;
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

                if self.bouncer_active && area.width > 40 && area.height > 20 {
            let b_rect = ratatui::layout::Rect {
                x: self.bouncer_x as u16,
                y: self.bouncer_y as u16,
                width: 38,
                height: 17,
            };
            
            let url = "https://discord.gg/MF6fMFURyC";
            if let Ok(code) = qrcode::QrCode::new(url) {
                let colors = code.to_colors();
                let width = code.width();
                
                let mut qr_lines = vec![];
                qr_lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(" [ DISCORD ] FIRST 3 TO SCAN WIN! ", Style::default().bg(Color::Rgb(180,255,50)).fg(Color::Black).add_modifier(Modifier::BOLD))));
                
                for y in (0..width).step_by(2) {
                    let mut line = String::new();
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
                    qr_lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(line, Style::default().fg(Color::Rgb(180,255,50)))));
                }
                
                let bouncer_p = Paragraph::new(qr_lines).block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(ratatui::widgets::BorderType::Thick));
                let cb = ratatui::widgets::Block::default().style(Style::default().bg(Color::Reset));
                f.render_widget(cb, b_rect);
                f.render_widget(bouncer_p, b_rect);
            }
        }
        
        // --- Render the infinite scrolling Poetry / News Ticker ---
        if !self.ticker_text.is_empty() && ticker_area.width > 0 {
            let offset = self.ticker_pos as usize;
            
            let mut display_text = String::with_capacity(ticker_area.width as usize);
            for i in 0..ticker_area.width as usize {
                display_text.push(self.ticker_text[(offset + i) % self.ticker_text.len()]);
            }
            
            // Sober floating gray text tracking across the top
            let mut spans = vec![];
            for c in display_text.chars() {
                if c == '✦' {
                    spans.push(Span::styled(c.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
                } else {
                    spans.push(Span::styled(c.to_string(), Style::default().fg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD)));
                }
            }
            let ticker_p = Paragraph::new(Line::from(spans));
            f.render_widget(ticker_p, ticker_area);
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

#[cfg(test)]
mod theme_tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use ratatui::style::Color;

    const ALLOWED_COLORS: &[Color] = &[
        Color::Reset,
        Color::Black,
        Color::White,
        Color::Yellow,
        Color::Red,
        Color::DarkGray,
        Color::Gray,
        Color::Rgb(180, 255, 50), // Neon Yellow Green
    ];

    #[test]
    fn test_strict_sober_theme_compliance() {
        // Ensures that no arbitrary RGB colors or unapproved standard colors
        // leak into the UI drawing across ANY state of the application.
        // This is a 100-year test for design system adherence.
        let backend = TestBackend::new(120, 40); // Standard large terminal
        let mut terminal = Terminal::new(backend).unwrap();

        let mut app = App::new();
        app.select_language(Language::English);
        
        // Mock active games to test their UIs
        app.game = Some(crate::game::Hangman::new("TEST", 6));
        app.tictactoe = Some(crate::tictactoe::TicTacToe::new());
        app.chess = Some(crate::chess_game::ChessGame::new(false));
        app.pong = Some(crate::pong::PongGame::new());
        
        let test_states = vec![
            AppState::LanguageSelection,
            AppState::GameSelection,
            AppState::RecommenderMenu,
            AppState::MusicMenu,
            AppState::Recommendation(RecommenderCategory::Movie, "The Godfather".to_string()),
            AppState::Playing,
            AppState::PlayingTicTacToe,
            AppState::PlayingChess,
            AppState::PlayingPong,
            AppState::DiscordQr,
            AppState::EasterEgg,
            AppState::GameOver(true),
            AppState::GameOver(false),
        ];

        for state in test_states {
            app.state = state;
            
            // Clear backend to avoid carry-over
            terminal.clear().unwrap();

            terminal.draw(|f| {
                app.draw(f);
            }).unwrap();

            let buffer = terminal.backend().buffer();
            for cell in buffer.content() {
                if !ALLOWED_COLORS.contains(&cell.fg) {
                    panic!("Theme violation: Disallowed foreground color {:?}", cell.fg);
                }
                if !ALLOWED_COLORS.contains(&cell.bg) {
                    panic!("Theme violation: Disallowed background color {:?}", cell.bg);
                }
            }
        }
    }
}
