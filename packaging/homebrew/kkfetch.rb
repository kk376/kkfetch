class Kkfetch < Formula
  desc "Fast, lightweight Linux, Windows, macOS, and Android system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  license any_of: ["MIT", "Apache-2.0"]
  version "0.18.3"

  on_macos do
    url "https://github.com/kk376/kkfetch/archive/refs/tags/v0.18.3.tar.gz"
    sha256 "05fb3b151b76cd168edc2af396b1e28abb5355cf7dbe0c031aee5b4104bd963a"
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
    url "https://github.com/kk376/kkfetch/releases/download/v0.18.3/kkfetch-0.18.3-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "a35cb4e6045e10604b390736a082b470411843ad4ce7f59b5f1837b95d2c2472"

    def install
      bin.install "kkfetch"
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  test do
    assert_match "kkfetch 0.18.3", shell_output("#{bin}/kkfetch --version")
  end
end
