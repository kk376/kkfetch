//! Diagnostic module for detecting conflicting and shadowed `kkfetch` binary installations in `$PATH`.

use std::path::PathBuf;

/// Scans the provided `PATH` string (or system `$PATH` if `None`) for existing `kkfetch` binaries.
pub fn find_kkfetch_in_path(path_env: Option<&std::ffi::OsStr>) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let exe_name = if cfg!(windows) {
        "kkfetch.exe"
    } else {
        "kkfetch"
    };

    let path_val = match path_env {
        Some(val) => Some(val.to_os_string()),
        None => std::env::var_os("PATH"),
    };

    if let Some(paths) = path_val {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(exe_name);
            if candidate.is_file() {
                let canonical = candidate
                    .canonicalize()
                    .unwrap_or_else(|_| candidate.clone());
                if !found
                    .iter()
                    .any(|p: &PathBuf| p.canonicalize().unwrap_or_else(|_| p.clone()) == canonical)
                {
                    found.push(candidate);
                }
            }
        }
    }
    found
}

/// Inspects the environment and prints diagnostic feedback if multiple `kkfetch` binaries exist.
pub fn check_binary_shadowing(is_explicit_doctor: bool) {
    let found = find_kkfetch_in_path(None);
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("kkfetch"));
    let current_canonical = current_exe
        .canonicalize()
        .unwrap_or_else(|_| current_exe.clone());

    if found.len() > 1 {
        eprintln!("\x1b[33mNotice: Multiple kkfetch installations detected in your PATH:\x1b[0m");
        for (i, p) in found.iter().enumerate() {
            let p_canonical = p.canonicalize().unwrap_or_else(|_| p.clone());
            let is_current = p_canonical == current_canonical;
            let is_first_in_path = i == 0;

            let marker = if is_current && is_first_in_path {
                " \x1b[32m(active & executing)\x1b[0m"
            } else if is_current {
                " \x1b[33m(executing, but shadowed in PATH by #1)\x1b[0m"
            } else if is_first_in_path {
                " \x1b[32m(active in PATH)\x1b[0m"
            } else {
                " \x1b[31m(shadowed in PATH)\x1b[0m"
            };
            eprintln!("  {}. {}{}", i + 1, p.display(), marker);
        }
        eprintln!("Your shell executes the first match in PATH (#1).");
        eprintln!("If an upgrade did not take effect, remove older binaries with:");
        eprintln!("  cargo uninstall kkfetch   # for ~/.cargo/bin/kkfetch");
        eprintln!("  rm -f <path-to-binary>    # for manual or local installs");
    } else if is_explicit_doctor {
        if found.len() == 1 {
            println!(
                "\x1b[32mInstallation is clean:\x1b[0m Single kkfetch binary found in PATH at: {}",
                found[0].display()
            );
        } else {
            println!(
                "\x1b[33mNotice:\x1b[0m No kkfetch binary found in current PATH. Currently executing from: {}",
                current_exe.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_find_kkfetch_in_mock_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir1 = temp_dir.path().join("bin1");
        let dir2 = temp_dir.path().join("bin2");
        let dir3 = temp_dir.path().join("bin3");

        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();
        fs::create_dir_all(&dir3).unwrap();

        let exe_name = if cfg!(windows) {
            "kkfetch.exe"
        } else {
            "kkfetch"
        };
        File::create(dir1.join(exe_name)).unwrap();
        File::create(dir2.join(exe_name)).unwrap();

        let mock_path = std::env::join_paths([&dir1, &dir2, &dir3]).unwrap();
        let found = find_kkfetch_in_path(Some(&mock_path));

        assert_eq!(found.len(), 2);
        assert_eq!(found[0], dir1.join(exe_name));
        assert_eq!(found[1], dir2.join(exe_name));
    }

    #[test]
    fn test_find_kkfetch_empty_path() {
        let empty = std::ffi::OsString::new();
        let found = find_kkfetch_in_path(Some(&empty));
        assert!(found.is_empty());
    }
}
