# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.18.0] - 2026-09-25

### Performance & Optimizations
- **Persistent Disk Caching Across Reboots**: Migrated volatile `$XDG_RUNTIME_DIR` caches for terminal and battery collectors to persistent `$XDG_CACHE_HOME/kkfetch/` (`~/.cache/kkfetch/`). Eliminates cache misses on fresh boots where `/run` tmpfs is cleared, bringing first run latency down from 152 ms to sub 1 ms.
- **Binary Mtime Cache Validation**: Validates persistent terminal version caches against the executable binary modification timestamp, preventing stale version reporting while avoiding expensive child process spawning (such as 50 ms GTK/libadwaita initialization for `ghostty --version`).
- **Non-blocking AC Power State Transitions**: Reading power supply AC adapter state (`/sys/class/power_supply/ADP1/online`) in 150 µs without blocking on the battery Embedded Controller (EC) SMBus. Dispatches capacity and battery charge updates asynchronously in a background thread to maintain sub 1 ms interactive CLI execution.
- **Parallel Dispatch for Battery Module**: Removed battery from synchronous fast path to ensure any cold cache read executes in parallel within a scoped background thread without blocking the main output thread.

### Documentation & Packaging
- **Updated Benchmarks & Terminal Preview**: Refreshed `README.md` with new statistical benchmarks against Fastfetch and updated terminal preview showing live Wayland scaling and persistent cache performance.
- **Updated Package Manifests**: Synchronized 0.18.0 release across Fedora Copr (RPM spec), Ubuntu Launchpad PPA (Noble changelog), Homebrew tap Formula, Arch Linux PKGBUILD, Gentoo (`kkfetch-0.18.0.ebuild`), KISS Linux, Void Linux, Nix, and WinGet.

## [0.17.0] - 2026-09-24

### Added
- **Dynamic Wayland Compositor Scale Detection**: Live IPC querying of active compositor monitor scale for Hyprland (`hyprctl -j monitors`) and Sway/wlroots (`swaymsg -t get_outputs -r`), providing instant and accurate scale reporting without depending on static configuration files.
- **X11 DPI Display Scaling**: Probes live X11 display resolution scale factor via `xrdb -query` (`Xft.dpi`), accurately calculating scale factors (such as 1.25x for 120 DPI, 1.50x for 144 DPI).
- **Compositor Authoritative Cursor Resolution**: Overlays live compositor environment variables (`HYPRCURSOR_THEME`, `HYPRCURSOR_SIZE`, `XCURSOR_THEME`, `XCURSOR_SIZE`) to ensure reported cursor sizes strictly match the active Wayland compositor surface rather than legacy dconf values.
- **Session-Bound Theme Cache Invalidation**: Binds `~/.cache/kkfetch/theme_v2.cache` directly to the active desktop session and cursor size, automatically invalidating stale caches when switching between desktop environments or window managers.

### Changed
- **Cross-Session Residue Elimination**: Removed over-permissive fallbacks in `display.rs` and `theme.rs`. Gated `monitors.xml` strictly to GNOME/Mutter sessions, `kdeglobals` strictly to KDE/Plasma sessions, and `xsettings.xml` to XFCE sessions, eliminating false positive scale and theme leakage across window managers.
- **Contextual GTK Interface Labeling**: Updated GSettings source tagging to dynamically render as `[GTK]` on tiling window managers (Hyprland, Sway, River, i3) while reserving `[GTK/GNOME]` for active GNOME desktop sessions.
- **Documentation & Packaging**: Synchronized all package manifests and installation guides to `0.17.0`, with updated terminal preview in `README.md`.

### Packaging
- **Updated Package Manifests**: Synchronized 0.17.0 release across Fedora Copr (RPM spec), Ubuntu Launchpad PPA (Noble changelog), Homebrew tap Formula, Arch Linux PKGBUILD, Gentoo (`kkfetch-0.17.0.ebuild`), KISS Linux, Void Linux, Nix, and WinGet.

## [0.16.0] - 2026-09-21

### Added
- **Fractional Display Scaling**: Accurate detection and rendering of compositor fractional scaling multipliers (such as `@ 1.33x in 15", 144 Hz [Built-in]`) by parsing `~/.config/monitors.xml` and Wayland compositor configuration.
- **Live Desktop Version Invalidation**: Desktop Environment cache validated against binary modification timestamps (`mtime` of `/usr/bin/gnome-shell`, `plasmashell`, etc.) to eliminate stale version reporting immediately after system upgrades.
- **Categorized Local IP with CIDR Notation**: User-friendly network interface classification (`Local IP (Wi-Fi)`, `Local IP (Ethernet)`, `Local IP (Cellular)`, `Local IP (Loopback)`) with CIDR netmask notation.
- **Automated SRPM Generation**: Added `.copr/Makefile` for seamless automated builds in Fedora Copr.

### Changed
- **Packaging Stale Cache Cleanup**: Added automated cleanup of stale user caches (`de_*.cache`, `theme_*.cache`) during package upgrades across RPM, Debian, Arch Linux, and standalone `install.sh`.
- **Documentation & Credits**: Synchronized all command references and download links in `README.md` to `0.16.0`, with credit given to Fastfetch for CIDR subnet notation.

### Packaging
- **Updated Package Manifests**: Synchronized 0.16.0 release across Fedora Copr (RPM spec), Ubuntu Launchpad PPA (Noble changelog), Homebrew tap Formula, Arch Linux PKGBUILD, Gentoo (`kkfetch-0.16.0.ebuild`), KISS Linux, Void Linux, Nix, and WinGet.

## [0.15.2] - 2026-09-21

### Changed
- **Automatic Silent Binary Conflict Resolution**: Package post-install hooks (RPM `%post`, Debian `postinst`, Arch `kkfetch.install`) and `install.sh` now automatically and silently remove any older conflicting Cargo or local binaries from `$USER_HOME/.cargo/bin/kkfetch` and `$USER_HOME/.local/bin/kkfetch` upon upgrade, eliminating terminal warnings and manual deletion steps.
- **Clean Version Output**: Removed binary shadowing checks and warning messages from `-V, --version`, preserving quiet single-line output. Diagnostic health inspection remains available via `kkfetch --doctor`.
- **Homebrew Tap Renamed**: Transitioned official Homebrew tap repository to `kk376/homebrew-kkfetch`, enabling direct installation via `brew tap kk376/kkfetch && brew install kkfetch`.

## [0.15.1] - 2026-09-21

### Added
- **Installation Health Doctor (`--doctor`)**: Added diagnostic tool to scan system `$PATH` for multiple, duplicate, or shadowed `kkfetch` binary installations (such as older binaries installed via `cargo install` residing in `~/.cargo/bin` shadowing system packages in `/usr/bin`). Displays active vs shadowed paths with resolution steps.
- **Binary Shadowing Warning on Version Query (`-V, --version`)**: Enhanced version check to inspect `$PATH` for conflicting binaries and display an informative notice when an older binary or multiple installations are detected.
- **Packaging Post-Install Detection**: Added `%post` scriptlet in RPM spec, post-installation hook in Debian package (`postinst`), and Arch Linux install scriptlet (`kkfetch.install`) to inspect the user's home directory for existing Cargo or local binaries upon package installation or upgrade, alerting users to remove shadowed binaries to prevent version conflicts.
- **Standalone Installer (`install.sh`)**: Added a zero-dependency shell installer script with automatic binary shadowing detection, shell completions setup, and man page installation.

### Changed
- **Shell Completions**: Updated Bash, Zsh, and Fish completion scripts to include `--doctor`, `--timings`, and `--no-plugins`.

## [0.15.0] - 2026-09-21

### Added
- **Colorized Battery Time Estimates**: Dynamic color rendering for battery charge and discharge telemetry (`xh ym remaining` / `xh ym until full`). Charging estimates are styled in bright green (`\x1b[32m`), while discharging estimates transition dynamically based on battery capacity (green for >= 50%, yellow for 20% to 49%, red for < 20%).
- **Instantaneous AC Power Transition Detection**: Probes live AC power supply online status via sysfs (`/sys/class/power_supply/*/online`) in sub-60-microsecond latency. When the AC adapter is plugged in or unplugged, stale battery cache is bypassed immediately, updating battery metrics without delay.

### Changed
- **Optimized Battery Cache TTL**: Reduced default battery telemetry cache time-to-live from 30 seconds to 5 seconds, providing responsive estimates while preventing excessive battery sysfs wakeups.
- **In-Memory Module Threading Bypass**: Streamlined execution of 13 pure in-memory collectors (`title`, `os`, `kernel`, `cpu`, `theme`, `icons`, `font`, `cursor`, `colors`, `wm`, `wmtheme`, `terminal`, `terminalfont`) directly on the main thread, avoiding thread spawn and context-switch latency, reducing OS thread pool allocation from 27 down to ~10 threads.
- **Platform-Targeted Package Manager Scans**: Excluded non-native package managers on Linux (Windows-specific Winget, Scoop, Chocolatey; Darwin MacPorts; FreeBSD pkg) to eliminate unnecessary directory probing and system call overhead.

### Packaging
- **Updated Package Manifests**: Synchronized 0.15.0 release metadata across RPM spec, Debian changelog, Homebrew formula, Arch PKGBUILD, Alpine APKBUILD, Gentoo ebuild, KISS Linux, Void Linux, Nix, and WinGet.

## [0.14.5] - 2026-09-12

### Fixed
- **Version String & Binary Synchronization**: Hotfix release synchronizing binary `--version` output, packaging manifests, and documentation sample output with the latest system information metrics.

### Changed
- **Updated Sample Output**: Refreshed system info display in README with latest Linux 7.2.4 kernel, updated uptime, package counts, and environment metrics.

### Packaging
- **Horizon-Wide Package Release**: Refreshed release packages across Fedora Copr, Ubuntu Launchpad PPA, Homebrew tap, Arch Linux, Termux, Windows standalone binaries, and musl static builds.

## [0.14.0] - 2026-09-12

### Fixed
- **Hardware-Fingerprinted GPU Cache Invalidation**: Resolved stale GPU model reporting when swapping a storage drive or NVMe SSD across differing hardware platforms (e.g. Intel to AMD Ryzen laptop). `get_gpu_list_uncached()` now generates an ultra-fast sysfs PCI display controller hardware fingerprint (`PCI:<vendor>:<device>,...`) in < 15 microseconds, embedding it into `# FINGERPRINT:<fp>` headers in `~/.cache/kkfetch/gpu_list_v2.cache`. Mismatched hardware fingerprints or legacy unversioned caches are automatically invalidated and re-probed from sysfs without user intervention. *(Discovered & reported by laysnb)*

### Changed
- **Updated Fastfetch Benchmarks**: Re-ran comparative 100-run statistical benchmarks with `hyperfine` on bare-metal Fedora 44 (Linux 7.2.4-200.fc44.x86_64, AMD Ryzen 5 7535HS), clocking `kkfetch` at ~4.0 ms mean runtime (5.3x faster than Fastfetch at 21.4 ms).

### Packaging
- **Updated Package Manifests**: Synchronized 0.14.0 release configurations across Fedora Copr (RPM spec), Ubuntu Launchpad PPA (Noble changelog), Homebrew tap Formula, Arch Linux PKGBUILD, Gentoo (`kkfetch-0.14.0.ebuild`), KISS Linux, Alpine Linux, Void Linux, Nix, and WinGet.

## [0.13.0] - 2026-09-04

### Added
- **No-Plugins Execution Flag**: Added `--no-plugins` command-line option to explicitly disable third-party shell plugin execution in automated, headless, or security-sensitive environments.
- **Darwin (macOS) Kernel Boot Detection**: Integrated native Darwin `sysctl` (`CTL_KERN`, `KERN_BOOTTIME`) fallback in the uptime module when `libc::sysinfo` is unavailable.
- **Darwin & BSD Filesystem Metadata Fallback**: Added portable APFS/HFS+ file creation and modification timestamp resolution when Linux `statx` direct syscalls are unsupported.
- **Cross-Platform Matrix Testing**: Added Windows and macOS CI test matrices to ensure cross-platform compatibility across releases.

### Fixed
- **Win32 Pointer Alignment**: Resolved potential unaligned pointer dereferences in `win_util.rs` via `std::ptr::copy_nonoverlapping` into properly aligned stack structures, eliminating undefined behavior.
- **Windows Platform Gates**: Gated default terminal cache directory and disk storage constants with appropriate platform cfg attributes, and fixed Windows test suite imports in `battery.rs`.
- **Collector Panic Module Attribution**: Corrected multi-threaded collector panic reporting to attribute errors to the specific module identifier rather than a generic index.
- **Darwin Statx Guard**: Scoped Linux-specific `libc::SYS_statx` direct syscall to `#[cfg(target_os = "linux")]` to resolve compilation failures on macOS and BSD.

### Security
- **Hardened Unsafe Invariants**: Audited and documented formal `// SAFETY:` invariants across all FFI blocks, enforcing compile-time compliance via `#![warn(clippy::undocumented_unsafe_blocks)]`.
- **CI Security Audit**: Added automated `cargo-audit` verification job into the continuous integration workflow.

### Packaging
- **Updated Package Manifests**: Synchronized 0.13.0 release configurations across Fedora Copr (RPM spec), Ubuntu Launchpad PPA (Noble changelog), Homebrew tap Formula, Arch Linux PKGBUILD, Gentoo (`kkfetch-0.13.0.ebuild`), KISS Linux, Alpine Linux, Termux, Void Linux, Nix, and WinGet.

## [0.12.0] - 2026-09-01

### Changed
- **Official Project Rebrand to KKFetch**: Full transition from `ferrisfetch` to `kkfetch`, including binary name migration, default config directory move to `~/.config/kkfetch/`, updated completions, manual pages, and multi-distro packaging templates.
- **Maintainer Identity**: Updated project authorship and maintainership to Kushagra Kumar (`kk376`).

### Security
- **Documented Unsafe Invariants**: Added formal `// SAFETY:` explanatory comments across all `unsafe` blocks in 7 modules (`context.rs`, `cpu.rs`, `uptime.rs`, `battery.rs`, `installed.rs`, `localip.rs`, `plugin.rs`), verifying OS kernel invariants and memory safety.
- **Enforced Safety Lint**: Activated `#![warn(clippy::undocumented_unsafe_blocks)]` in `main.rs` to guarantee compile-time safety documentation across the codebase.
- **Automated Security Auditing**: Added automated `cargo audit` workflow to GitHub Actions CI pipeline.
- **Plugin Threat Model**: Formalized threat model and privilege-boundary mitigations in `SECURITY.md`.

### Packaging & CI
- **Multi-Platform Release Pipeline**: Upgraded GitHub Actions release automation to build standalone static musl binaries (x86_64, aarch64), Windows x86_64 zip, Termux arm64, Debian, RPM, and Arch Linux packages.
- **Ubuntu Noble PPA**: Configured Launchpad PPA packaging for Ubuntu 24.04 LTS (`noble`).

## [0.11.7] - 2026-08-29

### Security
- **Plugin Privilege Escalation Guardrail (F1)**: Automatically disable arbitrary shell plugin execution when `kkfetch` is invoked in elevated contexts (`sudo`, `su`, `setuid` where `euid == 0` or `euid != uid`) to prevent privilege escalation via untrusted user configs. *(Discovered & reported by Laysnb)*
- **Strict Executable Bit & Ownership Validation for Plugins (F2)**: Enforce regular file ownership and executable permission bit (`+x`) checks on `~/.config/kkfetch/plugins/` scripts, ignoring non-executable files, hidden files, and editor swap/backup files. *(Discovered & reported by Laysnb)*
- **Canonical System Path Resolution for Subprocesses (F3)**: Replaced bare-name `$PATH` resolution for all helper commands (`lspci`, `dpkg-query`, `rpm`, `gsettings`, `dconf`, `xrandr`, `wlr-randr`, `getprop`) with canonical trusted system directories (`/usr/bin`, `/bin`, `/usr/sbin`, `/sbin`, `/usr/local/bin`), eliminating untrusted PATH search hijacking. *(Discovered & reported by Laysnb)*
- **User-Isolated Private Cache Path Resolution (F4)**: Hardened tmpfs caching in `battery` and `terminal` modules to prefer `$XDG_RUNTIME_DIR` (mode 0700) and `$XDG_CACHE_HOME` (`~/.cache/kkfetch/`), creating private user-owned directories (`/tmp/kkfetch-<uid>/`) with `0o700` permissions on temp fallback. *(Discovered & reported by Laysnb)*
- **Terminal Control Sequence Sanitization & JSON Key Escaping (F5)**: Added `sanitize_terminal_string` to strip dangerous OSC terminal manipulation codes and raw non-printable C0 control characters from external display values, and implemented complete escaping for JSON output keys. *(Discovered & reported by Laysnb)*

## [0.11.6] - 2026-08-29

### Optimized
- **Zero-Wait Stale-While-Revalidate Battery Architecture**: Implemented asynchronous background revalidation for Linux sysfs battery telemetry, completely eliminating hardware ACPI EC bus stalls (~100ms) on all runs and guaranteeing sub-millisecond ($< 1\text{ ms}$) execution times on every single invocation.

## [0.11.5] - 2026-08-29

### Optimized
- **Instantaneous Terminal Version Probing & Tmpfs Runtime Caching**: Added mtime-aware persistent tmpfs runtime caching for GUI terminal emulator version detection (`Ptyxis`, `GNOME Terminal`, `Kitty`, `Alacritty`, `Konsole`, `WezTerm`, `Foot`), reducing terminal module latency from ~65ms down to **26 µs** and dropping total wall-clock runtime below **1 ms**.

## [0.11.4] - 2026-08-29

### Added
- **Exhaustive Multi-Vendor Integrated & SoC GPU Taxonomy**: Added comprehensive hardware taxonomy catalog spanning AMD APU generations (Mendocino, Phoenix, Rembrandt, Hawk Point, Strix Point, A-series, E-series, D-series desktop APUs, R2-R7), Intel (Arc iGPUs, Lunar Lake 140V/130V, Meteor Lake, Iris Xe, UHD, HD, GMA), Apple Silicon (M1-M4, A-series), Qualcomm Snapdragon Adreno (all 5xx-8xx & X Elite), ARM Mali/Immortalis, Broadcom VideoCore, Samsung Xclipse, NVIDIA Tegra, and virtual hypervisor adapters (VirtIO, VMware, VirtualBox, Hyper-V, QXL, BMC ASPEED/Matrox).

## [0.11.3] - 2026-08-29

### Fixed
- **Mobile APU & SoC Integrated GPU Classification**: Expanded iGPU pattern detection to properly recognize AMD Radeon 610M (Mendocino APUs), 600/700/800-series mobile APU processors, Intel Arc iGPUs, and mobile SoCs as `[Integrated]` graphics on single-GPU and hybrid configurations (reported by [@Laynsb](https://github.com/Laynsb)).

## [0.11.2] - 2026-08-29

### Optimized
- **Instantaneous Battery Probing & Tmpfs Runtime Cache**: Implemented single-pass sysfs scan with non-blocking 15-second TTL tmpfs runtime caching in `$XDG_RUNTIME_DIR/kkfetch_battery.cache` to completely eliminate hardware ACPI EC bus latency spikes on laptops, guaranteeing deterministic sub-3ms parallel execution.

## [0.11.1] - 2026-08-29

### Added
- **Dedicated GPU VRAM Memory Probing**: Zero-fork GPU memory scanning parsing `mem_info_vram_total` from sysfs and 64-bit prefetchable memory apertures from `/sys/bus/pci/devices/*/resource` to report dedicated VRAM size (e.g. `512 MiB`, `4 GiB`).
- **Windows Display Adapter VRAM Parsing**: Extracted `HardwareInformation.qwMemorySize` and `HardwareInformation.MemorySize` directly from Windows display adapter registry classes.
- **Updated Statistical Benchmarks**: Re-benchmarked on Fedora Linux 44 with sub-3ms average latency across all 27 active modules.

## [0.11.0] - 2026-08-29

### Added
- **Zero-Dependency TOML Configuration System**: Added hierarchical configuration parser loading `~/.config/kkfetch/config.toml` (and `/etc/kkfetch/config.toml`) supporting module ordering, custom labels, logo presets, colors, separator formatting, and per-module settings.
- **Custom Info Module Loader & Script Plugin System**: Parallel execution of external scripts in `~/.config/kkfetch/plugins/` and custom command modules declared in `config.toml`.
- **CPU Topology & Dual Live Clock Frequency**: Physical cores vs logical threads reporting (`6c 12t`) alongside live instantaneous core clock and rated maximum turbo boost frequency (`@ 4.351GHz [4.60GHz max]`).
- **Extended Zero-Fork Sysfs EDID DRM Parser**: Decodes 128-byte EDID binary payloads from `/sys/class/drm/card*-*/edid` to extract PNP monitor manufacturer code, diagonal physical inch size, native resolution, refresh rate, and connector type.
- **System Font, WM Theme, and Terminal Font Collectors**: Native parsers for GTK 2/3/4 `settings.ini`, GNOME GSettings/dconf, KDE `kdeglobals`, and Kitty, Alacritty, and Foot dotfiles.
- **Cursor Theme & Pixel Size Collector**: Resolves cursor theme and size from GTK, GNOME, and KDE configurations.
- **Terminal Emulator Version Detection**: Probes version numbers across all major terminal emulators (Kitty, Alacritty, Foot, WezTerm, Ghostty, GNOME Terminal, GNOME Console, Konsole, XFCE Terminal, MATE Terminal, Tilix, Terminator, tmux, Zellij, Rio, Contour, BlackBox, Ptyxis, xterm, VS Code).
- **Deep Terminal Capability Probing**: Real-time detection of 24-bit TrueColor, UTF-8 Unicode, and Nerd Fonts support.
- **Multi-OS Support & Dedicated ASCII Art Logos**: Added brand-colored ASCII logos and native OS detection for **Android** (Termux), **macOS** (Darwin), **OpenBSD**, and **NetBSD**.
- **Microsecond Profiling Mode (`--timings`)**: Per-module latency diagnostics showing individual execution durations and wall-clock times.

## [0.10.2] - 2026-08-24

### Fixed
- **Permanent Dual-Boot RTC Skew Detection**: Inspects `/etc/adjtime` for `LOCAL` mode to deterministically normalize rootfs birth timestamps on dual-boot Windows/Linux machines regardless of current uptime or time-of-day.
- **Calendar-Day Relative Time Resolution**: Computes installation age based on localized calendar day midnight boundaries, correctly displaying `yesterday` (instead of `today`) when crossed over midnight.
- **Deduplicated Desktop & Window Manager Display**: Suppresses redundant default compositor annotations in Desktop module (e.g. `GNOME 50.4 (Wayland)` instead of repeating `Mutter` alongside the dedicated `WM` module).

## [0.10.1] - 2026-08-24

### Fixed
- **Clean GPU Marketing Model Resolution**: Automatically extracts consumer product brand names from bracketed PCI hardware identifiers (e.g. `[GeForce RTX 2050]`, `[Radeon 680M]`), eliminating internal raw silicon codenames (`GA107`, `Rembrandt`).
- **Dual-Boot RTC Installer Skew Normalization**: Automatically detects and normalizes filesystem birth time offsets caused by Live USB installer hardware clock assumptions on dual-boot Windows/Linux systems.

## [0.10.0] - 2026-08-23

### Optimized
- **Zero-Subprocess Windows Shell & Terminal Resolution**: Replaced process spawning with native Win32 Toolhelp32 process snapshot ancestry traversal (`CreateToolhelp32Snapshot`), eliminating 200ms–400ms startup latency.
- **Accurate Windows Shell Detection**: Resolved Command Prompt (`cmd.exe`) version directly from Registry `CurrentBuildNumber` + `UBR` (`CMD 10.0.<build>.<ubr>`).
- **Dedicated Windows Package Manager Pipeline**: Added Scoop package manager discovery (`%USERPROFILE%\scoop\apps`) and bypassed Linux filesystem probes on Windows.

## [0.9.9] - 2026-08-23

### Fixed
- **Vendored Package Checksum Preservation**: Preserved crate package checksum hashes while zeroing file-level maps in `.cargo-checksum.json` for full compatibility with Cargo lockfile verification during offline builds.

## [0.9.8] - 2026-08-23

### Fixed
- **Launchpad PPA Checksum Normalization**: Fixed vendored crate `.cargo-checksum.json` generation and cargo vendor path configuration for Ubuntu Noble offline builds.

## [0.9.7] - 2026-08-23

### Optimized
- **RPM MTIME Package Caching**: Added persistent mtime-based disk caching for RPM database queries, slashing Fedora package query times from ~1.5s to <0.05ms.
- **DRM-First Display Resolution**: Prioritized direct kernel sysfs DRM connector parsing over graphical server roundtrips, eliminating unnecessary `xrandr` / `wlr-randr` subprocesses on Wayland.
- **Desktop Environment Caching**: Added persistent caching for desktop environment versions to eliminate heavy runtime subprocess spawning (e.g. `gnome-shell --version`).

## [0.9.6] - 2026-08-23

### Added
- **WSL Host Version Discovery**: Probes and reports host WSL version on the Host line (e.g. `Host: Windows Subsystem for Linux - 2.7.12.0`).
- **Vendored Checksum Normalization**: Optimized Debian/PPA offline cargo build compatibility.

## [0.9.5] - 2026-08-22

### Added
- **WSLg Version Discovery**: Probes and reports the active WSLg version from `/mnt/wslg/versions.txt` (e.g. `WM: WSLg 1.0.73.2 (Wayland)`).
- **GPU Type Classification**: Automatically identifies and annotates `[Integrated]` vs `[Discrete]` GPUs across hybrid laptop and multi-GPU workstation setups.

## [0.9.0] - 2026-08-22

### Added
- **Enhanced ASCII Distro Art**: High-contrast white outer framing with brand-colored inner emblems across all 26+ distributions.
- **WSL2 Storage Resolution**: Normalized 9p/drvfs virtualization filesystem mappings to native NTFS for Windows drive mounts.

## [0.8.5] - 2026-08-22

### Added
- **Neofetch Dual-Tone ASCII Art Suite**: Integrated the complete, classic multi-color ASCII art logo suite from Neofetch across all 26 supported distributions and operating systems (Ubuntu, Fedora, Arch Linux, Debian, Linux Mint, NixOS, openSUSE, Gentoo, Void, Pop!_OS, RHEL, Rocky, AlmaLinux, EndeavourOS, Manjaro, Alpine, Kali, FreeBSD, Slackware, Artix, Zorin, Windows 11/10/7, Tux, and Ferris the Crab).
- **Dual-Tone Color Token Rendering Engine**: Added internal ANSI color token parsing (`{p}` for primary distro color, `{a}` for accent/white highlights, `{0}` for reset) with automated ANSI stripping for `--no-color` mode and zero-distortion column alignment.
- **Filesystem Type Discovery on Disks**: Enumerates filesystem types (e.g. `ext4`, `btrfs`, `ntfs`, `9p`, `vfat`, `zfs`) across all mounted storage partitions on Linux (`/proc/mounts`) and Windows (`GetVolumeInformationW`) with zero subprocess overhead (suggested by [@Laynsb](https://github.com/Laynsb)).
- **ZRAM Compression Algorithm Discovery on Swap**: Detects active in-memory swap compression algorithms from `/sys/block/zram*/comp_algorithm` (e.g. `Swap: 0.00 GiB / 4.00 GiB (0%) - LZ4`) on ZRAM-enabled distributions (Fedora, Pop!_OS, ChromeOS, Android), while leaving traditional swap files/partitions clean (suggested by [@Laynsb](https://github.com/Laynsb)).

## [0.8.0] - 2026-08-22

### Added
- **Filesystem Type Discovery on Disks**: Enumerates filesystem types (e.g. `ext4`, `btrfs`, `ntfs`, `9p`, `vfat`, `zfs`) across all mounted storage partitions on Linux (`/proc/mounts`) and Windows (`GetVolumeInformationW`) with zero subprocess overhead (suggested by [@Laynsb](https://github.com/Laynsb)).
- **ZRAM Compression Algorithm Discovery on Swap**: Detects active in-memory swap compression algorithms from `/sys/block/zram*/comp_algorithm` (e.g. `Swap: 0.00 GiB / 4.00 GiB (0%) - LZ4`) on ZRAM-enabled distributions (Fedora, Pop!_OS, ChromeOS, Android), while leaving traditional swap files/partitions clean (suggested by [@Laynsb](https://github.com/Laynsb)).
- **High-Fidelity Distro ASCII Art Logos**: Redesigned all distribution and OS ASCII art logos with proportionally taller, high-contrast silhouettes across Ubuntu, Fedora, Debian, Arch Linux, Linux Mint, RHEL, Rocky, AlmaLinux, openSUSE, Gentoo, Void, Pop!_OS, NixOS, Kali, FreeBSD, Windows 11/10/7, and Tux (suggested by [@Laynsb](https://github.com/Laynsb)).

## [0.7.0] - 2026-08-22

### Added
- **Localized Installation Timestamps**: System installation date and time (`Installed:`) is now automatically formatted in the user's local timezone (including daylight saving time adjustments) using native OS APIs (`localtime_r` / `tm_gmtoff` on POSIX systems and `GetTimeZoneInformation` on Windows), replacing raw UTC+0 display (suggested by [@Laynsb](https://github.com/Laynsb)).

## [0.6.0] - 2026-08-22

### Added
- **Native Windows NT Platform Support**: Full native Win32 execution without WSL or external runtime dependencies.
- **Win32 Hardware & System Probers**:
  - **OS**: Probes Windows product name, display version, and build number from registry (`HKLM\...\CurrentVersion`), with automatic Windows 11 upgrade build detection.
  - **Kernel**: Reports `Windows NT <version>.<build>`.
  - **Host**: Resolves manufacturer, product name, and BIOS version from `HKLM\HARDWARE\DESCRIPTION\System\BIOS`.
  - **CPU**: Probes processor name and clock frequency from central processor registry keys and queries active logical processor count.
  - **Memory & Swap**: Queries physical RAM and pagefile capacity via Win32 `GlobalMemoryStatusEx`.
  - **GPU**: Discovers display adapters and dedicated VRAM from video controller registry keys.
  - **Disks**: Enumerates Windows drive letters (`C:\`, `D:\`, etc.) and storage metrics via Win32 `GetDiskFreeSpaceExW`.
  - **Battery**: Queries battery capacity, charging status, and AC line state via `GetSystemPowerStatus`.
  - **Uptime**: Computes system elapsed uptime via `GetTickCount64`.
  - **Install Date**: Formats system installation timestamp from registry records with relative time delta.
  - **Theme**: Detects Windows Light/Dark mode preference from personalization registry keys.
  - **Desktop & WM**: Reports `Windows Explorer` and `Desktop Window Manager (DWM)`.
- **Windows Package Managers**: Native package counting for **WinGet** (`%LOCALAPPDATA%\Microsoft\WinGet`), **Chocolatey** (`C:\ProgramData\chocolatey\lib`), and **Cargo** (`.crates.toml`).
- **Windows Shells & Terminals**: Detection and version parsing for **PowerShell 7** (`pwsh`), **Windows PowerShell 5.1** (`powershell`), **Command Prompt** (`cmd`), **Nushell** (`nu`), **Windows Terminal** (`$WT_SESSION`), and **Console Window Host** (`ConHost`).
- **Windows ASCII Art Logos**: Added high-resolution ANSI logos for **Windows 11**, **Windows 10**, and **Classic Windows / Windows 7**.
- **Distribution Channels**: Added packaging manifests for **WinGet**.

## [0.5.0] - 2026-08-19

### Added
- **System Installation Date Module (`Installed`)**: Probes root filesystem creation timestamp (`stx_btime`) and distribution installer records, formatting as intuitive `DD Mon YYYY, hh:mm AM/PM (X days ago)` (e.g. `Installed: 16 Aug 2026, 02:32 PM (3 days ago)`). Suggested by [@Laynsb](https://github.com/Laynsb).
- **Universal Terminal Detection Expansion**: Added native detection signatures and version resolution for **Ptyxis** (`$PTYXIS_VERSION`), **Ghostty** (`$GHOSTTY_VERSION`), **GNOME Console** (`kgx`), **BlackBox**, **Contour**, **Rio**, **Yakuake**, **Guake**, **LXTerminal**, **MATE Terminal**, **QTerminal**, **Deepin Terminal**, **Pantheon Terminal**, **Warp**, and **Zellij**.
- **Desktop Environment Version Resolution**: Appends detected DE versions from metadata files and version queries (e.g. `GNOME 50.1`, `KDE Plasma 6.1`, `XFCE 4.18`, `MATE 1.28`, `Cinnamon 6.0`).
- **Intel iGPU & Linux GPU Clock Speed**: Probes maximum graphics clock frequency from `/sys/class/drm/card*/gt_max_freq_mhz` and hwmon sysfs (e.g. `GPU0: Intel HD Graphics 620 @ 1.000GHz`).
- **Wayland Display Refresh Rate**: Added native refresh rate resolution for Wayland compositors via `wlr-randr` and DRM sysfs.

## [0.4.2] - 2026-08-19

### Fixed
- **Android / Termux Disk Filtering**: Filtered out Android internal loop mounts and read-only system subsystems (`/apex`, `/bootstrap-apex`, `/data/app`, `/data/user`, `/metadata`, `/product`, `/vendor`, `/system`). Only actual user storage partitions (`/`, `/data`, `/storage/*`) are listed in Termux.

## [0.4.1] - 2026-08-19

### Changed
- **Clean Battery Formatting**: Streamlined battery output across WSL and native Linux to display standard metrics and AC connection state (e.g. `Battery: 97% [AC Connected]`) without redundant hypervisor model strings.

## [0.4.0] - 2026-08-19

### Added
- **OS Architecture**: Displays system architecture alongside distribution name (e.g. `OS: Ubuntu 24.04.4 LTS x86_64`).
- **Shell Version**: Resolves active shell version (e.g. `Shell: zsh 5.9`, `Shell: bash 5.2.21`).
- **Display Resolution & Refresh Rate**: Probes connected displays and refresh rates (e.g. `Display: 1920x1080 @ 60Hz`).
- **Window Manager Module (`WM`)**: Resolves active window managers including Mutter, KWin, Xfwm4, Sway, Hyprland, and `WSLg (Weston)`.
- **GPU VRAM & Clock Frequency**: Displays GPU memory capacity and max graphics clock (e.g. `GPU0: Intel Iris Xe Graphics (1 GiB) @ 1.400GHz`, `GPU1: NVIDIA GeForce RTX 4090 (24 GiB) @ 2.520GHz`).
- **Swap Memory Module**: Displays total and active used swap partition/file memory.
- **Partition Disk Enumeration**: Discovers all active physical and virtual partitions labeled sequentially (`Disk0`, `Disk1`, `Disk2`), formatting WSL Windows drives directly (e.g. `(C)`, `(D)`).
- **Physical Battery Detection**: Probes battery percentage and status from sysfs while automatically filtering out Microsoft Hyper-V virtual batteries in WSL.
- **Local IP Module**: Probes primary local IPv4 address via standard POSIX interface enumeration without subprocesses.

## [0.3.0] - 2026-08-19

### Added
- **Multi-Socket CPU Scaling**: Formats multi-socket CPU systems as `<n>x <CPU Name> (<Total Threads>)` (e.g. `3x AMD EPYC 9654 (384)`).
- **CPU Clock Speed**: Added frequency resolution (`@ nGHz`) from `/proc/cpuinfo` and `cpufreq` sysfs.
- **Dynamic Sequential GPU Indexing**: Assigns sequential indices (`GPU0`, `GPU1`, `GPU2`, ...) without skipping numbers.
- **iGPU `GPU0` Priority**: Integrated graphics always occupy `GPU0` and scale across multi-socket systems (`GPU0: <n>x <iGPU Name>`).
- **dGPU Automatic Grouping**: Automatically groups identical discrete GPUs into a single line (`GPU<index>: <n>x <dGPU Name>`).
- **Sub-30ms WSL GPU Caching**: Persistent caching for discrete GPU queries in WSL2, reducing execution time from ~1.7s to under 30ms.

### Changed
- Stripped redundant integrated graphics marketing strings (`with Radeon Graphics`, `with Intel UHD Graphics`) from CPU model lines for a cleaner terminal silhouette.

## [0.2.5] - 2026-08-19

### Added
- Native WSL2 hybrid GPU detection (`GpuCollector`) resolving both integrated graphics (e.g. `Intel Iris Xe` / `AMD Radeon Graphics`) and discrete NVIDIA graphics (e.g. `NVIDIA GeForce RTX 4080` / `RTX 4090`) via the native Windows driver bridge without extra Linux drivers.

## [0.2.0] - 2026-08-19

### Added
- `--json` CLI flag for structured JSON output across all enabled modules without external dependencies.
- Native `pci.ids` database parser resolving PCI vendor/device hex pairs to human-readable graphics cards without spawning subprocesses.
- WSL2 hypervisor and host motherboard model identification in `HostCollector`.
- Package manager counting support for **Homebrew** (`Cellar`) and **Gentoo** (`/var/db/pkg`).
- Foreground solid color block glyphs (`███`) in `ColorsCollector` for consistent light/dark terminal rendering.
- Built-in ASCII logos for **NixOS**, **Kali Linux**, **FreeBSD**, **Slackware**, **Artix Linux**, and **Zorin OS**.

### Fixed
- Fixed GPU detection prioritizing motherboard ACPI DMI slot labels (e.g. `Onboard - Video` on ASUS/Dell laptops) over actual graphics processor model names.

## [0.1.0] - 2026-08-16

### Added

- Core system information fetch engine implemented in Rust without spawning shell subprocesses.
- System metrics collectors:
  - **Title**: Username and hostname resolution via environment variables and `getpwuid`/`uname`.
  - **OS**: Linux distribution identification via `/etc/os-release` and `/usr/lib/os-release`.
  - **Host**: Hardware model and chassis parsing from `/sys/devices/virtual/dmi/id/` and device-tree.
  - **Kernel**: Release and architecture parsing from POSIX `libc::uname`.
  - **Uptime**: Accurate uptime calculation from `/proc/uptime` and `libc::sysinfo`.
  - **Packages**: Direct file-based package counting for Debian (`dpkg/status`), Arch (`pacman/local`), Red Hat (RPM database), Alpine (`apk`), Flatpak, and Snap.
  - **Shell**: Current shell process detection and version extraction via `/proc/<pid>/status` and `$SHELL`.
  - **Terminal**: Active terminal emulator detection via environment variables (`TERM_PROGRAM`, Alacritty, Kitty, Konsole, Foot) and process tree walking.
  - **Desktop / WM**: Desktop environment and window manager detection via `$XDG_CURRENT_DESKTOP`, Wayland socket signatures, and process scans.
  - **CPU**: Multi-socket, core count, and model name parsing from `/proc/cpuinfo` with vendor string sanitization.
  - **GPU**: Direct PCI sysfs scan (`/sys/bus/pci/devices`) with vendor ID mapping and fallback detection.
  - **Memory**: Accurate memory consumption calculation (`MemTotal - MemAvailable`) from `/proc/meminfo`.
  - **Disk**: Mount point capacity and utilization querying using POSIX `libc::statvfs`.
  - **Colors**: 8-color terminal palette block rendering.
- Layout and rendering engine:
  - Dynamic side-by-side logo and metric alignment.
  - ANSI escape code stripping for accurate visible character width calculation.
  - Automatic vertical layout fallback for narrow terminal displays (< 60 columns).
  - Terminal color auto-detection respecting `NO_COLOR`, `CLICOLOR_FORCE`, and non-TTY stdout redirection.
- Built-in ASCII logos:
  - Ferris the Rust mascot.
  - Distribution art for Arch, Debian, Ubuntu, Linux Mint, Fedora, RHEL, Rocky Linux, AlmaLinux, EndeavourOS, Manjaro, openSUSE, Alpine, Gentoo, Void Linux, Pop!_OS, and generic Tux.
- Command-line interface (`clap` derive):
  - `-m, --modules`: Module selection and ordering.
  - `-d, --disable`: Selective module disabling.
  - `-l, --logo`: ASCII logo override by distribution name or alias.
  - `--no-logo`: Logo suppression.
  - `--no-color`: ANSI color disabling.
  - `--disk-path`: Target path selection for disk metrics.
  - `--list-modules`: Available module enumeration.
- Comprehensive test suite:
  - Unit tests covering parsing logic across edge cases and malformed files.
  - Integration tests with synthetic procfs and sysfs fixtures for 15+ Linux distributions.
  - CLI flag combination tests using `assert_cmd`.
- Packaging and CI infrastructure:
  - GitHub Actions CI workflow for formatting, clippy, unit/integration testing, and release builds.
  - Release workflow building standalone GNU and Musl binaries, Debian (`.deb`), Red Hat (`.rpm`), Arch Linux (`.pkg.tar.zst`), and Android / Termux (`.deb` & ARM64 binary) packages with SHA256 checksums.
  - Arch Linux `PKGBUILD` and Debian packaging specifications.
