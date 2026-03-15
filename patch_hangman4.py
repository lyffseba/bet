import re

with open("src/ui.rs", "r") as f:
    content = f.read()

new_art = r"""const HANGMAN_ART: [&str; 7] = [
    // Stage 0
    "    +-----------+\n    |           |\n    |           |\n                |\n                |\n                |\n                |\n                |\n                |\n                |\n                |\n=====================",
    // Stage 1
    "    +-----------+\n    |           |\n    |           |\n   ( )          |\n                |\n                |\n                |\n                |\n                |\n                |\n                |\n=====================",
    // Stage 2
    "    +-----------+\n    |           |\n    |           |\n   ( )          |\n    |           |\n    |           |\n    |           |\n                |\n                |\n                |\n                |\n=====================",
    // Stage 3
    "    +-----------+\n    |           |\n    |           |\n   ( )          |\n  / |           |\n /  |           |\n    |           |\n                |\n                |\n                |\n                |\n=====================",
    // Stage 4
    "    +-----------+\n    |           |\n    |           |\n   ( )          |\n  / | \\         |\n /  |  \\        |\n    |           |\n                |\n                |\n                |\n                |\n=====================",
    // Stage 5
    "    +-----------+\n    |           |\n    |           |\n   ( )          |\n  / | \\         |\n /  |  \\        |\n    |           |\n   /            |\n  /             |\n /              |\n                |\n=====================",
    // Stage 6
    "    +-----------+\n    |           |\n    |           |\n   ( )          |\n  / | \\         |\n /  |  \\        |\n    |           |\n   / \\          |\n  /   \\         |\n /     \\        |\n                |\n=====================",
];"""

content = re.sub(
    r"const HANGMAN_ART: \[\&str; 7\] = \[.*?\];",
    new_art,
    content,
    flags=re.DOTALL
)

# Update layout constraints
content = re.sub(
    r"Constraint::Length\(3\),  // Title\s*Constraint::Length\(10\), // Hangman art",
    r"Constraint::Length(2),  // Title\n                            Constraint::Length(12), // Hangman art",
    content
)

# Update is_man logic
content = re.sub(
    r"let is_man = row >= 2 && row < 9 && col < 8 && c != ' ';",
    r"let is_man = row >= 3 && row < 11 && col < 10 && c != ' ';",
    content
)

with open("src/ui.rs", "w") as f:
    f.write(content)

print("Replaced successfully")
