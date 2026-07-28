class Bet < Formula
  desc "Terminal games + multiplayer virtual-point bets (Rust core)"
  homepage "https://github.com/lyffseba/bet"
  url "https://github.com/lyffseba/bet/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "3d8dc6dfeb656194be9e271186d63149328a4d6186d84d3fdf0137d1aada7ea1"
  license "MIT"
  head "https://github.com/lyffseba/bet.git", branch: "main"

  depends_on "rust" => :build

  def install
    # Cargo workspace: binary is crates/bet-cli
    system "cargo", "install", *std_cargo_args(path: "crates/bet-cli")
  end

  test do
    assert_match "multiplayer", shell_output("#{bin}/bet --help")
    assert_match version.to_s, shell_output("#{bin}/bet --version")
  end
end
