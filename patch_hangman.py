import re

with open("src/ui.rs", "r") as f:
    content = f.read()

# Replace HANGMAN_ART array
new_art = """const HANGMAN_ART: [&str; 7] = [
    // Stage 0
    "   +-------+\\n   |       |\\n           |\\n           |\\n           |\\n           |\\n           |\\n           |\\n           |\\n=============",
    // Stage 1
    "   +-------+\\n   |       |\\n  ( )      |\\n           |\\n           |\\n           |\\n           |\\n           |\\n           |\\n=============",
    // Stage 2
    "   +-------+\\n   |       |\\n  ( )      |\\n   |       |\\n   |       |\\n           |\\n           |\\n           |\\n           |\\n=============",
    // Stage 3
    "   +-------+\\n   |       |\\n  ( )      |\\n  /|       |\\n / |       |\\n           |\\n           |\\n           |\\n           |\\n=============",
    // Stage 4
    "   +-------+\\n   |       |\\n  ( )      |\\n  /|\\\\      |\\n / | \\\\     |\\n           |\\n           |\\n           |\\n           |\\n=============",
    // Stage 5
    "   +-------+\\n   |       |\\n  ( )      |\\n  /|\\\\      |\\n / | \\\\     |\\n  /        |\\n /         |\\n           |\\n           |\\n=============",
    // Stage 6
    "   +-------+\\n   |       |\\n  ( )      |\\n  /|\\\\      |\\n / | \\\\     |\\n  / \\\\     |\\n /   \\\\    |\\n           |\\n           |\\n=============",
];"""

content = re.sub(
    r"const HANGMAN_ART: \[\&str; 7\] = \[.*?\];",
    new_art,
    content,
    flags=re.DOTALL
)

# Replace is_man logic
content = content.replace(
    "let is_man = row >= 2 && col < 5 && c != ' ';",
    "let is_man = row >= 2 && col < 8 && c != ' ';"
)

# Replace word_spans push background style
content = content.replace(
    'word_spans.push(Span::styled(format!(" {} ", c), Style::default().bg(Color::Rgb(180, 255, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)));',
    'word_spans.push(Span::styled(format!(" {} ", c), Style::default().fg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD)));'
)

# For game over states where the word might be shown
content = content.replace(
    'word_spans.push(Span::styled(format!(" {} ", c), Style::default().bg(Color::Rgb(180, 255, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)));',
    'word_spans.push(Span::styled(format!(" {} ", c), Style::default().fg(Color::Rgb(180, 255, 50)).add_modifier(Modifier::BOLD)));'
)


with open("src/ui.rs", "w") as f:
    f.write(content)

print("Replaced successfully")
