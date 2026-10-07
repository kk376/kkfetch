class Kkfetch < Formula
  desc "Fast, lightweight Linux, Windows, macOS, and Android system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  license any_of: ["MIT", "Apache-2.0"]
  version "0.20.0"

  on_macos do
    url "https://github.com/kk376/kkfetch/archive/refs/tags/v0.20.0.tar.gz"
    sha256 "f41a6e1a4bf52948009561232a68978813e2e7160be1fd600feb5ba9119458d2"
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
    url "https://github.com/kk376/kkfetch/releases/download/v0.20.0/kkfetch-0.20.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "73e7c5f64372782d9d8afa967e50be27ca37403cf536d423acc00e5a63795c5f"

    def install
      bin.install "kkfetch"
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  test do
    assert_match "kkfetch 0.20.0", shell_output("#{bin}/kkfetch --version")
  end
end
