//! Shared user-private file cache helpers with ownership and symlink validation.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

const MAX_CACHE_BYTES: u64 = 1024 * 1024;
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheScope {
    /// Persistent cache (`$XDG_CACHE_HOME/kkfetch` or `$HOME/.cache/kkfetch`).
    Persistent,
    /// Ephemeral runtime cache (`$XDG_RUNTIME_DIR/kkfetch`, `/dev/shm/kkfetch-<uid>`, or private temp).
    Runtime,
    /// Persistent cache when configured, falling back to private runtime/temp storage.
    PersistentOrRuntime,
}

/// Resolves the target directory for `scope` without creating it.
pub fn resolve_cache_dir(scope: CacheScope) -> Option<PathBuf> {
    match scope {
        CacheScope::Persistent => persistent_cache_dir(),
        CacheScope::Runtime => runtime_cache_dir(),
        CacheScope::PersistentOrRuntime => persistent_cache_dir().or_else(runtime_cache_dir),
    }
}

/// Resolves a verified cache directory path inside an explicit root base directory.
pub(crate) fn ensure_private_dir(dir: &Path) -> bool {
    if let Err(err) = fs::symlink_metadata(dir) {
        if err.kind() != std::io::ErrorKind::NotFound {
            return false;
        }
        if fs::create_dir_all(dir).is_err() {
            return false;
        }
    }

    let Ok(meta) = fs::symlink_metadata(dir) else {
        return false;
    };
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        // SAFETY: getuid takes no pointers and returns the real user ID.
        let current_uid = unsafe { libc::getuid() };
        if meta.uid() != current_uid {
            return false;
        }
        if meta.permissions().mode() & 0o077 != 0
            && fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).is_err()
        {
            return false;
        }
    }

    true
}

/// Reads cached text if the cache file exists, is owned by the current user, and is not a symlink.
pub fn read_cache(scope: CacheScope, file_name: &str) -> Option<String> {
    read_cache_in_dir(resolve_cache_dir(scope)?.as_path(), file_name).map(|(text, _)| text)
}

/// Reads cached text along with elapsed age since last modification.
pub fn read_cache_with_age(scope: CacheScope, file_name: &str) -> Option<(String, Duration)> {
    let dir = resolve_cache_dir(scope)?;
    let (text, modified) = read_cache_in_dir(&dir, file_name)?;
    let age = modified.elapsed().ok()?;
    Some((text, age))
}

/// Writes `content` atomically inside the private directory resolved for `scope`.
pub fn write_cache(scope: CacheScope, file_name: &str, content: &str) -> bool {
    let Some(dir) = resolve_cache_dir(scope) else {
        return false;
    };
    write_cache_in_dir(&dir, file_name, content)
}

pub(crate) fn read_cache_in_dir(dir: &Path, file_name: &str) -> Option<(String, SystemTime)> {
    if !is_safe_existing_dir(dir) {
        return None;
    }
    let path = safe_child_path(dir, file_name)?;
    let meta = fs::symlink_metadata(&path).ok()?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > MAX_CACHE_BYTES {
        return None;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: getuid takes no pointers and returns the real user ID.
        let current_uid = unsafe { libc::getuid() };
        if meta.uid() != current_uid {
            return None;
        }
    }

    let modified = meta.modified().ok()?;
    let text = fs::read_to_string(&path).ok()?;
    Some((text, modified))
}

pub(crate) fn write_cache_in_dir(dir: &Path, file_name: &str, content: &str) -> bool {
    let Some(target) = safe_child_path(dir, file_name) else {
        return false;
    };
    if !ensure_private_dir(dir) {
        return false;
    }

    // Refuse overwriting a symlink left at the target path.
    if let Ok(meta) = fs::symlink_metadata(&target) {
        if meta.file_type().is_symlink() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            // SAFETY: getuid takes no pointers and returns the real user ID.
            let current_uid = unsafe { libc::getuid() };
            if meta.uid() != current_uid {
                return false;
            }
        }
    }

    let seq = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_path = dir.join(format!(".{}.{}.{}.tmp", file_name, std::process::id(), seq));

    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    let Ok(mut file) = opts.open(&tmp_path) else {
        return false;
    };

    if file.write_all(content.as_bytes()).is_err() || file.flush().is_err() {
        drop(file);
        let _ = fs::remove_file(&tmp_path);
        return false;
    }
    drop(file);

    if fs::rename(&tmp_path, &target).is_err() {
        let _ = fs::remove_file(&tmp_path);
        return false;
    }
    true
}

fn persistent_cache_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("kkfetch"));
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home).join(".cache").join("kkfetch"));
        }
    }
    None
}

fn runtime_cache_dir() -> Option<PathBuf> {
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        if !runtime_dir.is_empty() {
            let base = PathBuf::from(runtime_dir);
            if base.is_dir() {
                return Some(base.join("kkfetch"));
            }
        }
    }

    let user_suffix = current_user_dir_suffix();
    let shm_dir = Path::new("/dev/shm");
    if shm_dir.is_dir() {
        return Some(shm_dir.join(&user_suffix));
    }

    Some(std::env::temp_dir().join(user_suffix))
}

fn current_user_dir_suffix() -> String {
    #[cfg(unix)]
    {
        // SAFETY: getuid takes no pointers and returns the real user ID.
        let uid = unsafe { libc::getuid() };
        format!("kkfetch-{}", uid)
    }
    #[cfg(not(unix))]
    {
        "kkfetch".to_string()
    }
}

fn is_safe_existing_dir(dir: &Path) -> bool {
    let Ok(meta) = fs::symlink_metadata(dir) else {
        return false;
    };
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: getuid takes no pointers and returns the real user ID.
        let current_uid = unsafe { libc::getuid() };
        if meta.uid() != current_uid {
            return false;
        }
    }
    true
}

fn safe_child_path(dir: &Path, file_name: &str) -> Option<PathBuf> {
    if file_name.is_empty()
        || file_name == "."
        || file_name == ".."
        || file_name.contains('/')
        || file_name.contains('\\')
    {
        return None;
    }
    Some(dir.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_round_trip_and_private_permissions() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("kkfetch-cache");

        assert!(write_cache_in_dir(&cache_dir, "sample.cache", "v1:ok\n"));
        let (content, modified) = read_cache_in_dir(&cache_dir, "sample.cache").unwrap();
        assert_eq!(content, "v1:ok\n");
        assert!(modified.elapsed().unwrap() < Duration::from_secs(10));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = fs::symlink_metadata(&cache_dir)
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            let file_mode = fs::symlink_metadata(cache_dir.join("sample.cache"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(dir_mode, 0o700);
            assert_eq!(file_mode, 0o600);
        }
    }

    #[test]
    fn test_cache_rejects_symlinks_and_path_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let real_file = temp.path().join("secret.txt");
        fs::write(&real_file, "secret").unwrap();

        let cache_dir = temp.path().join("cache");
        assert!(ensure_private_dir(&cache_dir));

        #[cfg(unix)]
        {
            let linked_file = cache_dir.join("link.cache");
            std::os::unix::fs::symlink(&real_file, &linked_file).unwrap();
            assert!(read_cache_in_dir(&cache_dir, "link.cache").is_none());
            assert!(!write_cache_in_dir(&cache_dir, "link.cache", "overwrite"));

            let linked_dir = temp.path().join("linked_dir");
            std::os::unix::fs::symlink(&cache_dir, &linked_dir).unwrap();
            assert!(!write_cache_in_dir(&linked_dir, "test.cache", "data"));
            assert!(read_cache_in_dir(&linked_dir, "test.cache").is_none());
        }

        assert!(!write_cache_in_dir(&cache_dir, "../escape.cache", "bad"));
        assert!(read_cache_in_dir(&cache_dir, "../escape.cache").is_none());
    }
}
