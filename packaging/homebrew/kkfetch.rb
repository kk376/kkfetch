class Kkfetch < Formula
  desc "Fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust"
  homepage "https://github.com/kk376/kkfetch"
  license "MIT"
  version "0.14.0"

  on_macos do
    url "https://github.com/kk376/kkfetch/archive/refs/tags/v0.14.0.tar.gz"
    sha256 "d22652aa971b901bb5828f928609cf81e5a2ce21bb63c60074c1178aae880890"
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
    url "https://github.com/kk376/kkfetch/releases/download/v0.14.0/kkfetch-0.14.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "75b4081aa746795f52710a53f282a6e82da0398d37e0fb1a5402ea45c41faa52"

    def install
      bin.install "kkfetch"
      man1.install "man/kkfetch.1" if File.exist?("man/kkfetch.1")
      bash_completion.install "completions/kkfetch.bash" => "kkfetch" if File.exist?("completions/kkfetch.bash")
      zsh_completion.install "completions/_kkfetch" => "_kkfetch" if File.exist?("completions/_kkfetch")
      fish_completion.install "completions/kkfetch.fish" if File.exist?("completions/kkfetch.fish")
    end
  end

  test do
    assert_match "kkfetch 0.14.0", shell_output("#{bin}/kkfetch --version")
  end
end
