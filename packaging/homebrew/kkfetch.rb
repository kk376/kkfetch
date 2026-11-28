class Kkfetch < Formula
  desc "Fast, lightweight Linux, Windows, macOS, and Android system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  license any_of: ["MIT", "Apache-2.0"]
  version "0.29.0"

  on_macos do
    url "https://github.com/kk376/kkfetch/archive/refs/tags/v0.29.0.tar.gz"
    sha256 "f6d233e67bf96d05dce1efb19ebe4cbe3a2721d800eaf9543af3929cbb6bbb38"
    depends_on "rust" => :build

    def install
      system "cargo", "install", *std_cargo_args
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  on_linux do
    url "https://github.com/kk376/kkfetch/releases/download/v0.29.0/kkfetch-0.29.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "465ccf01dff364fbd3304e19ecdc862854b71546abacf0aa608474b6bd7d458e"

    def install
      bin.install "kkfetch"
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  test do
    assert_match "kkfetch 0.29.0", shell_output("#{bin}/kkfetch --version")
  end
end
