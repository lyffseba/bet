import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { matchesKey, visibleWidth } from "@mariozechner/pi-tui";

type Game = "menu" | "tictactoe" | "hangman" | "recommender";

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

	constructor(tui: { requestRender: () => void }, onClose: () => void) {
		this.tui = tui;
		this.onClose = onClose;
	}

	handleInput(data: string): void {
		if (matchesKey(data, "escape") || data === "q" || data === "Q") {
			if (this.currentGame !== "menu") {
				this.currentGame = "menu";
				this.version++;
				this.tui.requestRender();
			} else {
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
				if (this.board[this.cursorY][this.cursorX] === null) {
					this.board[this.cursorY][this.cursorX] = this.currentPlayer;
					this.checkWinner();
					if (!this.winner && !this.draw) {
						this.currentPlayer = this.currentPlayer === "X" ? "O" : "X";
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

	private checkWinner() {
		const lines = [
			// rows
			[[0,0], [0,1], [0,2]],
			[[1,0], [1,1], [1,2]],
			[[2,0], [2,1], [2,2]],
			// cols
			[[0,0], [1,0], [2,0]],
			[[0,1], [1,1], [2,1]],
			[[0,2], [1,2], [2,2]],
			// diag
			[[0,0], [1,1], [2,2]],
			[[0,2], [1,1], [2,0]]
		];

		for (const line of lines) {
			const [a, b, c] = line;
			if (
				this.board[a[0]][a[1]] &&
				this.board[a[0]][a[1]] === this.board[b[0]][b[1]] &&
				this.board[a[0]][a[1]] === this.board[c[0]][c[1]]
			) {
				this.winner = this.board[a[0]][a[1]];
				return;
			}
		}

		let isDraw = true;
		for (let y = 0; y < 3; y++) {
			for (let x = 0; x < 3; x++) {
				if (this.board[y][x] === null) isDraw = false;
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
			return dim(" │") + content + " ".repeat(padding) + dim("│");
		};

		const padToCenter = (line: string) => {
			const visibleLen = visibleWidth(line);
			const leftPad = Math.max(0, Math.floor((width - visibleLen) / 2));
			return " ".repeat(leftPad) + line;
		};

		lines.push(padToCenter(dim(` ╭${"─".repeat(boxWidth)}╮`)));
		lines.push(padToCenter(boxLine(` ${bold(green("★ B$T (BET) - Terminal Game Hub ★"))} `)));
		lines.push(padToCenter(dim(` ├${"─".repeat(boxWidth)}┤`)));

		if (this.currentGame === "menu") {
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  ${bold("Select an arcade game to play:")}`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  [1] Tic-Tac-Toe  ${dim("(Play vs AI / Local)")}`)));
			lines.push(padToCenter(boxLine(`  [2] Hangman      ${dim("(Movie word survival)")}`)));
			lines.push(padToCenter(boxLine(`  [3] Recommender  ${dim("(Book/Anime/Movie recs)")}`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  [Q] Quit Game Hub`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine("")));
		} else if (this.currentGame === "tictactoe") {
			lines.push(padToCenter(boxLine("")));
			let header = `  Player ${this.currentPlayer}'s turn`;
			if (this.winner) {
				header = `  ${bold(green(`PLAYER ${this.winner} WINS!`))}`;
			} else if (this.draw) {
				header = `  ${bold(yellow("IT'S A DRAW!"))}`;
			}
			lines.push(padToCenter(boxLine(header)));
			lines.push(padToCenter(boxLine("")));

			for (let y = 0; y < 3; y++) {
				let rowStr = "           ";
				for (let x = 0; x < 3; x++) {
					const cell = this.board[y][x] || " ";
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
			
			// Render title
			let statusStr = `  Guess the movie! Attempts left: ${red(String(this.hangmanAttemptsLeft))}`;
			if (this.hangmanOver) {
				statusStr = this.hangmanWon 
					? `  ${bold(green("YOU GUESSED IT! CONGRATULATIONS!"))}`
					: `  ${bold(red("GAME OVER! The movie was: " + this.hangmanWord))}`;
			}
			lines.push(padToCenter(boxLine(statusStr)));
			lines.push(padToCenter(boxLine("")));

			// Display word blanks
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

			// Display HANGMAN Art
			const artIdx = 6 - this.hangmanAttemptsLeft;
			const artLines = HANGMAN_ART[artIdx];
			for (const artLine of artLines) {
				lines.push(padToCenter(boxLine(`      ${dim(artLine)}`)));
			}
			lines.push(padToCenter(boxLine("")));

			// Guessed list
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
				
				// Wrap recommendation line if too long
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
		}

		lines.push(padToCenter(dim(` ╰${"─".repeat(boxWidth)}╯`)));

		this.cachedLines = lines;
		this.cachedWidth = width;
		this.cachedVersion = this.version;

		return lines;
	}
}

export default function (pi: ExtensionAPI) {
	pi.registerCommand("b$t", {
		description: "Play BET natively in the terminal while you wait!",
		handler: async (_args, ctx) => {
			if (!ctx.hasUI) {
				ctx.ui.notify("BET requires interactive mode", "error");
				return;
			}

			await ctx.ui.custom((tui, _theme, _kb, done) => {
				return new BetNativeComponent(tui, () => done(undefined));
			});
		},
	});
}
