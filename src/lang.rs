pub enum Language {
    English,
    Spanish,
    Portuguese,
}

pub struct Lang {
    pub title: &'static str,
    pub menu_solo: &'static str,
    pub menu_multi: &'static str,
    pub menu_quit: &'static str,
    pub prompt_option: &'static str,
    pub prompt_guess: &'static str,
    pub win_msg: &'static str,
    pub lose_msg: &'static str,
    pub press_enter: &'static str,
    pub word_label: &'static str,
    pub guessed_label: &'static str,
    pub attempts_label: &'static str,
    pub word_input_prompt: &'static str,
    pub word_input_instruction: &'static str,
    pub word_input_warning: &'static str,
    pub error_not_letter: &'static str,
    pub error_already_guessed: &'static str,
    pub movies: &'static [&'static str],
}

impl Lang {
    pub fn english() -> Self {
        Lang {
            title: "HANGMAN",
            menu_solo: "1. Solo (random movie)",
            menu_multi: "2. Multiplayer (one player sets word)",
            menu_quit: "3. Quit",
            prompt_option: "Choose option: ",
            prompt_guess: "Enter a letter (or 'quit' to exit): ",
            win_msg: "Congratulations! You won!",
            lose_msg: "Game over! The word was: ",
            press_enter: "Press Enter to continue...",
            word_label: "Word: ",
            guessed_label: "Guessed letters: ",
            attempts_label: "Attempts left: ",
            word_input_prompt: "Enter the word for the other player to guess (will not be shown):",
            word_input_instruction: "(Type the word and press Enter)",
            word_input_warning: "Second player, look away now!",
            error_not_letter: "Please enter a letter",
            error_already_guessed: "You already guessed that letter",
            movies: super::wordlist::ENGLISH_MOVIES,
        }
    }

    pub fn spanish() -> Self {
        Lang {
            title: "AHORCADO",
            menu_solo: "1. Solo (película aleatoria)",
            menu_multi: "2. Multijugador (un jugador pone la palabra)",
            menu_quit: "3. Salir",
            prompt_option: "Elige una opción: ",
            prompt_guess: "Ingresa una letra (o 'salir' para salir): ",
            win_msg: "¡Felicidades! ¡Ganaste!",
            lose_msg: "¡Juego terminado! La palabra era: ",
            press_enter: "Presiona Enter para continuar...",
            word_label: "Palabra: ",
            guessed_label: "Letras adivinadas: ",
            attempts_label: "Intentos restantes: ",
            word_input_prompt: "Ingresa la palabra para que el otro jugador adivine (no se mostrará):",
            word_input_instruction: "(Escribe la palabra y presiona Enter)",
            word_input_warning: "¡Segundo jugador, mira hacia otro lado ahora!",
            error_not_letter: "Por favor ingresa una letra",
            error_already_guessed: "Ya adivinaste esa letra",
            movies: super::wordlist::SPANISH_MOVIES,
        }
    }

    pub fn portuguese() -> Self {
        Lang {
            title: "FORCA",
            menu_solo: "1. Solo (filme aleatório)",
            menu_multi: "2. Multijogador (um jogador coloca a palavra)",
            menu_quit: "3. Sair",
            prompt_option: "Escolha uma opção: ",
            prompt_guess: "Digite uma letra (ou 'sair' para sair): ",
            win_msg: "Parabéns! Você ganhou!",
            lose_msg: "Fim de jogo! A palavra era: ",
            press_enter: "Pressione Enter para continuar...",
            word_label: "Palavra: ",
            guessed_label: "Letras adivinhadas: ",
            attempts_label: "Tentativas restantes: ",
            word_input_prompt: "Digite a palavra para o outro jogador adivinhar (não será mostrada):",
            word_input_instruction: "(Digite a palavra e pressione Enter)",
            word_input_warning: "Segundo jogador, olhe para outro lado agora!",
            error_not_letter: "Por favor, digite uma letra",
            error_already_guessed: "Você já adivinhou essa letra",
            movies: super::wordlist::PORTUGUESE_MOVIES,
        }
    }

    pub fn from_language(lang: Language) -> Self {
        match lang {
            Language::English => Self::english(),
            Language::Spanish => Self::spanish(),
            Language::Portuguese => Self::portuguese(),
        }
    }
}
