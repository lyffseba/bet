pub enum Language {
    English,
    Spanish,
    Portuguese,
    German,
    Dutch,
}

#[derive(Clone)]
pub struct Lang {
    pub title: &'static str,
    pub prompt_guess: &'static str,
    pub win_msg: &'static str,
    pub lose_msg: &'static str,
    pub press_enter: &'static str,
    pub word_label: &'static str,
    pub guessed_label: &'static str,
    pub attempts_label: &'static str,
    pub time_left_label: &'static str,
    pub error_not_letter: &'static str,
    pub error_already_guessed: &'static str,
    pub movies: &'static [&'static str],
}

impl Lang {
    pub fn english() -> Self {
        Lang {
            title: "HANGMAN",
            prompt_guess: "Enter a letter (or 'ESC' to exit): ",
            win_msg: "Congratulations! You won!",
            lose_msg: "Game over! The word was: ",
            press_enter: "Press Enter to continue...",
            word_label: "Word: ",
            guessed_label: "Guessed letters: ",
            attempts_label: "Attempts left: ",
            time_left_label: "Time left: ",
            error_not_letter: "Please enter a letter",
            error_already_guessed: "You already guessed that letter",
            movies: super::wordlist::ENGLISH_MOVIES,
        }
    }

    pub fn spanish() -> Self {
        Lang {
            title: "AHORCADO",
            prompt_guess: "Ingresa una letra (o 'ESC' para salir): ",
            win_msg: "¡Felicidades! ¡Ganaste!",
            lose_msg: "¡Juego terminado! La palabra era: ",
            press_enter: "Presiona Enter para continuar...",
            word_label: "Palabra: ",
            guessed_label: "Letras adivinadas: ",
            attempts_label: "Intentos restantes: ",
            time_left_label: "Tiempo restante: ",
            error_not_letter: "Por favor ingresa una letra",
            error_already_guessed: "Ya adivinaste esa letra",
            movies: super::wordlist::SPANISH_MOVIES,
        }
    }

    pub fn portuguese() -> Self {
        Lang {
            title: "FORCA",
            prompt_guess: "Digite uma letra (ou 'ESC' para sair): ",
            win_msg: "Parabéns! Você ganhou!",
            lose_msg: "Fim de jogo! A palavra era: ",
            press_enter: "Pressione Enter para continuar...",
            word_label: "Palavra: ",
            guessed_label: "Letras adivinhadas: ",
            attempts_label: "Tentativas restantes: ",
            time_left_label: "Tempo restante: ",
            error_not_letter: "Por favor, digite uma letra",
            error_already_guessed: "Você já adivinhou essa letra",
            movies: super::wordlist::PORTUGUESE_MOVIES,
        }
    }

    pub fn german() -> Self {
        Lang {
            title: "GALGENMÄNNCHEN",
            prompt_guess: "Buchstabe eingeben (oder 'ESC' zum Beenden): ",
            win_msg: "Herzlichen Glückwunsch! Du hast gewonnen!",
            lose_msg: "Spiel vorbei! Das Wort war: ",
            press_enter: "Drücke Enter, um fortzufahren...",
            word_label: "Wort: ",
            guessed_label: "Geratene Buchstaben: ",
            attempts_label: "Verbleibende Versuche: ",
            time_left_label: "Verbleibende Zeit: ",
            error_not_letter: "Bitte gib einen Buchstaben ein",
            error_already_guessed: "Du hast diesen Buchstaben bereits geraten",
            movies: super::wordlist::GERMAN_MOVIES,
        }
    }

    pub fn dutch() -> Self {
        Lang {
            title: "GALGJE",
            prompt_guess: "Voer een letter in (of 'ESC' om te sluiten): ",
            win_msg: "Gefeliciteerd! Je hebt gewonnen!",
            lose_msg: "Spel voorbij! Het woord was: ",
            press_enter: "Druk op Enter om door te gaan...",
            word_label: "Woord: ",
            guessed_label: "Geraden letters: ",
            attempts_label: "Resterende pogingen: ",
            time_left_label: "Resterende tijd: ",
            error_not_letter: "Voer een letter in",
            error_already_guessed: "Je hebt deze letter al geraden",
            movies: super::wordlist::DUTCH_MOVIES,
        }
    }

    pub fn from_language(lang: Language) -> Self {
        match lang {
            Language::English => Self::english(),
            Language::Spanish => Self::spanish(),
            Language::Portuguese => Self::portuguese(),
            Language::German => Self::german(),
            Language::Dutch => Self::dutch(),
        }
    }
}
