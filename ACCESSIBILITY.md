# Accessibility Statement

KKFetch is committed to ensuring digital accessibility for all users, including individuals with disabilities, users navigating via screen readers or braille displays, and those operating in high-contrast or specialized terminal environments.

---

## Technical Standards & Features

As a terminal-based system information CLI written in Rust, KKFetch adheres to the following accessibility principles:

### 1. Color Suppression (`NO_COLOR` & `--no-color`)
KKFetch adheres strictly to the [`NO_COLOR`](https://no-color.org/) standard. When the `NO_COLOR` environment variable is defined (regardless of its value) or when invoked with the `--no-color` flag, all ANSI color codes and escape styling sequences are suppressed. This guarantees that:
* Screen readers and braille terminals do not encounter or announce raw ANSI escape sequences.
* Users relying on personalized terminal color palettes or monochrome displays receive clean, unformatted text.

### 2. Screen Reader & Non-Visual Navigation
* **Linear Key-Value Output**: System metrics are rendered in a structured, top-to-bottom key-value format readable by screen readers (e.g. Orca on Linux, VoiceOver on macOS, NVDA/JAWS in Windows Terminal) without column confusion.
* **Logo Suppression (`--no-logo`)**: Running `kkfetch --no-logo` suppresses the ASCII distribution artwork entirely, emitting a single clean stream of system metrics.
* **Structured JSON Export (`--json`)**: Running `kkfetch --json` outputs pure, unstyled machine-readable JSON, ideal for assistive software or custom audio/braille rendering pipelines.
* **Compact Layouts (`--compact-logo`)**: Provides reduced-width ASCII artwork to prevent line wrapping on low-resolution displays or braille terminals.

### 3. Keyboard-Only & Assistive Navigation
* Operating entirely within standard terminal emulators, KKFetch requires zero pointer/mouse interaction and functions fully via standard shell keyboard workflows.

### 4. Semantic Documentation
* Project documentation, manual pages, and help outputs maintain logical heading hierarchies, descriptive text, and table formatting compatible with accessibility tools.

---

## Known Limitations

* **ASCII Art Logos**: Default multi-color ASCII distribution logos consist of non-semantic character art that may cause verbosity with screen readers if not suppressed. Users utilizing screen readers are encouraged to run KKFetch with logo suppression or pipe into text processors.

---

## Reporting Accessibility Barriers

If you encounter accessibility issues, unexpected terminal escape behavior, or barriers using assistive technologies with KKFetch, please let us know:

* **GitHub Issues**: [Open an issue on GitHub](https://github.com/kk376/kkfetch/issues/new) using the `accessibility` label.
* **Direct Contact**: Email the maintainer at [kkushagra86@gmail.com](mailto:kkushagra86@gmail.com).

Please include details about:
1. The terminal emulator and operating system version.
2. Any assistive technology used (e.g. Orca, VoiceOver, NVDA).
3. The exact command line invoked and the unexpected output.

All reports will be reviewed promptly to improve accessibility across supported environments.
