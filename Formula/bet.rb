class Bet < Formula
  desc "A 3A Carmack-level terminal multiplexer with games and utilities"
  homepage "https://github.com/lyffseba/bet"
  url "https://github.com/lyffseba/bet/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "3d8dc6dfeb656194be9e271186d63149328a4d6186d84d3fdf0137d1aada7ea1"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Run a simple test checking the binary outputs something when we give it a nonsense command
    # Wait, the CLI doesn't have a `--version` right now, let's just make sure it executes.
    system "#{bin}/bet", "--help"
  end
end
