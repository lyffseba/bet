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

content = content.replace(content[content.find("const HANGMAN_ART: [&str; 7] = ["):content.find("];", content.find("const HANGMAN_ART: [&str; 7] = [")) + 2], new_art.replace("\\", "\\\\").replace("\\\\n", "\\n"))

with open("src/ui.rs", "w") as f:
    f.write(content)

print("Replaced successfully")
