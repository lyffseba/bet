import re

with open("src/ui.rs", "r") as f:
    content = f.read()

new_art = r"""const HANGMAN_ART: [&str; 7] = [
    // Stage 0
    "   +-------+\n   |       |\n           |\n           |\n           |\n           |\n           |\n           |\n           |\n=============",
    // Stage 1
    "   +-------+\n   |       |\n  ( )      |\n           |\n           |\n           |\n           |\n           |\n           |\n=============",
    // Stage 2
    "   +-------+\n   |       |\n  ( )      |\n   |       |\n   |       |\n           |\n           |\n           |\n           |\n=============",
    // Stage 3
    "   +-------+\n   |       |\n  ( )      |\n  /|       |\n / |       |\n           |\n           |\n           |\n           |\n=============",
    // Stage 4
    "   +-------+\n   |       |\n  ( )      |\n  /|\\      |\n / | \\     |\n           |\n           |\n           |\n           |\n=============",
    // Stage 5
    "   +-------+\n   |       |\n  ( )      |\n  /|\\      |\n / | \\     |\n  /        |\n /         |\n           |\n           |\n=============",
    // Stage 6
    "   +-------+\n   |       |\n  ( )      |\n  /|\\      |\n / | \\     |\n  / \\      |\n /   \\     |\n           |\n           |\n=============",
];"""

content = re.sub(
    r"const HANGMAN_ART: \[\&str; 7\] = \[.*?\];",
    new_art,
    content,
    flags=re.DOTALL
)

with open("src/ui.rs", "w") as f:
    f.write(content)

print("Replaced successfully")
