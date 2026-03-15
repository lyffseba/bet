fn main() {
    let art = "    +-----------+\n    |           |\n    |           |\n   ( )          |\n  / | \\         |\n /  |  \\        |\n    |           |\n   / \\          |\n  /   \\         |\n /     \\        |\n                |\n=====================";
    for (row, line) in art.lines().enumerate() {
        println!("{:2}: length {}", row, line.len());
    }
}
