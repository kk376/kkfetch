use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;

const KNOWN_TERMINALS: &[(&str, &str)] = &[
    ("ptyxis-agent", "Ptyxis"),
    ("ptyxis", "Ptyxis"),
    ("gnome-terminal-server", "GNOME Terminal"),
    ("gnome-terminal", "GNOME Terminal"),
    ("gnome-console", "GNOME Console"),
    ("kgx", "GNOME Console"),
    ("konsole", "Konsole"),
    ("alacritty", "Alacritty"),
    ("kitty", "kitty"),
    ("ghostty", "Ghostty"),
    ("wezterm-gui", "WezTerm"),
    ("wezterm", "WezTerm"),
    ("foot", "foot"),
    ("rio", "Rio"),
    ("contour", "Contour"),
    ("blackbox", "BlackBox"),
    ("xterm", "xterm"),
    ("urxvt", "urxvt"),
    ("rxvt", "rxvt"),
    ("st", "st"),
    ("terminator", "Terminator"),
    ("xfce4-terminal", "XFCE Terminal"),
    ("mate-terminal", "MATE Terminal"),
    ("lxterminal", "LXTerminal"),
    ("qterminal", "QTerminal"),
    ("tilix", "Tilix"),
    ("guake", "Guake"),
    ("yakuake", "Yakuake"),
    ("tilda", "Tilda"),
    ("sakura", "Sakura"),
    ("termite", "Termite"),
    ("tabby", "Tabby"),
    ("hyper", "Hyper"),
    ("warp", "Warp"),
    ("deepin-terminal", "Deepin Terminal"),
    ("pantheon-terminal", "Pantheon Terminal"),
    ("io.elementary.terminal", "Pantheon Terminal"),
    ("zellij", "Zellij"),
    ("tmux", "tmux"),
];

/// Inspects environment variables to detect terminal emulator.
pub fn detect_terminal_from_env(
    term_program: Option<&str>,
    term_program_version: Option<&str>,
    env_vars: &[(&str, &str)],
    term: Option<&str>,
) -> Option<String> {
    // 1. Check dedicated terminal environment signatures first
    let has_env = |var_name: &str| env_vars.iter().any(|&(k, _)| k == var_name);
    let get_env_val = |var_name: &str| {
        env_vars
            .iter()
            .find(|&&(k, _)| k == var_name)
            .map(|&(_, v)| v.trim())
    };

    if has_env("VSCODE_INJECTION") {
        return Some("Visual Studio Code".to_string());
    }

    if let Some(ver) = get_env_val("PTYXIS_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Ptyxis {}", ver));
        }
        return Some("Ptyxis".to_string());
    }

    if let Some(ver) = get_env_val("GHOSTTY_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Ghostty {}", ver));
        }
        return Some("Ghostty".to_string());
    }
    if has_env("GHOSTTY_RESOURCES_DIR") {
        return Some("Ghostty".to_string());
    }

    if let Some(ver) = get_env_val("KGX_VERSION") {
        if !ver.is_empty() {
            return Some(format!("GNOME Console {}", ver));
        }
        return Some("GNOME Console".to_string());
    }

    if has_env("ALACRITTY_LOG") || has_env("ALACRITTY_WINDOW_ID") || has_env("ALACRITTY_SOCKET") {
        return Some("Alacritty".to_string());
    }

    if has_env("KITTY_PID") || has_env("KITTY_WINDOW_ID") {
        return Some("kitty".to_string());
    }

    if let Some(ver) = get_env_val("KONSOLE_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Konsole {}", ver));
        }
        return Some("Konsole".to_string());
    }

    if has_env("WT_SESSION") {
        return Some("Windows Terminal".to_string());
    }

    if let Some(ver) = get_env_val("CONTOUR_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Contour {}", ver));
        }
        return Some("Contour".to_string());
    }

    if let Some(ver) = get_env_val("RIO_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Rio {}", ver));
        }
        return Some("Rio".to_string());
    }

    if let Some(ver) = get_env_val("BLACKBOX_VERSION") {
        if !ver.is_empty() {
            return Some(format!("BlackBox {}", ver));
        }
        return Some("BlackBox".to_string());
    }

    if has_env("TERMINOLOGY") {
        return Some("Terminology".to_string());
    }

    if let Some(ver) = get_env_val("XTERM_VERSION") {
        if !ver.is_empty() {
            return Some(format!("xterm {}", ver));
        }
        return Some("xterm".to_string());
    }

    if has_env("GNOME_TERMINAL_SCREEN") || has_env("GNOME_TERMINAL_SERVICE") {
        return Some("GNOME Terminal".to_string());
    }

    if has_env("MATE_TERMINAL_SCREEN") {
        return Some("MATE Terminal".to_string());
    }

    if has_env("TILIX_ID") {
        return Some("Tilix".to_string());
    }

    if has_env("WEZTERM_PANE") {
        return Some("WezTerm".to_string());
    }

    if has_env("WARP_IS_LOCAL_SHELL_SESSION") {
        return Some("Warp".to_string());
    }

    if has_env("FOOT_PID") {
        return Some("foot".to_string());
    }

    // 2. Check $TERM_PROGRAM for non-multiplexers
    if let Some(prog) = term_program {
        let clean_prog = prog.trim();
        let is_multiplexer = clean_prog.eq_ignore_ascii_case("tmux")
            || clean_prog.eq_ignore_ascii_case("screen")
            || clean_prog.eq_ignore_ascii_case("zellij");

        if !is_multiplexer {
            if clean_prog.eq_ignore_ascii_case("vscode") || clean_prog == "Code" {
                if let Some(ver) = term_program_version {
                    let clean_ver = ver.trim();
                    if !clean_ver.is_empty() {
                        return Some(format!("Visual Studio Code {}", clean_ver));
                    }
                }
                return Some("Visual Studio Code".to_string());
            } else if !clean_prog.is_empty() {
                if let Some(ver) = term_program_version {
                    let clean_ver = ver.trim();
                    if !clean_ver.is_empty() {
                        return Some(format!("{} {}", clean_prog, clean_ver));
                    }
                }
                return Some(clean_prog.to_string());
            }
        }
    }

    // 3. Fallback to $TERM
    if let Some(t) = term {
        let clean = t.trim();
        if !clean.is_empty() && clean != "unknown" && clean != "dumb" {
            return Some(clean.to_string());
        }
    }

    None
}

pub fn match_terminal_proc(comm: &str) -> Option<&'static str> {
    for &(proc_name, display_name) in KNOWN_TERMINALS {
        let is_match = if proc_name == "st" {
            comm == "st" || comm == "stterm" || comm.starts_with("st-")
        } else {
            comm == proc_name
                || comm
                    .strip_prefix(proc_name)
                    .is_some_and(|rest| rest.starts_with('-'))
        };
        if is_match {
            return Some(display_name);
        }
    }
    None
}

/// Pure helper to detect terminal on Windows from environment variables.
pub fn detect_windows_terminal_from_env(
    term_program: Option<&str>,
    term_program_version: Option<&str>,
    env_vars: &[(&str, &str)],
) -> String {
    detect_terminal_from_env(term_program, term_program_version, env_vars, None)
        .unwrap_or_else(|| "Console Window Host (ConHost)".to_string())
}

/// Inspects environment variables and process ancestry to detect terminal emulator.
#[cfg(windows)]
pub fn detect_terminal() -> Option<String> {
    use crate::modules::win_util::ffi;

    let term_prog = std::env::var("TERM_PROGRAM").ok();
    let term_prog_ver = std::env::var("TERM_PROGRAM_VERSION").ok();

    let env_signatures = [
        "WT_SESSION",
        "VSCODE_INJECTION",
        "ALACRITTY_LOG",
        "ALACRITTY_WINDOW_ID",
        "ALACRITTY_SOCKET",
        "WEZTERM_PANE",
        "KONSOLE_VERSION",
        "GHOSTTY_VERSION",
    ];

    let mut present_vars = Vec::new();
    for &sig in &env_signatures {
        if let Ok(val) = std::env::var(sig) {
            present_vars.push((sig, val));
        }
    }
    let ref_vars: Vec<(&str, &str)> = present_vars.iter().map(|(k, v)| (*k, v.as_str())).collect();

    if let Some(term) = detect_terminal_from_env(
        term_prog.as_deref(),
        term_prog_ver.as_deref(),
        &ref_vars,
        None,
    ) {
        return Some(term);
    }

    // Check parent process chain for terminal emulator
    let chain = ffi::get_parent_process_chain(5);
    for (_pid, name) in &chain {
        let lower = name.to_lowercase();
        if lower.contains("windowsterminal") {
            return Some("Windows Terminal".to_string());
        }
        if lower.contains("alacritty") {
            return Some("Alacritty".to_string());
        }
        if lower.contains("wezterm") {
            return Some("WezTerm".to_string());
        }
        if lower.contains("code") {
            return Some("Visual Studio Code".to_string());
        }
        if lower.contains("mintty") {
            return Some("MinTTY".to_string());
        }
    }

    Some("Console Window Host (ConHost)".to_string())
}

/// Extracts the first version-like token from terminal version stdout/stderr.
pub fn extract_terminal_version_from_output(output: &str) -> Option<String> {
    for line in output.lines() {
        for word in line.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_');
            if let Some(first) = clean.chars().next() {
                if first.is_ascii_digit() && (clean.contains('.') || clean.len() >= 6) {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

/// Appends version to terminal name if not already present.
pub fn append_version_if_missing(term_display_name: &str, version: Option<&str>) -> String {
    let Some(ver) = version else {
        return term_display_name.to_string();
    };
    let clean_ver = ver.trim();
    if clean_ver.is_empty() {
        return term_display_name.to_string();
    }
    if term_display_name.contains(clean_ver) {
        return term_display_name.to_string();
    }
    format!("{} {}", term_display_name, clean_ver)
}

#[cfg(not(windows))]
fn get_terminal_cache_path(binary: &str) -> std::path::PathBuf {
    // 1. Prefer $XDG_CACHE_HOME or ~/.cache/kkfetch/ for persistent caching across boots
    if let Ok(cache_home) = std::env::var("XDG_CACHE_HOME") {
        let dir = std::path::PathBuf::from(cache_home).join("kkfetch");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join(format!("term_{}.cache", binary));
    }
    if let Ok(home) = std::env::var("HOME") {
        let dir = std::path::PathBuf::from(home)
            .join(".cache")
            .join("kkfetch");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join(format!("term_{}.cache", binary));
    }
    // 2. Fallback to $XDG_RUNTIME_DIR
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = std::path::Path::new(&runtime_dir);
        if dir.is_dir() {
            return dir.join(format!("kkfetch_term_{}.cache", binary));
        }
    }
    // 3. Fallback to private user-isolated temporary directory (mode 0700 on Unix)
    #[cfg(unix)]
    let temp_dir = {
        // SAFETY: getuid() is a POSIX syscall that returns the real user ID of the calling process without side effects.
        let uid = unsafe { libc::getuid() };
        let dir = std::env::temp_dir().join(format!("kkfetch-{}", uid));
        let _ = std::fs::create_dir_all(&dir);
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        dir
    };
    #[cfg(not(unix))]
    let temp_dir = {
        let dir = std::env::temp_dir().join("kkfetch");
        let _ = std::fs::create_dir_all(&dir);
        dir
    };
    temp_dir.join(format!("term_{}.cache", binary))
}

#[cfg(not(windows))]
fn get_binary_mtime(binary: &str) -> Option<u64> {
    let standard_paths = [
        format!("/usr/bin/{}", binary),
        format!("/usr/local/bin/{}", binary),
        format!("/bin/{}", binary),
    ];

    for p in &standard_paths {
        let path = std::path::Path::new(p);
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                    return Some(dur.as_secs());
                }
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let local_bin = std::path::Path::new(&home).join(format!(".local/bin/{}", binary));
        if let Ok(meta) = fs::metadata(&local_bin) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                    return Some(dur.as_secs());
                }
            }
        }
    }

    None
}

#[cfg(not(windows))]
pub fn probe_terminal_cli_version(term_name: &str) -> Option<String> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, Option<String>>>,
    > = std::sync::OnceLock::new();
    let cache_mutex = CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));

    if let Ok(guard) = cache_mutex.lock() {
        if let Some(cached) = guard.get(term_name) {
            return cached.clone();
        }
    }

    let result = probe_terminal_cli_version_uncached(term_name);

    if let Ok(mut guard) = cache_mutex.lock() {
        guard.insert(term_name.to_string(), result.clone());
    }

    result
}

#[cfg(not(windows))]
fn probe_terminal_cli_version_uncached(term_name: &str) -> Option<String> {
    let lower = term_name.to_lowercase();
    let (binary, args): (&str, &[&str]) = if lower.contains("kitty") {
        ("kitty", &["--version"])
    } else if lower.contains("alacritty") {
        ("alacritty", &["--version"])
    } else if lower.contains("foot") {
        ("foot", &["--version"])
    } else if lower.contains("wezterm") {
        ("wezterm", &["--version"])
    } else if lower.contains("ghostty") {
        ("ghostty", &["--version"])
    } else if lower.contains("gnome-terminal") || lower == "gnome terminal" {
        ("gnome-terminal", &["--version"])
    } else if lower.contains("gnome-console") || lower == "gnome console" || lower == "kgx" {
        ("kgx", &["--version"])
    } else if lower.contains("konsole") {
        ("konsole", &["--version"])
    } else if lower.contains("xfce4-terminal") || lower == "xfce terminal" {
        ("xfce4-terminal", &["--version"])
    } else if lower.contains("mate-terminal") || lower == "mate terminal" {
        ("mate-terminal", &["--version"])
    } else if lower.contains("tilix") {
        ("tilix", &["--version"])
    } else if lower.contains("terminator") {
        ("terminator", &["--version"])
    } else if lower == "tmux" {
        ("tmux", &["-V"])
    } else if lower == "zellij" {
        ("zellij", &["--version"])
    } else if lower == "rio" {
        ("rio", &["--version"])
    } else if lower == "contour" {
        ("contour", &["--version"])
    } else if lower == "blackbox" {
        ("blackbox", &["--version"])
    } else if lower == "ptyxis" {
        ("ptyxis", &["--version"])
    } else if lower == "xterm" {
        ("xterm", &["-version"])
    } else {
        return None;
    };

    let cache_path = get_terminal_cache_path(binary);
    let bin_mtime = get_binary_mtime(binary);

    // 1. Try reading from mtime-validated tmpfs runtime cache (< 2 µs)
    if let Some(current_mtime) = bin_mtime {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Some((cached_mtime_str, ver)) = content.split_once('|') {
                if let Ok(cached_mtime) = cached_mtime_str.parse::<u64>() {
                    if cached_mtime == current_mtime && !ver.trim().is_empty() {
                        return Some(ver.trim().to_string());
                    }
                }
            }
        }
    }

    // 2. Cache miss: execute subprocess once and persist to tmpfs
    if let Ok(output) = crate::modules::system_command(binary).args(args).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let ver = if let Some(v) = extract_terminal_version_from_output(&stdout) {
            Some(v)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            extract_terminal_version_from_output(&stderr)
        };

        if let Some(ref version_str) = ver {
            if let Some(current_mtime) = bin_mtime {
                let _ = fs::write(&cache_path, format!("{}|{}", current_mtime, version_str));
            }
        }

        return ver;
    }
    None
}

#[cfg(not(windows))]
fn find_multiplexer_outer_terminal() -> Option<String> {
    let entries = fs::read_dir("/proc").ok()?;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name_bytes = file_name.as_encoded_bytes();
        if name_bytes.is_empty() || !name_bytes.iter().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let pid_str = match file_name.to_str() {
            Some(s) => s,
            None => continue,
        };
        let comm_path = format!("/proc/{}/comm", pid_str);
        let comm = match fs::read_to_string(&comm_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let comm_trimmed = comm.trim().to_lowercase();
        if !comm_trimmed.starts_with("tmux")
            && !comm_trimmed.starts_with("screen")
            && !comm_trimmed.starts_with("zellij")
        {
            continue;
        }

        let status_path = format!("/proc/{}/status", pid_str);
        let ppid = match fs::read_to_string(&status_path) {
            Ok(status) => status.lines().find_map(|l| {
                l.strip_prefix("PPid:")
                    .and_then(|p| p.trim().parse::<u32>().ok())
            }),
            Err(_) => continue,
        };

        let mut curr_ppid = match ppid {
            Some(p) if p > 1 => p,
            _ => continue,
        };

        for _ in 0..5 {
            if curr_ppid <= 1 {
                break;
            }
            let parent_comm = fs::read_to_string(format!("/proc/{}/comm", curr_ppid))
                .unwrap_or_default()
                .trim()
                .to_lowercase();

            if !parent_comm.is_empty()
                && !parent_comm.starts_with("tmux")
                && !parent_comm.starts_with("screen")
                && !parent_comm.starts_with("zellij")
            {
                if let Some(display_name) = match_terminal_proc(&parent_comm) {
                    let ver = probe_terminal_cli_version(display_name);
                    return Some(append_version_if_missing(display_name, ver.as_deref()));
                }
            }

            let next_ppid = fs::read_to_string(format!("/proc/{}/status", curr_ppid))
                .ok()
                .and_then(|s| {
                    s.lines().find_map(|l| {
                        l.strip_prefix("PPid:")
                            .and_then(|p| p.trim().parse::<u32>().ok())
                    })
                });

            match next_ppid {
                Some(p) if p > 1 => curr_ppid = p,
                _ => break,
            }
        }
    }
    None
}

/// Inspects environment variables and process ancestry to detect terminal emulator.
#[cfg(not(windows))]
pub fn detect_terminal() -> Option<String> {
    let term_prog = std::env::var("TERM_PROGRAM").ok();
    let term_prog_ver = std::env::var("TERM_PROGRAM_VERSION").ok();
    let term_val = std::env::var("TERM").ok();

    // 1. Check dedicated terminal emulator environment signatures
    let env_signatures = [
        "ALACRITTY_LOG",
        "ALACRITTY_WINDOW_ID",
        "ALACRITTY_SOCKET",
        "KITTY_PID",
        "KITTY_WINDOW_ID",
        "KONSOLE_VERSION",
        "WT_SESSION",
        "VSCODE_INJECTION",
        "FOOT_PID",
        "TERMINOLOGY",
        "XTERM_VERSION",
        "GNOME_TERMINAL_SCREEN",
        "GNOME_TERMINAL_SERVICE",
        "TILIX_ID",
        "WEZTERM_PANE",
        "PTYXIS_VERSION",
        "GHOSTTY_VERSION",
        "GHOSTTY_RESOURCES_DIR",
    ];

    let mut present_vars = Vec::new();
    for &sig in &env_signatures {
        if let Ok(val) = std::env::var(sig) {
            present_vars.push((sig, val));
        }
    }
    let ref_vars: Vec<(&str, &str)> = present_vars.iter().map(|(k, v)| (*k, v.as_str())).collect();

    if let Some(term) = detect_terminal_from_env(
        term_prog.as_deref(),
        term_prog_ver.as_deref(),
        &ref_vars,
        None, // Defer generic $TERM fallback until process ancestry is checked
    ) {
        let ver = probe_terminal_cli_version(&term);
        return Some(append_version_if_missing(&term, ver.as_deref()));
    }

    // 2. Process ancestry traversal: walk up to 8 levels of PPID to jump over subshells, tmux/screen, and sudo wrappers
    // SAFETY: getpid() is a standard POSIX syscall that returns the current process ID without memory safety implications.
    let mut current_pid = unsafe { libc::getpid() as u32 };
    let mut seen_multiplexer = false;
    for _ in 0..8 {
        let status_path = format!("/proc/{}/status", current_pid);
        let ppid = if let Ok(status) = fs::read_to_string(status_path) {
            status.lines().find_map(|l| {
                l.strip_prefix("PPid:")
                    .and_then(|p| p.trim().parse::<u32>().ok())
            })
        } else {
            None
        };

        if let Some(ppid) = ppid {
            if ppid <= 1 {
                break;
            }

            let comm = fs::read_to_string(format!("/proc/{}/comm", ppid))
                .unwrap_or_default()
                .trim()
                .to_lowercase();

            let is_multiplexer = comm == "tmux"
                || comm.starts_with("tmux:")
                || comm == "screen"
                || comm.starts_with("screen-")
                || comm == "zellij";

            if is_multiplexer {
                seen_multiplexer = true;
                current_pid = ppid;
                continue;
            }

            if let Some(display_name) = match_terminal_proc(&comm) {
                let ver = probe_terminal_cli_version(display_name);
                return Some(append_version_if_missing(display_name, ver.as_deref()));
            }

            current_pid = ppid;
        } else {
            break;
        }
    }

    // 3. If running inside a multiplexer, search /proc for outer GUI terminal emulator
    if seen_multiplexer
        || std::env::var_os("TMUX").is_some()
        || std::env::var_os("STY").is_some()
        || std::env::var_os("ZELLIJ").is_some()
        || std::env::var_os("ZELLIJ_SESSION_NAME").is_some()
        || term_prog.as_deref().is_some_and(|p| {
            let s = p.trim();
            s.eq_ignore_ascii_case("tmux")
                || s.eq_ignore_ascii_case("screen")
                || s.eq_ignore_ascii_case("zellij")
        })
    {
        if let Some(outer) = find_multiplexer_outer_terminal() {
            return Some(outer);
        }
    }

    // 4. Fallback to multiplexer name from $TERM_PROGRAM if outer terminal was not found
    if let Some(prog) = term_prog {
        let clean = prog.trim();
        if !clean.is_empty() {
            if let Some(ver) = term_prog_ver {
                let clean_ver = ver.trim();
                if !clean_ver.is_empty() {
                    return Some(format!("{} {}", clean, clean_ver));
                }
            }
            return Some(clean.to_string());
        }
    }

    // 5. Fallback to generic $TERM string if specific GUI terminal binary was not found
    if let Some(term) = term_val {
        let clean = term.trim();
        if !clean.is_empty() && clean != "unknown" && clean != "dumb" {
            return Some(clean.to_string());
        }
    }

    None
}

pub struct TerminalCollector;

impl Collector for TerminalCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Terminal
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let term = detect_terminal()?;
        Some(ModuleOutput {
            id: ModuleId::Terminal,
            label: "Terminal".to_string(),
            value: term,
            custom_rendered: None,
        })
    }
}

/// Formats a numeric font size string, trimming unnecessary trailing zeros (e.g. '12.0' -> '12', '12.5' -> '12.5').
pub fn format_size_num(raw: &str) -> String {
    let clean = raw.trim_end_matches("pt").trim_end_matches("px").trim();
    if let Ok(f) = clean.parse::<f32>() {
        if f.fract().abs() < f32::EPSILON {
            return format!("{:.0}", f);
        }
        return format!("{}", f);
    }
    clean.to_string()
}

#[cfg(any(not(windows), test))]
fn parse_ghostty_font(content: &str) -> Option<String> {
    let mut families = Vec::new();
    let mut size = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"').trim_matches('\'');
            if k == "font-family" && !v.is_empty() {
                families.push(v.to_string());
            } else if k == "font-size" && size.is_none() && !v.is_empty() {
                size = Some(v.to_string());
            }
        }
    }
    if !families.is_empty() {
        let f = families.join(", ");
        if let Some(s) = size {
            return Some(format!("{} ({}pt)", f, format_size_num(&s)));
        }
        return Some(f);
    } else if let Some(s) = size {
        return Some(format!("{}pt", format_size_num(&s)));
    }
    None
}

#[cfg(not(windows))]
fn probe_ghostty_font(home_path: &std::path::Path) -> Option<String> {
    for path in &[
        home_path.join(".config/ghostty/config"),
        home_path.join(".config/ghostty/config.ghostty"),
    ] {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some(res) = parse_ghostty_font(&content) {
                return Some(res);
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn probe_kitty_font(home_path: &std::path::Path) -> Option<String> {
    let kitty_conf = home_path.join(".config/kitty/kitty.conf");
    let mut family = None;
    let mut size = None;
    if let Ok(content) = fs::read_to_string(kitty_conf) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("font_family") {
                let f = rest.trim();
                if !f.is_empty() && f != "auto" {
                    family = Some(f.to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("font_size") {
                let s = rest.trim();
                if !s.is_empty() {
                    size = Some(s.to_string());
                }
            }
        }
    }

    let fam = family.unwrap_or_else(|| "monospace".to_string());
    let sz = size
        .map(|s| format_size_num(&s))
        .unwrap_or_else(|| "11".to_string());

    Some(format!("{} ({}pt)", fam, sz))
}

#[cfg(not(windows))]
fn probe_alacritty_font(home_path: &std::path::Path) -> Option<String> {
    let mut family = None;
    let mut size = None;

    for path in &[
        home_path.join(".config/alacritty/alacritty.toml"),
        home_path.join(".alacritty.toml"),
        home_path.join(".config/alacritty/alacritty.yml"),
        home_path.join(".alacritty.yml"),
    ] {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    continue;
                }
                let split = trimmed.split_once('=').or_else(|| trimmed.split_once(':'));
                if let Some((k, v)) = split {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'').trim();
                    if (k == "family" || k == "normal.family") && family.is_none() && !v.is_empty()
                    {
                        family = Some(v.to_string());
                    } else if (k == "size" || k == "font.size") && size.is_none() && !v.is_empty() {
                        size = Some(v.to_string());
                    }
                }
            }
            if family.is_some() || size.is_some() {
                break;
            }
        }
    }

    let fam = family.unwrap_or_else(|| "monospace".to_string());
    let sz = size
        .map(|s| format_size_num(&s))
        .unwrap_or_else(|| "11.25".to_string());

    Some(format!("{} ({}pt)", fam, sz))
}

/// Parses Fontconfig pattern syntax (e.g. 'Fira Code:size=12, Symbols Nerd Font Mono:size=12' or 'DejaVu-10').
pub fn parse_fontconfig_pattern(raw: &str) -> Option<String> {
    let clean = raw.trim().trim_matches('\'').trim_matches('"').trim();
    if clean.is_empty() {
        return None;
    }

    let mut families: Vec<String> = Vec::new();
    let mut detected_size: Option<String> = None;

    for entry in clean.split(',') {
        let entry_trimmed = entry.trim();
        if entry_trimmed.is_empty() {
            continue;
        }

        let mut parts = entry_trimmed.split(':');
        let head = parts.next().unwrap_or("").trim();
        let unescaped_head = head.replace(r"\ ", " ");

        let mut entry_family = unescaped_head.as_str();
        if let Some(dash_idx) = entry_family.rfind('-') {
            let potential_size = &entry_family[dash_idx + 1..];
            if !potential_size.is_empty()
                && potential_size
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.')
                && potential_size.chars().any(|c| c.is_ascii_digit())
            {
                if detected_size.is_none() {
                    let formatted_size = format_size_num(potential_size);
                    detected_size = Some(format!("{}pt", formatted_size));
                }
                entry_family = entry_family[..dash_idx].trim();
            }
        }

        let clean_family = entry_family.trim_matches('\'').trim_matches('"').trim();
        if !clean_family.is_empty() && !families.iter().any(|f| f == clean_family) {
            families.push(clean_family.to_string());
        }

        for prop in parts {
            let prop_trimmed = prop.trim();
            if let Some((k, v)) = prop_trimmed.split_once('=') {
                let k_lower = k.trim().to_ascii_lowercase();
                let v_clean = v.trim().trim_matches('\'').trim_matches('"').trim();
                if detected_size.is_none() {
                    if (k_lower == "size" || k_lower == "pointsize") && !v_clean.is_empty() {
                        let formatted_size = format_size_num(v_clean);
                        detected_size = Some(format!("{}pt", formatted_size));
                    } else if k_lower == "pixelsize" && !v_clean.is_empty() {
                        let formatted_size = format_size_num(v_clean);
                        detected_size = Some(format!("{}px", formatted_size));
                    }
                }
            }
        }
    }

    if families.is_empty() {
        return None;
    }

    let families_str = families.join(", ");
    if let Some(sz) = detected_size {
        Some(format!("{} ({})", families_str, sz))
    } else {
        Some(families_str)
    }
}

/// Parses Qt font format string (e.g. 'Fira Code,10,-1,5,50,0,0,0,0,0').
pub fn parse_qt_font(raw: &str) -> Option<String> {
    let clean = raw.trim().trim_matches('\'').trim_matches('"').trim();
    if !clean.contains(',') {
        return None;
    }
    let parts: Vec<&str> = clean.split(',').map(|s| s.trim()).collect();
    if parts.len() < 2 {
        return None;
    }
    let family = parts[0];
    if family.is_empty() {
        return None;
    }
    if let Ok(pt) = parts[1].parse::<f32>() {
        if pt > 0.0 {
            return Some(format!("{} ({}pt)", family, format_size_num(parts[1])));
        }
    }
    if parts.len() >= 3 {
        if let Ok(px) = parts[2].parse::<f32>() {
            if px > 0.0 {
                return Some(format!("{} ({}px)", family, format_size_num(parts[2])));
            }
        }
    }
    None
}

/// Parses Pango font description (e.g. 'Fira Code 12', 'Cantarell Bold 11').
pub fn parse_pango_font(raw: &str) -> Option<String> {
    let clean = raw.trim().trim_matches('\'').trim_matches('"').trim();
    if clean.is_empty() {
        return None;
    }
    if clean.ends_with("pt)") || clean.ends_with("px)") {
        return Some(clean.to_string());
    }
    if let Some((family, size_str)) = clean.rsplit_once(' ') {
        let is_px = size_str.ends_with("px");
        let size_num = size_str
            .trim_end_matches("pt")
            .trim_end_matches("px")
            .trim();
        if size_num.parse::<f32>().is_ok() {
            let formatted_sz = format_size_num(size_num);
            let unit = if is_px { "px" } else { "pt" };
            return Some(format!("{} ({}{})", family.trim(), formatted_sz, unit));
        }
    }
    None
}

/// Universal font normalizer that standardizes Fontconfig patterns, Pango descriptions,
/// Qt font strings, and pre-formatted font definitions across all terminal emulators.
pub fn normalize_font_string(raw: &str) -> String {
    let clean = raw.trim().trim_matches('\'').trim_matches('"').trim();
    if clean.is_empty() {
        return String::new();
    }
    if clean.ends_with("pt)") || clean.ends_with("px)") {
        return clean.to_string();
    }
    // 1. Fontconfig pattern with properties (contains ':')
    if clean.contains(':') {
        if let Some(parsed) = parse_fontconfig_pattern(clean) {
            return parsed;
        }
    }
    // 2. Comma-separated font descriptions: try Qt font format first, then Fontconfig fallback list
    if clean.contains(',') {
        if let Some(parsed) = parse_qt_font(clean) {
            return parsed;
        }
        if let Some(parsed) = parse_fontconfig_pattern(clean) {
            return parsed;
        }
    }
    // 3. Fontconfig family with dash-size suffix (e.g. "DejaVu-10")
    if clean.contains('-') && clean.chars().any(|c| c.is_ascii_digit()) {
        if let Some(parsed) = parse_fontconfig_pattern(clean) {
            return parsed;
        }
    }
    // 4. Pango font format (e.g. "Fira Code 12", "Cantarell Bold 11")
    if let Some(parsed) = parse_pango_font(clean) {
        return parsed;
    }
    clean.to_string()
}

/// Formats Pango / Fontconfig font descriptions (e.g. 'FiraCode Nerd Font 12') into display format ('FiraCode Nerd Font (12pt)').
pub fn format_pango_font(raw: &str) -> String {
    normalize_font_string(raw)
}

#[cfg(not(windows))]
fn probe_foot_font(home_path: &std::path::Path) -> Option<String> {
    let foot_ini = home_path.join(".config/foot/foot.ini");
    if let Ok(content) = fs::read_to_string(foot_ini) {
        let mut raw_font = None;
        let mut size_override = None;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("font=") {
                let f = rest.trim();
                if !f.is_empty() {
                    raw_font = Some(f.to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("font-size-override=") {
                let s = rest.trim();
                if !s.is_empty() {
                    size_override = Some(s.to_string());
                }
            }
        }
        if let Some(rf) = raw_font {
            let normalized = normalize_font_string(&rf);
            if let Some(override_sz) = size_override {
                let clean_sz = format_size_num(&override_sz);
                if let Some(idx) = normalized.rfind('(') {
                    let family_part = normalized[..idx].trim();
                    return Some(format!("{} ({}pt)", family_part, clean_sz));
                } else {
                    return Some(format!("{} ({}pt)", normalized, clean_sz));
                }
            }
            return Some(normalized);
        }
    }
    Some("monospace (8pt)".to_string())
}

#[cfg(not(windows))]
fn probe_ptyxis_font(home_path: &std::path::Path) -> Option<String> {
    let cache_path = get_terminal_cache_path("ptyxis_font");
    let dconf_path = home_path.join(".config/dconf/user");

    let dconf_mtime = if let Ok(meta) = fs::metadata(&dconf_path) {
        meta.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|dur| dur.as_secs())
            .unwrap_or(0)
    } else {
        0
    };

    // 1. Check persistent cache (sub-microsecond execution)
    if dconf_mtime > 0 {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Some((cached_mtime_str, font_str)) = content.split_once('|') {
                if let Ok(cached_mtime) = cached_mtime_str.parse::<u64>() {
                    if cached_mtime == dconf_mtime && !font_str.trim().is_empty() {
                        return Some(font_str.trim().to_string());
                    }
                }
            }
        }
    }

    // 2. Direct binary parse of ~/.config/dconf/user (no subprocess fork)
    if dconf_mtime > 0 {
        if let Ok(data) = fs::read(&dconf_path) {
            if let Some(pos) = data.windows(10).position(|w| w == b"font-name\0") {
                let rest = &data[pos + 10..];
                let mut start = 0;
                while start < rest.len() && rest[start] == 0 {
                    start += 1;
                }
                if let Some(end) = rest[start..].iter().position(|&b| b == 0) {
                    if let Ok(raw_font) = std::str::from_utf8(&rest[start..start + end]) {
                        let trimmed = raw_font.trim();
                        if trimmed.len() >= 3
                            && trimmed.len() <= 80
                            && trimmed.chars().all(|c| c.is_ascii_graphic() || c == ' ')
                        {
                            let formatted = format_pango_font(trimmed);
                            if !formatted.is_empty() {
                                let _ = fs::write(
                                    &cache_path,
                                    format!("{}|{}", dconf_mtime, formatted),
                                );
                                return Some(formatted);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: dconf / gsettings subprocess if direct binary parse missed
    let raw = crate::modules::system_command("dconf")
        .args(["read", "/org/gnome/Ptyxis/font-name"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
            None
        })
        .or_else(|| {
            let out = crate::modules::system_command("dconf")
                .args(["read", "/org/gnome/desktop/interface/monospace-font-name"])
                .output()
                .ok()?;
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
            None
        })
        .or_else(|| {
            let out = crate::modules::system_command("gsettings")
                .args(["get", "org.gnome.Ptyxis", "font-name"])
                .output()
                .ok()?;
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
            None
        });

    if let Some(r) = raw {
        let formatted = format_pango_font(&r);
        if !formatted.is_empty() {
            if dconf_mtime > 0 {
                let _ = fs::write(&cache_path, format!("{}|{}", dconf_mtime, formatted));
            }
            return Some(formatted);
        }
    }

    None
}

#[cfg(not(windows))]
fn probe_wezterm_font(home_path: &std::path::Path) -> Option<String> {
    for path in &[
        home_path.join(".config/wezterm/wezterm.lua"),
        home_path.join(".wezterm.lua"),
    ] {
        if let Ok(content) = fs::read_to_string(path) {
            let mut family = None;
            let mut size = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("--") {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_end_matches(';').trim();
                    if (k == "font_size" || k.ends_with(".font_size")) && size.is_none() {
                        let clean_v = v.trim_matches('"').trim_matches('\'').trim();
                        if clean_v.parse::<f32>().is_ok() {
                            size = Some(clean_v.to_string());
                        }
                    }
                }
                if family.is_none() && trimmed.contains("wezterm.font") {
                    if let Some(start_quote) = trimmed.find(['"', '\'']) {
                        let quote_char = trimmed.as_bytes()[start_quote] as char;
                        let rest = &trimmed[start_quote + 1..];
                        if let Some(end_quote) = rest.find(quote_char) {
                            let f = rest[..end_quote].trim();
                            if !f.is_empty() {
                                family = Some(f.to_string());
                            }
                        }
                    }
                }
            }
            if let Some(f) = family {
                if let Some(s) = size {
                    let sz = format_size_num(&s);
                    return Some(format!("{} ({}pt)", f, sz));
                }
                return Some(f);
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn probe_konsole_font(home_path: &std::path::Path) -> Option<String> {
    let konsolerc = home_path.join(".config/konsolerc");
    if let Ok(content) = fs::read_to_string(&konsolerc) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("Font=") {
                let normalized = normalize_font_string(rest.trim());
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }
    let profiles_dir = home_path.join(".local/share/konsole");
    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "profile") {
                if let Ok(content) = fs::read_to_string(path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(rest) = trimmed.strip_prefix("Font=") {
                            let normalized = normalize_font_string(rest.trim());
                            if !normalized.is_empty() {
                                return Some(normalized);
                            }
                        }
                    }
                }
            }
        }
    }
    let kdeglobals = home_path.join(".config/kdeglobals");
    if let Ok(content) = fs::read_to_string(kdeglobals) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("fixed=") {
                let normalized = normalize_font_string(rest.trim());
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }
    Some("monospace (10pt)".to_string())
}

#[cfg(not(windows))]
fn probe_xfce_font(home_path: &std::path::Path) -> Option<String> {
    let rc = home_path.join(".config/xfce4/terminal/terminalrc");
    if let Ok(content) = fs::read_to_string(rc) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("FontName=") {
                let normalized = normalize_font_string(rest.trim());
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn probe_gnome_console_font(_home_path: &std::path::Path) -> Option<String> {
    let raw = crate::modules::system_command("dconf")
        .args(["read", "/org/gnome/Console/custom-font"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
            None
        })
        .or_else(|| {
            let out = crate::modules::system_command("dconf")
                .args(["read", "/org/gnome/desktop/interface/monospace-font-name"])
                .output()
                .ok()?;
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
            None
        });

    if let Some(r) = raw {
        let normalized = normalize_font_string(&r);
        if !normalized.is_empty() {
            return Some(normalized);
        }
    }
    None
}

/// Probes the active terminal font configuration from local user dotfiles or system settings.
#[cfg(not(windows))]
pub fn detect_terminal_font() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let home_path = std::path::Path::new(&home);

    let active = detect_terminal().unwrap_or_default().to_lowercase();

    // Prioritize active terminal detection first (strictly isolated, NO cross-terminal contamination)
    if active.contains("ptyxis") {
        return probe_ptyxis_font(home_path);
    } else if active.contains("foot") {
        return probe_foot_font(home_path);
    } else if active.contains("ghostty") {
        return probe_ghostty_font(home_path);
    } else if active.contains("kitty") {
        return probe_kitty_font(home_path);
    } else if active.contains("alacritty") {
        return probe_alacritty_font(home_path);
    } else if active.contains("wezterm") {
        return probe_wezterm_font(home_path);
    } else if active.contains("konsole") || active.contains("yakuake") {
        return probe_konsole_font(home_path);
    } else if active.contains("xfce") {
        return probe_xfce_font(home_path);
    } else if active.contains("kgx") || active.contains("console") {
        return probe_gnome_console_font(home_path);
    }

    // Shield pure multiplexers: if session is running inside tmux/screen/zellij and no outer GUI terminal
    // was identified, NEVER fall through to leak Ghostty or other local GUI terminal fonts!
    if active.starts_with("tmux") || active.starts_with("screen") || active.starts_with("zellij") {
        return None;
    }

    // Fallback waterfall ONLY when active terminal is unknown or generic ($TERM)
    probe_ghostty_font(home_path)
        .or_else(|| probe_foot_font(home_path))
        .or_else(|| probe_kitty_font(home_path))
        .or_else(|| probe_alacritty_font(home_path))
        .or_else(|| probe_wezterm_font(home_path))
        .or_else(|| probe_konsole_font(home_path))
        .or_else(|| probe_ptyxis_font(home_path))
}

#[cfg(windows)]
pub fn detect_terminal_font() -> Option<String> {
    Some("Consolas (11pt)".to_string())
}

pub struct TerminalFontCollector;

impl Collector for TerminalFontCollector {
    fn id(&self) -> ModuleId {
        ModuleId::TerminalFont
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let font = detect_terminal_font()?;
        Some(ModuleOutput {
            id: ModuleId::TerminalFont,
            label: "Terminal Font".to_string(),
            value: font,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_terminal_from_env_term_program() {
        let res =
            detect_terminal_from_env(Some("WezTerm"), Some("20240203-110809-5046fc22"), &[], None);
        assert_eq!(res, Some("WezTerm 20240203-110809-5046fc22".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_alacritty() {
        let envs = [("ALACRITTY_SOCKET", "/run/user/1000/alacritty.sock")];
        let res = detect_terminal_from_env(None, None, &envs, None);
        assert_eq!(res, Some("Alacritty".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_kitty() {
        let envs = [("KITTY_PID", "12345")];
        let res = detect_terminal_from_env(None, None, &envs, None);
        assert_eq!(res, Some("kitty".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_konsole() {
        let envs = [("KONSOLE_VERSION", "230805")];
        let res = detect_terminal_from_env(None, None, &envs, None);
        assert_eq!(res, Some("Konsole 230805".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_fallback_term() {
        let res = detect_terminal_from_env(None, None, &[], Some("xterm-256color"));
        assert_eq!(res, Some("xterm-256color".to_string()));

        let res_unknown = detect_terminal_from_env(None, None, &[], Some("unknown"));
        assert_eq!(res_unknown, None);

        let res_none = detect_terminal_from_env(None, None, &[], None);
        assert_eq!(res_none, None);
    }

    #[test]
    fn test_detect_windows_terminals() {
        // 1. Windows Terminal
        let wt_env = [("WT_SESSION", "3f2e1a0b-1234-5678-9abc-def012345678")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &wt_env),
            "Windows Terminal"
        );

        // 2. Visual Studio Code via TERM_PROGRAM
        assert_eq!(
            detect_windows_terminal_from_env(Some("vscode"), Some("1.87.2"), &[]),
            "Visual Studio Code 1.87.2"
        );
        assert_eq!(
            detect_windows_terminal_from_env(Some("Code"), None, &[]),
            "Visual Studio Code"
        );

        // 3. Visual Studio Code via VSCODE_INJECTION
        let vscode_inj = [("VSCODE_INJECTION", "1")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &vscode_inj),
            "Visual Studio Code"
        );

        // 4. Alacritty
        let alacritty_env = [("ALACRITTY_LOG", "C:\\alacritty.log")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &alacritty_env),
            "Alacritty"
        );

        // 5. WezTerm
        let wezterm_env = [("WEZTERM_PANE", "0")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &wezterm_env),
            "WezTerm"
        );

        // 6. ConHost fallback
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &[]),
            "Console Window Host (ConHost)"
        );
    }

    #[test]
    fn test_match_terminal_proc() {
        assert_eq!(match_terminal_proc("st"), Some("st"));
        assert_eq!(match_terminal_proc("stterm"), Some("st"));
        assert_eq!(match_terminal_proc("st-256color"), Some("st"));
        assert_eq!(match_terminal_proc("alacritty"), Some("Alacritty"));
        assert_eq!(match_terminal_proc("kitty"), Some("kitty"));
        assert_eq!(
            match_terminal_proc("gnome-terminal-server"),
            Some("GNOME Terminal")
        );

        // Ensure false-positive substrings do not match
        assert_eq!(match_terminal_proc("systemd"), None);
        assert_eq!(match_terminal_proc("starship"), None);
        assert_eq!(match_terminal_proc("strace"), None);
        assert_eq!(match_terminal_proc("install"), None);
        assert_eq!(match_terminal_proc("gst-plugin"), None);
    }

    #[test]
    fn test_extract_terminal_version_from_output() {
        assert_eq!(
            extract_terminal_version_from_output("kitty 0.48.2 created by Kovid Goyal").as_deref(),
            Some("0.48.2")
        );
        assert_eq!(
            extract_terminal_version_from_output("alacritty 0.17.0").as_deref(),
            Some("0.17.0")
        );
        assert_eq!(
            extract_terminal_version_from_output("foot version: 1.16.2").as_deref(),
            Some("1.16.2")
        );
        assert_eq!(
            extract_terminal_version_from_output("wezterm 20240203-110809-5046fc22").as_deref(),
            Some("20240203-110809-5046fc22")
        );
        assert_eq!(
            extract_terminal_version_from_output("GNOME Terminal 3.50.1 using VTE 0.74.0 +BFD")
                .as_deref(),
            Some("3.50.1")
        );
        assert_eq!(
            extract_terminal_version_from_output("tmux 3.4").as_deref(),
            Some("3.4")
        );
        assert_eq!(
            extract_terminal_version_from_output("invalid output without version"),
            None
        );
    }

    #[test]
    fn test_append_version_if_missing() {
        assert_eq!(
            append_version_if_missing("kitty", Some("0.48.2")),
            "kitty 0.48.2"
        );
        assert_eq!(
            append_version_if_missing("Ptyxis 47.0", Some("47.0")),
            "Ptyxis 47.0"
        );
        assert_eq!(append_version_if_missing("Alacritty", None), "Alacritty");
    }

    #[test]
    fn test_parse_ghostty_font() {
        let conf = r#"
# Comments should be ignored
font-family = "Fira Code"
font-family = Symbols Nerd Font Mono
font-size = 13
"#;
        assert_eq!(
            parse_ghostty_font(conf),
            Some("Fira Code, Symbols Nerd Font Mono (13pt)".to_string())
        );

        let conf_no_size = "font-family = JetBrainsMono Nerd Font";
        assert_eq!(
            parse_ghostty_font(conf_no_size),
            Some("JetBrainsMono Nerd Font".to_string())
        );

        let conf_only_size = "font-size = 14";
        assert_eq!(parse_ghostty_font(conf_only_size), Some("14pt".to_string()));
    }

    #[test]
    fn test_format_pango_font() {
        assert_eq!(
            format_pango_font("FiraCode Nerd Font 12"),
            "FiraCode Nerd Font (12pt)"
        );
        assert_eq!(
            format_pango_font("'FiraCode Nerd Font 12'"),
            "FiraCode Nerd Font (12pt)"
        );
        assert_eq!(
            format_pango_font("\"Adwaita Mono 11\""),
            "Adwaita Mono (11pt)"
        );
        assert_eq!(
            format_pango_font("JetBrains Mono 10.5"),
            "JetBrains Mono (10.5pt)"
        );
        assert_eq!(format_pango_font("Monospace"), "Monospace");
        assert_eq!(format_pango_font("Fira Code (13pt)"), "Fira Code (13pt)");
    }

    #[test]
    fn test_format_size_num() {
        assert_eq!(format_size_num("12"), "12");
        assert_eq!(format_size_num("12.0"), "12");
        assert_eq!(format_size_num("12.5"), "12.5");
        assert_eq!(format_size_num("13pt"), "13");
        assert_eq!(format_size_num("14.0px"), "14");
    }

    #[test]
    fn test_parse_fontconfig_pattern() {
        assert_eq!(
            parse_fontconfig_pattern("Fira Code:size=12, Symbols Nerd Font Mono:size=12"),
            Some("Fira Code, Symbols Nerd Font Mono (12pt)".to_string())
        );
        assert_eq!(
            parse_fontconfig_pattern(
                "JetBrains Mono:style=Regular:size=11, Noto Color Emoji:size=11"
            ),
            Some("JetBrains Mono, Noto Color Emoji (11pt)".to_string())
        );
        assert_eq!(
            parse_fontconfig_pattern("DejaVu Sans Mono-10"),
            Some("DejaVu Sans Mono (10pt)".to_string())
        );
        assert_eq!(
            parse_fontconfig_pattern("monospace:pixelsize=16"),
            Some("monospace (16px)".to_string())
        );
        assert_eq!(
            parse_fontconfig_pattern("Courier New:size=12.5"),
            Some("Courier New (12.5pt)".to_string())
        );
        assert_eq!(
            parse_fontconfig_pattern("Fira Code"),
            Some("Fira Code".to_string())
        );
    }

    #[test]
    fn test_parse_qt_font() {
        assert_eq!(
            parse_qt_font("Fira Code,10,-1,5,50,0,0,0,0,0"),
            Some("Fira Code (10pt)".to_string())
        );
        assert_eq!(
            parse_qt_font("JetBrains Mono,-1,14,5,50,0,0,0,0,0"),
            Some("JetBrains Mono (14px)".to_string())
        );
        assert_eq!(parse_qt_font("Invalid"), None);
    }

    #[test]
    fn test_normalize_font_string() {
        assert_eq!(
            normalize_font_string("Fira Code:size=12, Symbols Nerd Font Mono:size=12"),
            "Fira Code, Symbols Nerd Font Mono (12pt)"
        );
        assert_eq!(normalize_font_string("Fira Code 12"), "Fira Code (12pt)");
        assert_eq!(
            normalize_font_string("Fira Code,10,-1,5,50,0,0,0,0,0"),
            "Fira Code (10pt)"
        );
        assert_eq!(
            normalize_font_string("Fira Code, Symbols Nerd Font Mono (13pt)"),
            "Fira Code, Symbols Nerd Font Mono (13pt)"
        );
        assert_eq!(normalize_font_string("Monospace"), "Monospace");
    }

    #[test]
    fn test_detect_terminal_from_env_multiplexer_deferred() {
        let res_tmux = detect_terminal_from_env(Some("tmux"), Some("3.7c"), &[], None);
        assert_eq!(res_tmux, None);

        let res_screen = detect_terminal_from_env(Some("screen"), Some("4.09.00"), &[], None);
        assert_eq!(res_screen, None);

        let res_zellij = detect_terminal_from_env(Some("zellij"), Some("0.40.1"), &[], None);
        assert_eq!(res_zellij, None);
    }

    #[test]
    fn test_detect_terminal_from_env_multiplexer_with_outer_signatures() {
        let kitty_env = [("KITTY_PID", "1234")];
        assert_eq!(
            detect_terminal_from_env(Some("tmux"), Some("3.7c"), &kitty_env, None),
            Some("kitty".to_string())
        );

        let foot_env = [("FOOT_PID", "5678")];
        assert_eq!(
            detect_terminal_from_env(Some("tmux"), Some("3.7c"), &foot_env, None),
            Some("foot".to_string())
        );

        let alacritty_env = [("ALACRITTY_SOCKET", "/tmp/alacritty.sock")];
        assert_eq!(
            detect_terminal_from_env(Some("tmux"), Some("3.7c"), &alacritty_env, None),
            Some("Alacritty".to_string())
        );

        let ghostty_env = [("GHOSTTY_RESOURCES_DIR", "/usr/share/ghostty")];
        assert_eq!(
            detect_terminal_from_env(Some("tmux"), Some("3.7c"), &ghostty_env, None),
            Some("Ghostty".to_string())
        );
    }
}
