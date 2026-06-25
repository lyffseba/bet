import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { matchesKey, visibleWidth } from "@mariozechner/pi-tui";

type Game = "menu" | "tictactoe";

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

	constructor(tui: { requestRender: () => void }, onClose: () => void) {
		this.tui = tui;
		this.onClose = onClose;
	}

	handleInput(data: string): void {
		if (matchesKey(data, "escape") || data === "q" || data === "Q") {
			if (this.currentGame !== "menu") {
				this.currentGame = "menu";
				this.resetTicTacToe();
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
				this.version++;
				this.tui.requestRender();
			}
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
		const boxWidth = 40;

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
		lines.push(padToCenter(boxLine(` ${bold(green("BET Native Game Hub"))} `)));
		lines.push(padToCenter(dim(` ├${"─".repeat(boxWidth)}┤`)));

		if (this.currentGame === "menu") {
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  ${bold("Select a game:")}`)));
			lines.push(padToCenter(boxLine("")));
			lines.push(padToCenter(boxLine(`  [1] Tic-Tac-Toe`)));
			lines.push(padToCenter(boxLine(`  [Q] Quit`)));
			lines.push(padToCenter(boxLine("")));
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
				let rowStr = "        ";
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
