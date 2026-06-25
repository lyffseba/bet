import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { matchesKey, visibleWidth } from "@earendil-works/pi-tui";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

type Game = "menu" | "tictactoe" | "hangman" | "recommender" | "matrix";

const HIGHSCORE_FILE = path.join(os.homedir(), ".pi", "bet_highscores.json");

function loadHighScore(): number {
	try {
		if (fs.existsSync(HIGHSCORE_FILE)) {
			const data = fs.readFileSync(HIGHSCORE_FILE, "utf8");
			const json = JSON.parse(data);
			return typeof json.matrixHighScore === "number" ? json.matrixHighScore : 0;
		}
	} catch (e) {
		// Ignore
	}
	return 0;
}

function saveHighScore(score: number) {
	try {
		const dir = path.dirname(HIGHSCORE_FILE);
		if (!fs.existsSync(dir)) {
			fs.mkdirSync(dir, { recursive: true });
		}
		fs.writeFileSync(HIGHSCORE_FILE, JSON.stringify({ matrixHighScore: score }), "utf8");
	} catch (e) {
		// Ignore
	}
}

const HANGMAN_WORDS = [
	"THE MATRIX", "INCEPTION", "PULP FICTION", "THE DARK KNIGHT", 
	"FIGHT CLUB", "INTERSTELLAR", "GLADIATOR", "BLADE RUNNER", 
	"SPIDERMAN", "AVATAR", "JURASSIC PARK", "THE TERMINATOR", 
	"ALIEN", "TITANIC", "PSYCHO", "GOODFELLAS", "SEVEN", 
	"TOY STORY", "THE SHINING", "SCARFACE", "CHINATOWN", 
	"AMADEUS", "FARGO", "MEMENTO", "JOKER", "BRAVEHEART", "COCO"
];

const RECOMMENDATIONS: Record<string, string[]> = {
	"Movies": [
		"The Godfather - A masterpiece of mafia cinema",
		"Inception - Mind-bending dream heist by Nolan",
		"The Dark Knight - The ultimate gritty superhero film",
		"Spirited Away - Gorgeous Ghibli fantasy animation",
		"Interstellar - Epic emotional journey through spacetime"
	],
	"Books": [
		"Dune by Frank Herbert - The foundational sci-fi epic",
		"Neuromancer by William Gibson - The original cyberpunk novel",
		"1984 by George Orwell - The legendary dystopian warning",
		"The Hobbit by J.R.R. Tolkien - The classic fantasy adventure",
		"The Hitchhiker's Guide to the Galaxy - Hilarious cosmic comedy"
	],
	"Anime": [
		"Fullmetal Alchemist: Brotherhood - A perfect fantasy journey",
		"Neon Genesis Evangelion - Deep psychological sci-fi mecha",
		"Attack on Titan - Intense, mystery-filled dark fantasy",
		"Death Note - High-stakes intellectual battle of wits",
		"Cowboy Bebop - Timeless space-jazz bounty hunter adventure"
	],
	"Games": [
		"Portal 2 - Hilarious, mind-bending puzzle masterpiece",
		"The Witcher 3: Wild Hunt - Gorgeous, narrative-rich action RPG",
		"Elden Ring - Immersive and challenging open-world fantasy",
		"Hades - Fast-paced, story-driven Greek roguelike",
		"Minecraft - Infinite creative sandbox exploration"
	]
};

const MATRIX_DICTIONARY = [
	"KUBERNETES", "TYPESCRIPT", "CYBERPUNK", "DOCKER", "COMPILER",
	"PARADIGM", "DATABASE", "FIREWALL", "TERMINAL", "MAINBOARD",
	"INTERFACE", "RECURSION", "ALGORITHM", "METADATA", "CONTAINER",
	"MAINFRAME", "BITCOIN", "REFACTOR", "DECRYPT", "CYBERSPACE",
	"ETHERNET", "KEYBOARD", "PROCESSOR", "EMULATOR", "FRONTEND"
];

const HANGMAN_ART: string[][] = [
	[
		"   +-----------+   ",
		"   |           |   ",
		"   |           |   ",
		"               |   ",
		"               |   ",
		"               |   ",
		"               |   ",
		"               |   ",
		"==================="
	],
	[
		"   +-----------+   ",
		"   |           |   ",
		"   |           |   ",
		"  ( )          |   ",
		"               |   ",
		"               |   ",
		"               |   ",
		"               |   ",
		"==================="
	],
	[
		"   +-----------+   ",
		"   |           |   ",
		"   |           |   ",
		"  ( )          |   ",
		"   |           |   ",
		"   |           |   ",
		"               |   ",
		"               |   ",
		"==================="
	],
	[
		"   +-----------+   ",
		"   |           |   ",
		"   |           |   ",
		"  ( )          |   ",
		"  /|           |   ",
		" / |           |   ",
		"               |   ",
		"               |   ",
		"==================="
	],
	[
		"   +-----------+   ",
		"   |           |   ",
		"   |           |   ",
		"  ( )          |   ",
		"  /|\\          |   ",
		" / | \\         |   ",
		"               |   ",
		"               |   ",
		"==================="
	],
	[
		"   +-----------+   ",
		"   |           |   ",
		"   |           |   ",
		"  ( )          |   ",
		"  /|\\          |   ",
		" / | \\         |   ",
		"  /            |   ",
		" /             |   ",
		"==================="
	],
	[
		"   +-----------+   ",
		"   |           |   ",
		"   |           |   ",
		"  ( )          |   ",
		"  /|\\          |   ",
		" / | \\         |   ",
		"  / \\          |   ",
		" /   \\         |   ",
		"==================="
	]
];

interface MatrixWord {
	text: string;
	typed: string;
	x: number;
	y: number;
}

interface ConfettiParticle {
	x: number;
	char: string;
	color: string;
}

class BetNativeComponent {
	private tui: { requestRender: () => void };
	private onClose: () => void;
	
	private currentGame: Game = "menu";
	private cachedLines: string[] = [];
	private cachedWidth = 0;
	private version = 0;
	private cachedVersion = -1;

	// Tic-Tac-Toe State
	private board: (string | null)[][] = [
		[null, null, null],
		[null, null, null],
		[null, null, null],
	];
	private cursorX = 0;
	private cursorY = 0;
	private currentPlayer = "X";
	private winner: string | null = null;
	private draw = false;

	// Hangman State
	private hangmanWord = "";
	private hangmanGuessed: Set<string> = new Set();
	private hangmanAttemptsLeft = 6;
	private hangmanOver = false;
	private hangmanWon = false;

	// Recommender State
	private recCategory: string | null = null;
	private recSelection: string | null = null;

	// Matrix State
	private matrixWords: MatrixWord[] = [];
	private matrixScore = 0;
	private matrixHighScore = 0;
	private matrixLives = 3;
	private matrixOver = false;
	private matrixTargetWord: MatrixWord | null = null;
	private matrixSpawnCooldown = 0;
	private matrixInterval: ReturnType<typeof setInterval> | null = null;

	// Global Ticker State (Breathing Borders + Confetti)
	private globalInterval: ReturnType<typeof setInterval> | null = null;
	private confetti: ConfettiParticle[] = [];
	private confettiTimer = 0;

	constructor(tui: { requestRender: () => void }, onClose: () => void) {
		this.tui = tui;
		this.onClose = onClose;
		this.matrixHighScore = loadHighScore();

		// Start Global Animation Timer (every 100ms)
		this.globalInterval = setInterval(() => {
			this.globalTick();
		}, 100);
	}

	private globalTick() {
		// Update Confetti Particles
		if (this.confettiTimer > 0) {
			this.confettiTimer--;
			// Shift particles randomly to simulate falling
			for (const p of this.confetti) {
				p.x = (p.x + (Math.random() > 0.5 ? 1 : -1) + 42) % 42;
			}
			if (this.confettiTimer === 0) {
				this.confetti = [];
			}
		}

		this.version++;
		this.tui.requestRender();
	}

	private triggerConfetti() {
		this.confettiTimer = 35; // active for 3.5 seconds
		const chars = ["*", "+", "o", "◆", "★", "x"];
		const colors = [
			"\x1b[31m", // red
			"\x1b[32m", // green
			"\x1b[33m", // yellow
			"\x1b[34m", // blue
			"\x1b[35m", // magenta
			"\x1b[36m"  // cyan
		];
		this.confetti = [];
		for (let i = 0; i < 20; i++) {
			this.confetti.push({
				x: Math.floor(Math.random() * 40) + 1,
				char: chars[Math.floor(Math.random() * chars.length)],
				color: colors[Math.floor(Math.random() * colors.length)]
			});
		}
	}

	private getBreathingGreen(): string {
		const time = Date.now() / 400; // pulse speed
		const sine = (Math.sin(time) + 1.0) / 2.0; // scale 0 to 1
		const r = Math.floor(15 + 25 * sine);
		const g = Math.floor(160 + 85 * sine); // neon green oscillation
		const b = Math.floor(15 + 25 * sine);
		return `\x1b[38;2;${r};${g};${b}m`;
	}

	handleInput(data: string): void {
		if (matchesKey(data, "escape") || data === "q" || data === "Q") {
			if (this.currentGame !== "menu") {
				this.stopMatrix();
				this.currentGame = "menu";
				this.version++;
				this.tui.requestRender();
			} else {
				this.stopMatrix();
				this.onClose();
			}
			return;
		}

		if (this.currentGame === "menu") {
			if (data === "1") {
				this.currentGame = "tictactoe";
				this.resetTicTacToe();
			} else if (data === "2") {
				this.currentGame = "hangman";
				this.resetHangman();
			} else if (data === "3") {
				this.currentGame = "recommender";
				this.resetRecommender();
			} else if (data === "4") {
				this.currentGame = "matrix";
				this.resetMatrix();
			}
			this.version++;
			this.tui.requestRender();
			return;
		}

		if (this.currentGame === "tictactoe") {
			if (this.winner || this.draw) {
				if (data === "r" || data === "R" || data === " ") {
					this.resetTicTacToe();
					this.version++;
					this.tui.requestRender();
				}
				return;
			}

			if (matchesKey(data, "up") || data === "w" || data === "W") {
				this.cursorY = Math.max(0, this.cursorY - 1);
			} else if (matchesKey(data, "down") || data === "s" || data === "S") {
				this.cursorY = Math.min(2, this.cursorY + 1);
			} else if (matchesKey(data, "left") || data === "a" || data === "A") {
				this.cursorX = Math.max(0, this.cursorX - 1);
			} else if (matchesKey(data, "right") || data === "d" || data === "D") {
				this.cursorX = Math.min(2, this.cursorX + 1);
			} else if (data === "\r" || data === " ") {
				const row = this.board[this.cursorY];
				if (row && row[this.cursorX] === null) {
					row[this.cursorX] = "X";
					this.checkWinner();
					if (this.winner === "X") {
						this.triggerConfetti();
					} else if (!this.winner && !this.draw) {
						this.currentPlayer = "O";
						this.runTicTacToeAI();
					}
				}
			}
			this.version++;
			this.tui.requestRender();
			return;
		}

		if (this.currentGame === "hangman") {
			if (this.hangmanOver) {
				if (data === "r" || data === "R" || data === " ") {
					this.resetHangman();
					this.version++;
					this.tui.requestRender();
				}
				return;
			}

			if (/^[a-zA-Z]$/.test(data)) {
				const char = data.toUpperCase();
				if (!this.hangmanGuessed.has(char)) {
					this.hangmanGuessed.add(char);
					if (!this.hangmanWord.includes(char)) {
						this.hangmanAttemptsLeft = Math.max(0, this.hangmanAttemptsLeft - 1);
					}
					this.checkHangmanStatus();
				}
			}
			this.version++;
			this.tui.requestRender();
			return;
		}

		if (this.currentGame === "recommender") {
			if (data === "1") {
				this.getRecommendation("Movies");
			} else if (data === "2") {
				this.getRecommendation("Books");
			} else if (data === "3") {
				this.getRecommendation("Anime");
			} else if (data === "4") {
				this.getRecommendation("Games");
			}
			this.version++;
			this.tui.requestRender();
			return;
		}

		if (this.currentGame === "matrix") {
			if (this.matrixOver) {
				if (data === "r" || data === "R" || data === " ") {
					this.resetMatrix();
					this.version++;
					this.tui.requestRender();
				}
				return;
			}

			if (/^[a-zA-Z]$/.test(data)) {
				const letter = data.toUpperCase();
				if (!this.matrixTargetWord) {
					const found = this.matrixWords.find(w => w.text.startsWith(letter));
					if (found) {
						this.matrixTargetWord = found;
						found.typed = letter;
					}
				} else if (this.matrixTargetWord) {
					const targetIndex = this.matrixTargetWord.typed.length;
					if (this.matrixTargetWord.text[targetIndex] === letter) {
						this.matrixTargetWord.typed += letter;
						if (this.matrixTargetWord.typed === this.matrixTargetWord.text) {
							this.matrixScore += this.matrixTargetWord.text.length * 10;
							
							if (this.matrixScore > this.matrixHighScore) {
								this.matrixHighScore = this.matrixScore;
								saveHighScore(this.matrixHighScore);
							}

							this.triggerConfetti(); // Explode on matrix word cleared!
							const completed = this.matrixTargetWord;
							this.matrixWords = this.matrixWords.filter(w => w !== completed);
							this.matrixTargetWord = null;
						}
					}
				}
			}
			this.version++;
			this.tui.requestRender();
			return;
		}
	}

	private resetTicTacToe() {
		this.board = [
			[null, null, null],
			[null, null, null],
			[null, null, null],
		];
		this.cursorX = 0;
		this.cursorY = 0;
		this.currentPlayer = "X";
		this.winner = null;
		this.draw = false;
	}

	private runTicTacToeAI() {
		if (this.winner || this.draw) return;

		const empty: [number, number][] = [];
		for (let r = 0; r < 3; r++) {
			for (let c = 0; c < 3; c++) {
				if (this.board[r]?.[c] === null) empty.push([r, c]);
			}
		}

		if (empty.length === 0) return;

		// 1. Can AI ("O") win immediately?
		for (const [r, c] of empty) {
			const row = this.board[r];
			if (row) {
				row[c] = "O";
				if (this.checkWinCondition("O")) {
					this.winner = "O";
					return;
				}
				row[c] = null;
			}
		}

		// 2. Can Player ("X") win immediately? Block them!
		for (const [r, c] of empty) {
			const row = this.board[r];
			if (row) {
				row[c] = "X";
				if (this.checkWinCondition("X")) {
					row[c] = "O";
					this.currentPlayer = "X";
					this.checkWinner();
					return;
				}
				row[c] = null;
			}
		}

		// 3. Take center if available
		const centerRow = this.board[1];
		if (centerRow && centerRow[1] === null) {
			centerRow[1] = "O";
			this.currentPlayer = "X";
			this.checkWinner();
			return;
		}

		// 4. Take random corner/side
		const choice = empty[Math.floor(Math.random() * empty.length)];
		if (choice) {
			const [r, c] = choice;
			const row = this.board[r];
			if (row) {
				row[c] = "O";
				this.currentPlayer = "X";
				this.checkWinner();
			}
		}
	}

	private checkWinCondition(player: string): boolean {
		const lines: [[number, number], [number, number], [number, number]][] = [
			[[0,0], [0,1], [0,2]],
			[[1,0], [1,1], [1,2]],
			[[2,0], [2,1], [2,2]],
			[[0,0], [1,0], [2,0]],
			[[0,1], [1,1], [2,1]],
			[[0,2], [1,2], [2,2]],
			[[0,0], [1,1], [2,2]],
			[[0,2], [1,1], [2,0]]
		];
		for (const line of lines) {
			const [a, b, c] = line;
			if (
				this.board[a[0]]?.[a[1]] === player &&
				this.board[b[0]]?.[b[1]] === player &&
				this.board[c[0]]?.[c[1]] === player
			) {
				return true;
			}
		}
		return false;
	}

	private resetHangman() {
		const idx = Math.floor(Math.random() * HANGMAN_WORDS.length);
		this.hangmanWord = HANGMAN_WORDS[idx];
		this.hangmanGuessed = new Set();
		this.hangmanAttemptsLeft = 6;
		this.hangmanOver = false;
		this.hangmanWon = false;
	}

	private checkHangmanStatus() {
		if (this.hangmanAttemptsLeft === 0) {
			this.hangmanOver = true;
			this.hangmanWon = false;
			return;
		}

		let won = true;
		for (const char of this.hangmanWord) {
			if (/[A-Z]/.test(char) && !this.hangmanGuessed.has(char)) {
				won = false;
				break;
			}
		}

		if (won) {
			this.hangmanOver = true;
			this.hangmanWon = true;
			this.triggerConfetti(); // Confetti on Hangman win!
		}
	}

	private resetRecommender() {
		this.recCategory = null;
		this.recSelection = null;
	}

	private getRecommendation(cat: string) {
		const list = RECOMMENDATIONS[cat];
		const idx = Math.floor(Math.random() * list.length);
		this.recCategory = cat;
		this.recSelection = list[idx];
	}

	private resetMatrix() {
		this.matrixWords = [];
		this.matrixScore = 0;
		this.matrixLives = 3;
		this.matrixOver = false;
		this.matrixTargetWord = null;
		this.matrixSpawnCooldown = 0;

		this.spawnMatrixWord();

		this.stopMatrix();
		this.matrixInterval = setInterval(() => {
			this.matrixTick();
		}, 180);
	}

	private spawnMatrixWord() {
		const idx = Math.floor(Math.random() * MATRIX_DICTIONARY.length);
		const text = MATRIX_DICTIONARY[idx];
		if (this.matrixWords.some(w => w.text === text)) return;

		const x = Math.floor(Math.random() * 16) + 2;
		this.matrixWords.push({
			text,
			typed: "",
			x,
			y: 0
		});
	}

	private matrixTick() {
		if (this.matrixOver) return;

		for (const word of this.matrixWords) {
			word.y += 0.35; // fall rate

			if (word.y >= 11) {
				this.matrixLives = Math.max(0, this.matrixLives - 1);
				if (this.matrixTargetWord === word) {
					this.matrixTargetWord = null;
				}
				this.matrixWords = this.matrixWords.filter(w => w !== word);

				if (this.matrixLives <= 0) {
					this.matrixOver = true;
					this.stopMatrix();
				}
				break;
			}
		}

		this.matrixSpawnCooldown++;
		if (this.matrixSpawnCooldown >= 10) {
			this.spawnMatrixWord();
			this.matrixSpawnCooldown = 0;
		}

		this.version++;
		this.tui.requestRender();
	}

	private stopMatrix() {
		if (this.matrixInterval) {
			clearInterval(this.matrixInterval);
			this.matrixInterval = null;
		}
	}

	private checkWinner() {
		const lines: [[number, number], [number, number], [number, number]][] = [
			[[0,0], [0,1], [0,2]],
			[[1,0], [1,1], [1,2]],
			[[2,0], [2,1], [2,2]],
			[[0,0], [1,0], [2,0]],
			[[0,1], [1,1], [2,1]],
			[[0,2], [1,2], [2,2]],
			[[0,0], [1,1], [2,2]],
			[[0,2], [1,1], [2,0]]
		];

		for (const line of lines) {
			const [a, b, c] = line;
			const cellA = this.board[a[0]]?.[a[1]];
			const cellB = this.board[b[0]]?.[b[1]];
			const cellC = this.board[c[0]]?.[c[1]];
			if (cellA && cellA === cellB && cellA === cellC) {
				this.winner = cellA;
				return;
			}
		}

		let isDraw = true;
		for (let y = 0; y < 3; y++) {
			for (let x = 0; x < 3; x++) {
				if (this.board[y]?.[x] === null) isDraw = false;
			}
		}
		if (isDraw) this.draw = true;
	}

	invalidate(): void {
		this.cachedWidth = 0;
	}

	render(width: number): string[] {
		if (width === this.cachedWidth && this.cachedVersion === this.version) {
			return this.cachedLines;
		}

		const lines: string[] = [];
		const boxWidth = 46;

		const breath = this.getBreathingGreen();
		const dim = (s: string) => `\x1b[2m${s}\x1b[22m`;
		const green = (s: string) => `\x1b[32m${s}\x1b[0m`;
		const blue = (s: string) => `\x1b[34m${s}\x1b[0m`;
		const red = (s: string) => `\x1b[31m${s}\x1b[0m`;
		const yellow = (s: string) => `\x1b[33m${s}\x1b[0m`;
		const bold = (s: string) => `\x1b[1m${s}\x1b[22m`;
		const inverse = (s: string) => `\x1b[7m${s}\x1b[27m`;

		const boxLine = (content: string) => {
			const contentLen = visibleWidth(content);
			const padding = Math.max(0, boxWidth - contentLen);
			return breath + "│\x1b[0m" + content + " ".repeat(padding) + breath + "│\x1b[0m";
		};

		const padToCenter = (line: string) => {
			const visibleLen = visibleWidth(line);
			const leftPad = Math.max(0, Math.floor((width - visibleLen) / 2));
			return " ".repeat(leftPad) + line;
		};

		lines.push(padToCenter(breath + `╭${"─".repeat(boxWidth)}╮\x1b[0m`));
		lines.push(padToCenter(boxLine(` ${bold(green("★ B$T (BET) - Terminal Game Hub ★"))} `)));
		lines.push(padToCenter(breath + `├${"─".repeat(boxWidth)}┤\x1b[0m`));

		if (this.currentGame === "menu") {
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  ${bold("Select an arcade game to play:")}`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  [1] Tic-Tac-Toe  ${dim("(Play vs Smart AI)")}`)));
			lines.push(padToCenter(boxLine(`  [2] Hangman      ${dim("(Movie word survival)")}`)));
			lines.push(padToCenter(boxLine(`  [3] Recommender  ${dim("(Book/Anime/Movie recs)")}`)));
			lines.push(padToCenter(boxLine(`  [4] The Matrix   ${dim("(Hacker typing survival)")}`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  [Q] Quit Game Hub`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine("")));
		} else if (this.currentGame === "tictactoe") {
			lines.push(padToCenter(boxLine("")));
			let header = `  Player ${this.currentPlayer}'s turn`;
			if (this.winner) {
				header = this.winner === "X" 
					? `  ${bold(green("CONGRATS! YOU BEAT THE AI!"))}` 
					: `  ${bold(red("GAME OVER! AI WINS!"))}`;
			} else if (this.draw) {
				header = `  ${bold(yellow("IT'S A DRAW!"))}`;
			}
			lines.push(padToCenter(boxLine(header)));
			lines.push(padToCenter(boxLine("")));

			for (let y = 0; y < 3; y++) {
				let rowStr = "           ";
				for (let x = 0; x < 3; x++) {
					const cell = this.board[y]?.[x] || " ";
					const coloredCell = cell === "X" ? red(cell) : cell === "O" ? blue(cell) : cell;
					
					if (this.cursorX === x && this.cursorY === y && !this.winner && !this.draw) {
						rowStr += `[${inverse(coloredCell)}]`;
					} else {
						rowStr += `[ ${coloredCell} ]`;
					}
				}
				lines.push(padToCenter(boxLine(rowStr)));
				if (y < 2) lines.push(padToCenter(boxLine("")));
			}
			lines.push(padToCenter(boxLine("")));
			if (this.winner || this.draw) {
				lines.push(padToCenter(boxLine(`  Press [R] to restart, [Q] for menu`)));
			} else {
				lines.push(padToCenter(boxLine(`  Use arrows to move, Enter to place`)));
			}
		} else if (this.currentGame === "hangman") {
			lines.push(padToCenter(boxLine("")));
			
			let statusStr = `  Guess the movie! Attempts left: ${red(String(this.hangmanAttemptsLeft))}`;
			if (this.hangmanOver) {
				statusStr = this.hangmanWon 
					? `  ${bold(green("YOU GUESSED IT! CONGRATULATIONS!"))}`
					: `  ${bold(red("GAME OVER! The movie was: " + this.hangmanWord))}`;
			}
			lines.push(padToCenter(boxLine(statusStr)));
			lines.push(padToCenter(boxLine("")));

			let displayWord = "";
			for (const char of this.hangmanWord) {
				if (char === " ") {
					displayWord += "  ";
				} else if (/[A-Z]/.test(char)) {
					displayWord += this.hangmanGuessed.has(char) ? `${char} ` : "_ ";
				} else {
					displayWord += `${char} `;
				}
			}
			lines.push(padToCenter(boxLine(`  Word: ${bold(displayWord.trim())}`)));
			lines.push(padToCenter(boxLine("")));

			const artIdx = 6 - this.hangmanAttemptsLeft;
			const artLines = HANGMAN_ART[artIdx];
			for (const artLine of artLines) {
				lines.push(padToCenter(boxLine(`      ${dim(artLine)}`)));
			}
			lines.push(padToCenter(boxLine("")));

			const guessedList = Array.from(this.hangmanGuessed).sort().join(", ");
			lines.push(padToCenter(boxLine(`  Guessed: [ ${yellow(guessedList)} ]`)));
			lines.push(padToCenter(boxLine("")));

			if (this.hangmanOver) {
				lines.push(padToCenter(boxLine(`  Press [R] to restart, [Q] for menu`)));
			} else {
				lines.push(padToCenter(boxLine(`  Type any letter A-Z on your keyboard to guess`)));
			}
		} else if (this.currentGame === "recommender") {
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  ${bold("Ultimate Media Recommender Hub")}`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  [1] Random Movie  [2] Random Book`)));
			lines.push(padToCenter(boxLine(`  [3] Random Anime  [4] Random Video Game`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  [Q] Back to Main Menu`)));
			lines.push(padToCenter(boxLine("")));

			if (this.recCategory && this.recSelection) {
				lines.push(padToCenter(boxLine(`  ${dim("────────────────────────────────────────────")}`)));
				lines.push(padToCenter(boxLine(`  Category: ${bold(green(this.recCategory))}`)));
				lines.push(padToCenter(boxLine("")));
				
				const rec = this.recSelection;
				if (rec.length > 40) {
					lines.push(padToCenter(boxLine(`  "${rec.substring(0, 40)}`)));
					lines.push(padToCenter(boxLine(`   ${rec.substring(40)}"`)));
				} else {
					lines.push(padToCenter(boxLine(`  "${rec}"`)));
				}
				lines.push(padToCenter(boxLine("")));
			} else {
				lines.push(padToCenter(boxLine("")));
				lines.push(padToCenter(boxLine("")));
				lines.push(padToCenter(boxLine("")));
				lines.push(padToCenter(boxLine("")));
			}
		} else if (this.currentGame === "matrix") {
			lines.push(padToCenter(boxLine("")));
			
			let scoreStr = `Score: ${green(String(this.matrixScore))} │ High Score: ${yellow(String(this.matrixHighScore))}`;
			let livesStr = `Lives: ${red("❤".repeat(this.matrixLives))}`;
			let statusStr = `  ${scoreStr} │ ${livesStr}`;
			if (this.matrixOver) {
				statusStr = `  ${bold(red(`SYSTEM CRASH! Final Score: ${this.matrixScore}`))}`;
			}
			lines.push(padToCenter(boxLine(statusStr)));
			lines.push(padToCenter(boxLine("")));

			for (let r = 0; r < 11; r++) {
				const activeWord = this.matrixWords.find(w => Math.floor(w.y) === r);
				if (activeWord) {
					const leftSpace = " ".repeat(activeWord.x);
					const typedPart = inverse(green(activeWord.typed));
					const remainingPart = bold(activeWord.text.substring(activeWord.typed.length));
					
					const printedLen = activeWord.x + activeWord.text.length;
					const rightSpaceLen = Math.max(0, boxWidth - printedLen - 4);
					const rightSpace = " ".repeat(rightSpaceLen);

					lines.push(padToCenter(boxLine(`  ${leftSpace}${typedPart}${remainingPart}${rightSpace}`)));
				} else {
					const rainEffects = [
						"       .          o                .      ",
						" .           o           .                ",
						"      o            .             o        ",
						"            .            o                ",
						" .    o            .            .       o ",
						"          .               o               ",
						"    o           o               .         ",
						" .        .            o             o    "
					];
					const rainRow = rainEffects[(r + this.matrixSpawnCooldown) % rainEffects.length];
					lines.push(padToCenter(boxLine(dim(`  ${green(rainRow.substring(0, boxWidth - 4))}`))));
				}
			}
			
			lines.push(padToCenter(boxLine("")));
			if (this.matrixOver) {
				lines.push(padToCenter(boxLine(`  Press [R] to restart, [Q] for menu`)));
			} else {
				lines.push(padToCenter(boxLine(`  Type falling letters to destroy them!`)));
			}
		}

		if (this.confettiTimer > 0) {
			let confettiRowStr = "";
			const activeConfetti = new Map<number, ConfettiParticle>();
			for (const p of this.confetti) {
				activeConfetti.set(p.x, p);
			}
			for (let col = 0; col < boxWidth; col++) {
				const p = activeConfetti.get(col);
				if (p) {
					confettiRowStr += p.color + p.char + "\x1b[0m";
				} else {
					confettiRowStr += " ";
				}
			}
			lines.push(padToCenter(boxLine(` ${confettiRowStr.substring(0, boxWidth - 2)}`)));
		} else {
			lines.push(padToCenter(boxLine("")));
		}

		lines.push(padToCenter(breath + `╰${"─".repeat(boxWidth)}╯\x1b[0m`));

		this.cachedLines = lines;
		this.cachedWidth = width;
		this.cachedVersion = this.version;

		return lines;
	}

	dispose(): void {
		this.stopMatrix();
		if (this.globalInterval) {
			clearInterval(this.globalInterval);
			this.globalInterval = null;
		}
	}
}

export default function (pi: ExtensionAPI) {
	const launchBet = async (_args: any, ctx: any) => {
		if (!ctx.hasUI) {
			ctx.ui.notify("BET requires interactive mode", "error");
			return;
		}

		await ctx.ui.custom((tui: any, _theme: any, _kb: any, done: any) => {
			return new BetNativeComponent(tui, () => {
				done(undefined);
			});
		});
	};

	pi.registerCommand("bet", {
		description: "Play BET natively in the terminal while you wait!",
		handler: launchBet,
	});
	pi.registerCommand("b$t", {
		description: "Play BET natively in the terminal while you wait!",
		handler: launchBet,
	});
	pi.registerCommand("b€t", {
		description: "Play BET natively in the terminal while you wait!",
		handler: launchBet,
	});
	pi.registerCommand("b¥t", {
		description: "Play BET natively in the terminal while you wait!",
		handler: launchBet,
	});
	pi.registerCommand("b￥t", {
		description: "Play BET natively in the terminal while you wait!",
		handler: launchBet,
	});
	pi.registerCommand("b元t", {
		description: "Play BET natively in the terminal while you wait!",
		handler: launchBet,
	});
}
