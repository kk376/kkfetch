class Kkfetch < Formula
  desc "Fast, lightweight Linux, Windows, macOS, and Android system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  license any_of: ["MIT", "Apache-2.0"]
  version "0.23.0"

  on_macos do
    url "https://github.com/kk376/kkfetch/archive/refs/tags/v0.23.0.tar.gz"
    sha256 "17665e32a6d95eb004bfe426a8cf21e1cb8b5ed5635777ed82bc7f93eeaf120c"
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
    url "https://github.com/kk376/kkfetch/releases/download/v0.23.0/kkfetch-0.23.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "9fb82cff90e213bee0e16017722e240400b0842fdceb20c6a6a696f211cc2b84"

    def install
      bin.install "kkfetch"
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  test do
    assert_match "kkfetch 0.23.0", shell_output("#{bin}/kkfetch --version")
  end
end
