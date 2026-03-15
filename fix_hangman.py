import re

with open("src/ui.rs", "r") as f:
    content = f.read()

# Instead of using a percentage height like 60% which might cut off if the terminal is exactly 24 lines,
# let's just use 100% height or a larger percentage (90) to guarantee it fits. 
# Also make sure the art fits in the constraints.
content = content.replace("centered_rect(70, 60, area)", "centered_rect(70, 90, area)")

# Hangman art needs at least 13 lines
# "    +-----------+\n
#     |           |\n
#     |           |\n
#    ( )          |\n
#   / | \\         |\n
#  /  |  \\        |\n
#     |           |\n
#    / \\          |\n
#   /   \\         |\n
#  /     \\        |\n
#                 |\n
# ====================="
# This is exactly 12 lines. Constraint::Length(12) is correct.
content = re.sub(
    r"Constraint::Length\(13\), // Hangman art",
    r"Constraint::Length(12), // Hangman art",
    content
)

with open("src/ui.rs", "w") as f:
    f.write(content)

print("Fixed Hangman height to 90%")
