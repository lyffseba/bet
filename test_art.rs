fn main() {
    let art = "    +-----------+\n    |           |\n    |           |\n   ( )          |\n  / | \\         |\n /  |  \\        |\n    |           |\n   / \\          |\n  /   \\         |\n /     \\        |\n                |\n=====================";
    for (row, line) in art.lines().enumerate() {
        for (col, c) in line.chars().enumerate() {
            let is_man = row >= 3 && row < 10 && col < 10 && c != ' ';
            if is_man {
                print!("M");
            } else if c == ' ' {
                print!(" ");
            } else {
                print!("X");
            }
        }
        println!();
    }
}
