class Kkfetch < Formula
  desc "Fast, lightweight Linux, Windows, macOS, and Android system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  license "MIT"
  version "0.18.1"

  on_macos do
    url "https://github.com/kk376/kkfetch/archive/refs/tags/v0.18.0.tar.gz"
    sha256 "36048b3ea5320ec6265de23065d60de22c326b97076b6a4d1f491e51fb27abb0"
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
    url "https://github.com/kk376/kkfetch/releases/download/v0.18.0/kkfetch-0.18.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "d72a1a55cce12bd51bb3f9f14afb73fa0763ba4e90535f3a724b9a7582943a86"

    def install
      bin.install "kkfetch"
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  test do
    assert_match "kkfetch 0.18.1", shell_output("#{bin}/kkfetch --version")
  end
end
