class Kkfetch < Formula
  desc "Fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  license "MIT"
  version "0.15.1"

  on_macos do
    url "https://github.com/kk376/kkfetch/archive/refs/tags/v0.15.1.tar.gz"
    sha256 "866d8aba159615f988ba66f169f2153c3c3819949ced9c47df2ad8d1c3303259"
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
    url "https://github.com/kk376/kkfetch/releases/download/v0.15.1/kkfetch-0.15.1-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "30979191565e15787c4e4176b6a1a14578f19d5c70ea0b81a84cf00f665d28bc"

    def install
      bin.install "kkfetch"
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  test do
    assert_match "kkfetch 0.15.1", shell_output("#{bin}/kkfetch --version")
  end
end
