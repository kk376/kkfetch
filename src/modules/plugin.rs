use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

pub struct PluginCollector;

/// Checks whether kkfetch is running in an elevated/privileged context (e.g. sudo, su, setuid).
/// In elevated contexts, user-configured arbitrary plugins are intentionally disabled to prevent
/// privilege escalation via untrusted user configuration (F1).
#[cfg(unix)]
fn is_elevated_context() -> bool {
    // SAFETY: getuid/geteuid/getgid/getegid are POSIX syscalls with no pointer arguments.
    unsafe {
        let uid = libc::getuid();
        let euid = libc::geteuid();
        let gid = libc::getgid();
        let egid = libc::getegid();
        if uid != euid || euid == 0 || gid != egid || egid == 0 {
            return true;
        }
    }
    std::env::var_os("SUDO_USER").is_some() || std::env::var_os("DOAS_USER").is_some()
}

#[cfg(not(unix))]
fn is_elevated_context() -> bool {
    false
}

/// Checks whether the plugins directory itself is a non-symlink directory owned by the current user or root
/// and not group/world-writable.
fn is_safe_plugin_dir(dir: &Path) -> bool {
    #[cfg(unix)]
    {
        let Ok(meta) = std::fs::symlink_metadata(dir) else {
            return false;
        };
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return false;
        }
        if meta.permissions().mode() & 0o022 != 0 {
            return false;
        }
        // SAFETY: getuid is a safe POSIX call returning the user ID without memory side-effects.
        let current_uid = unsafe { libc::getuid() };
        let dir_uid = meta.uid();
        dir_uid == current_uid || dir_uid == 0
    }
    #[cfg(not(unix))]
    {
        dir.is_dir()
    }
}

/// Checks whether a plugin file in ~/.config/kkfetch/plugins/ is safe and valid to execute (F2).
/// Requires:
/// 1. Regular file (not symlink or directory)
/// 2. Not a hidden file or editor backup (.swp, ~)
/// 3. Executable bit (+x) is set and not group/world-writable on Unix
/// 4. Owned by current user or root
fn is_safe_plugin_file(path: &Path) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    // Ignore hidden files, swap files, editor backups
    if file_name.starts_with('.') || file_name.ends_with('~') || file_name.ends_with(".swp") {
        return false;
    }

    #[cfg(unix)]
    {
        let meta = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        // Must be a regular file, not a symlink pointing elsewhere
        if !meta.file_type().is_file() {
            return false;
        }

        let mode = meta.permissions().mode();
        // Reject files without execute bit or writable by group/others
        if mode & 0o111 == 0 || mode & 0o022 != 0 {
            return false;
        }

        // Must be owned by current user or root
        // SAFETY: getuid is a safe POSIX call returning the user ID without memory side-effects.
        let current_uid = unsafe { libc::getuid() };
        let file_uid = meta.uid();
        if file_uid != current_uid && file_uid != 0 {
            return false;
        }
    }

    #[cfg(not(unix))]
    {
        if !path.is_file() {
            return false;
        }
    }

    true
}

impl Collector for PluginCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Plugin
    }

    fn collect_multiple(&self, ctx: &FetchContext) -> Vec<ModuleOutput> {
        // Prevent arbitrary code execution when running with elevated privileges (F1) or if disabled via CLI
        if is_elevated_context() || ctx.no_plugins {
            return Vec::new();
        }

        let mut outputs = Vec::new();

        // 1. Plugins defined in config.toml
        if let Some(ref plugins) = ctx.config.plugins {
            for plugin in plugins {
                if let Some(out) = execute_plugin_command(&plugin.name, &plugin.command) {
                    outputs.push(out);
                }
            }
        }

        // 2. Executable binaries/scripts in ~/.config/kkfetch/plugins/ invoked directly without a shell
        if let Some(plugins_dir) = crate::config::get_default_config_path()
            .and_then(|p| p.parent().map(|d| d.join("plugins")))
        {
            if is_safe_plugin_dir(&plugins_dir) {
                if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if is_safe_plugin_file(&path) {
                            let name = path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            if let Some(out) = collect_command_output(&name, Command::new(&path)) {
                                outputs.push(out);
                            }
                        }
                    }
                }
            }
        }

        outputs
    }
}

/// Splits a command string into `(program, args)` supporting single/double quotes and backslash
/// escapes without spawning a shell.
fn split_command_args(command: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_token = false;
    let mut quote: Option<char> = None;
    let mut chars = command.chars();

    while let Some(c) = chars.next() {
        match quote {
            Some('\'') => {
                if c == '\'' {
                    quote = None;
                } else {
                    current.push(c);
                }
            }
            Some('"') => {
                if c == '"' {
                    quote = None;
                } else if c == '\\' {
                    current.push(chars.next()?);
                } else {
                    current.push(c);
                }
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    in_token = true;
                }
                '\\' => {
                    let next = chars.next()?;
                    current.push(next);
                    in_token = true;
                }
                c if c.is_whitespace() => {
                    if in_token {
                        args.push(std::mem::take(&mut current));
                        in_token = false;
                    }
                }
                _ => {
                    current.push(c);
                    in_token = true;
                }
            },
            Some(_) => unreachable!(),
        }
    }

    if quote.is_some() {
        return None;
    }
    if in_token {
        args.push(current);
    }
    if args.is_empty() { None } else { Some(args) }
}

fn collect_command_output(name: &str, mut cmd: Command) -> Option<ModuleOutput> {
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let clean = stdout.trim();
    if clean.is_empty() {
        return None;
    }
    Some(ModuleOutput {
        id: ModuleId::Plugin,
        label: name.to_string(),
        value: clean.to_string(),
        custom_rendered: None,
    })
}

pub fn execute_plugin_command(name: &str, command: &str) -> Option<ModuleOutput> {
    let parts = split_command_args(command)?;
    let (program, args) = parts.split_first()?;
    let mut cmd = if !program.contains('/') && !program.contains('\\') {
        crate::modules::system_command(program)
    } else {
        Command::new(program)
    };
    cmd.args(args);
    collect_command_output(name, cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_plugin_command_echo() {
        let out = execute_plugin_command("TestEcho", "echo HelloFerris").unwrap();
        assert_eq!(out.label, "TestEcho");
        assert_eq!(out.value, "HelloFerris");
        assert_eq!(out.id, ModuleId::Plugin);
    }

    #[test]
    fn test_is_safe_plugin_file_filters_invalid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let regular_txt = temp_dir.path().join("readme.txt");
        std::fs::write(&regular_txt, "hello").unwrap();

        #[cfg(unix)]
        {
            // Non-executable should be rejected
            assert!(!is_safe_plugin_file(&regular_txt));

            // Executable should be accepted
            let mut perms = std::fs::metadata(&regular_txt).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&regular_txt, perms).unwrap();
            assert!(is_safe_plugin_file(&regular_txt));
        }

        // Hidden files should be rejected
        let hidden = temp_dir.path().join(".hidden_script");
        std::fs::write(&hidden, "echo hidden").unwrap();
        assert!(!is_safe_plugin_file(&hidden));

        #[cfg(unix)]
        {
            // Group-writable or world-writable executable files should be rejected
            let gw_script = temp_dir.path().join("group_writable.sh");
            std::fs::write(&gw_script, "#!/bin/sh\necho nope\n").unwrap();
            let mut perms = std::fs::metadata(&gw_script).unwrap().permissions();
            perms.set_mode(0o775);
            std::fs::set_permissions(&gw_script, perms).unwrap();
            assert!(!is_safe_plugin_file(&gw_script));

            // Symlinks pointing to executable scripts should be rejected
            let symlink_path = temp_dir.path().join("linked.sh");
            std::os::unix::fs::symlink(&regular_txt, &symlink_path).unwrap();
            assert!(!is_safe_plugin_file(&symlink_path));
        }
    }

    #[test]
    fn test_execute_plugin_command_no_shell_metachar_interpretation() {
        let out = execute_plugin_command("EchoLiteral", "printf %s $(id)").unwrap();
        assert_eq!(out.value, "$(id)");
        assert!(execute_plugin_command("Chained", "/bin/true && echo pwned").is_none());
    }
}
