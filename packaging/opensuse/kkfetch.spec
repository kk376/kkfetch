Name:           kkfetch
Version:        0.19.0
Release:        0
Summary:        Fast, lightweight system information fetch tool written in Rust
License:        MIT OR Apache-2.0
Group:          System/Monitoring
URL:            https://github.com/kk376/kkfetch
Source0:        https://github.com/kk376/kkfetch/archive/refs/tags/v%{version}.tar.gz#/%{name}-%{version}.tar.gz
Source1:        vendor.tar.zst
ExclusiveArch:  x86_64 aarch64

BuildRequires:  cargo >= 1.75.0
BuildRequires:  rust >= 1.75.0
BuildRequires:  zstd
BuildRequires:  gcc

%description
KKFetch is a fast, zero-runtime-dependency CLI system information fetch tool
written in Rust, specifically designed for Linux distributions. It gathers system
metrics including OS release, kernel version, CPU, GPU, memory, disk usage,
package managers, desktop environment, uptime, and shell information, formatting
them cleanly alongside colorful ANSI distribution ASCII logos.

%prep
%autosetup -n %{name}-%{version}
tar -I zstd -xf %{SOURCE1}
sed -i 's/version = "1.1.0"/version = "1.0.0"/' Cargo.lock
sed -i 's/c8d4a3bb8b1e0c1050499d1815f5ab16d04f0959b233085fb31653fbfc9d98f9/3a822ea5bc7590f9d40f1ba12c0dc3c2760f3482c6984db1573ad11031420831/' Cargo.lock
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
cargo build --release --offline

%check
cargo test --release --offline

%install
# Install executable binary
install -Dpm 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install shell completions
install -Dpm 0644 completions/%{name}.bash %{buildroot}%{_datadir}/bash-completion/completions/%{name}
install -Dpm 0644 completions/_%{name} %{buildroot}%{_datadir}/zsh/site-functions/_%{name}
install -Dpm 0644 completions/%{name}.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/%{name}.fish

# Install man page
if [ -f docs/%{name}.1 ]; then
    install -Dpm 0644 docs/%{name}.1 %{buildroot}%{_mandir}/man1/%{name}.1
fi

# Install documentation
install -Dpm 0644 README.md %{buildroot}%{_docdir}/%{name}/README.md

%files
%license LICENSE LICENSE-MIT LICENSE-APACHE
%doc README.md
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*
%{_datadir}/bash-completion/completions/%{name}
%dir %{_datadir}/zsh
%dir %{_datadir}/zsh/site-functions
%{_datadir}/zsh/site-functions/_%{name}
%dir %{_datadir}/fish
%dir %{_datadir}/fish/vendor_completions.d
%{_datadir}/fish/vendor_completions.d/%{name}.fish

%changelog
* Fri Oct 02 2026 Kushagra Kumar (kk376) <kk376@users.noreply.github.com> - 0.19.0-0
- Release version 0.19.0 for openSUSE with offline vendored sources

* Thu Oct 01 2026 Kushagra Kumar (kk376) <kk376@users.noreply.github.com> - 0.18.3-0
- Release version 0.18.3 for openSUSE with offline vendored sources
