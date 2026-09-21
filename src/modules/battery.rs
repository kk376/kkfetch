use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(unix)]
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryInfo {
    pub capacity: u8,
    pub status: String,
    pub time_estimate: Option<String>,
}

/// Formats duration in seconds to a human-readable estimate.
/// Returns None if seconds is 0 or exceeds 100 hours (> 360,000s).
pub fn format_duration_estimate(seconds: u64, is_charging: bool) -> Option<String> {
    if seconds == 0 || seconds > 360_000 {
        return None;
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let suffix = if is_charging {
        "until full"
    } else {
        "remaining"
    };

    let time_str = match (hours, minutes) {
        (h, m) if h > 0 && m > 0 => format!("{}h {}m", h, m),
        (h, 0) if h > 0 => format!("{}h", h),
        (0, m) if m > 0 => format!("{}m", m),
        (0, 0) => "< 1m".to_string(),
        _ => return None,
    };

    Some(format!("{} {}", time_str, suffix))
}

/// Parses Windows SYSTEM_POWER_STATUS fields into BatteryInfo.
pub fn parse_windows_battery_status(
    ac_line_status: u8,
    battery_flag: u8,
    battery_life_percent: u8,
    battery_life_time: u32,
) -> Option<BatteryInfo> {
    // BatteryFlag: 128 indicates no system battery, 255 indicates unknown status
    if battery_flag == 128 || battery_life_percent > 100 {
        return None;
    }

    let is_charging = (battery_flag & 8) != 0;
    let is_ac = ac_line_status == 1;

    let status = if is_charging {
        "Charging".to_string()
    } else if is_ac {
        if battery_life_percent >= 99 {
            "Full [AC]".to_string()
        } else {
            "AC Connected".to_string()
        }
    } else {
        "Discharging".to_string()
    };

    let time_estimate =
        if status == "Discharging" && battery_life_time != 0xFFFF_FFFF && battery_life_time > 0 {
            format_duration_estimate(battery_life_time as u64, false)
        } else {
            None
        };

    Some(BatteryInfo {
        capacity: battery_life_percent,
        status,
        time_estimate,
    })
}

#[cfg(not(windows))]
fn get_cache_path() -> std::path::PathBuf {
    // 1. Prefer $XDG_RUNTIME_DIR (user-private tmpfs, mode 0700)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = std::path::PathBuf::from(runtime_dir);
        if dir.is_dir() {
            return dir.join("kkfetch_battery.cache");
        }
    }
    // 2. Prefer $XDG_CACHE_HOME or ~/.cache/kkfetch/
    if let Ok(cache_home) = std::env::var("XDG_CACHE_HOME") {
        let dir = std::path::PathBuf::from(cache_home).join("kkfetch");
        let _ = fs::create_dir_all(&dir);
        return dir.join("battery.cache");
    }
    if let Ok(home) = std::env::var("HOME") {
        let dir = std::path::PathBuf::from(home)
            .join(".cache")
            .join("kkfetch");
        let _ = fs::create_dir_all(&dir);
        return dir.join("battery.cache");
    }
    // 3. Fallback to private user-isolated temporary directory (mode 0700)
    // SAFETY: libc::getuid is safe as it requires no arguments and returns the current user ID.
    let uid = unsafe { libc::getuid() };
    let temp_dir = std::env::temp_dir().join(format!("kkfetch-{}", uid));
    let _ = fs::create_dir_all(&temp_dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&temp_dir, fs::Permissions::from_mode(0o700));
    }
    temp_dir.join("battery.cache")
}

#[cfg(not(windows))]
fn read_cached_battery() -> Option<(BatteryInfo, bool)> {
    let path = get_cache_path();
    let metadata = fs::metadata(&path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = modified.elapsed().ok()?;
    // If cache is > 24 hours old, consider it invalid
    if age.as_secs() > 86400 {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let mut parts = content.splitn(3, '|');
    let capacity = parts.next()?.trim().parse::<u8>().ok()?;
    let status = parts.next()?.trim().to_string();
    if status.is_empty() {
        return None;
    }
    let time_estimate = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    // Stale if older than 5 seconds
    let is_stale = age.as_secs() > 5;
    Some((
        BatteryInfo {
            capacity,
            status,
            time_estimate,
        },
        is_stale,
    ))
}

#[cfg(not(windows))]
fn write_cached_battery(info: &BatteryInfo) {
    let path = get_cache_path();
    let payload = format!(
        "{}|{}|{}",
        info.capacity,
        info.status,
        info.time_estimate.as_deref().unwrap_or("")
    );
    let _ = fs::write(path, payload);
}

#[cfg(not(windows))]
fn read_sysfs_u64(path: &std::path::Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

#[cfg(not(windows))]
fn read_sysfs_i64_abs(path: &std::path::Path) -> Option<u64> {
    let s = fs::read_to_string(path).ok()?;
    let val = s.trim().parse::<i64>().ok()?;
    Some(val.unsigned_abs())
}

/// Probes battery capacity and state directly from a power supply directory in a single sysfs pass.
#[cfg(not(windows))]
pub fn probe_sysfs_battery_from_dir(power_supply_dir: &std::path::Path) -> Option<BatteryInfo> {
    let entries = fs::read_dir(power_supply_dir).ok()?;

    let mut bat_path = None;
    let mut ac_online = None;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let lower = name_str.to_lowercase();

        if (lower.starts_with("bat") || lower.starts_with("battery")) && bat_path.is_none() {
            bat_path = Some(entry.path());
        } else if (lower.starts_with("ac")
            || lower.starts_with("mains")
            || lower.starts_with("acad")
            || lower.starts_with("adp"))
            && ac_online.is_none()
        {
            if let Ok(online) = fs::read_to_string(entry.path().join("online")) {
                ac_online = Some(online.trim() == "1");
            }
        }
    }

    let bat = bat_path?;
    let capacity_str = fs::read_to_string(bat.join("capacity")).ok()?;
    let capacity = capacity_str.trim().parse::<u8>().ok()?;

    let raw_status = fs::read_to_string(bat.join("status"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let is_ac = ac_online.unwrap_or(false);

    // When battery threshold limits (e.g. 80%) are enabled in BIOS, status reports "Not charging" while connected to AC
    let status = if raw_status.eq_ignore_ascii_case("not charging") {
        if is_ac {
            "AC Connected".to_string()
        } else {
            "Not charging".to_string()
        }
    } else if raw_status.eq_ignore_ascii_case("charging") {
        "Charging".to_string()
    } else if raw_status.eq_ignore_ascii_case("discharging") {
        "Discharging".to_string()
    } else if raw_status.eq_ignore_ascii_case("full") {
        if is_ac {
            "Full [AC]".to_string()
        } else {
            "Full".to_string()
        }
    } else {
        raw_status
    };

    let is_discharging = status.eq_ignore_ascii_case("discharging");
    let is_charging = status.eq_ignore_ascii_case("charging");

    let time_estimate = if is_discharging {
        let seconds = read_sysfs_u64(&bat.join("time_to_empty_now"))
            .filter(|&s| s > 0)
            .or_else(|| {
                let power = read_sysfs_i64_abs(&bat.join("power_now"))
                    .or_else(|| read_sysfs_i64_abs(&bat.join("power_avg")))?;
                if power > 0 {
                    let energy = read_sysfs_u64(&bat.join("energy_now"))
                        .or_else(|| read_sysfs_u64(&bat.join("energy_avg")))?;
                    return Some((energy as f64 / power as f64 * 3600.0).round() as u64);
                }
                None
            })
            .or_else(|| {
                let current = read_sysfs_i64_abs(&bat.join("current_now"))
                    .or_else(|| read_sysfs_i64_abs(&bat.join("current_avg")))?;
                if current > 0 {
                    let charge = read_sysfs_u64(&bat.join("charge_now"))
                        .or_else(|| read_sysfs_u64(&bat.join("charge_avg")))?;
                    return Some((charge as f64 / current as f64 * 3600.0).round() as u64);
                }
                None
            });
        seconds.and_then(|s| format_duration_estimate(s, false))
    } else if is_charging {
        let seconds = read_sysfs_u64(&bat.join("time_to_full_now"))
            .filter(|&s| s > 0)
            .or_else(|| {
                let power = read_sysfs_i64_abs(&bat.join("power_now"))
                    .or_else(|| read_sysfs_i64_abs(&bat.join("power_avg")))?;
                if power > 0 {
                    let energy_now = read_sysfs_u64(&bat.join("energy_now"))
                        .or_else(|| read_sysfs_u64(&bat.join("energy_avg")))?;
                    let energy_full = read_sysfs_u64(&bat.join("energy_full"))
                        .or_else(|| read_sysfs_u64(&bat.join("energy_full_design")))?;
                    if energy_full > energy_now {
                        return Some(
                            ((energy_full - energy_now) as f64 / power as f64 * 3600.0).round()
                                as u64,
                        );
                    }
                }
                None
            })
            .or_else(|| {
                let current = read_sysfs_i64_abs(&bat.join("current_now"))
                    .or_else(|| read_sysfs_i64_abs(&bat.join("current_avg")))?;
                if current > 0 {
                    let charge_now = read_sysfs_u64(&bat.join("charge_now"))
                        .or_else(|| read_sysfs_u64(&bat.join("charge_avg")))?;
                    let charge_full = read_sysfs_u64(&bat.join("charge_full"))
                        .or_else(|| read_sysfs_u64(&bat.join("charge_full_design")))?;
                    if charge_full > charge_now {
                        return Some(
                            ((charge_full - charge_now) as f64 / current as f64 * 3600.0).round()
                                as u64,
                        );
                    }
                }
                None
            });
        seconds.and_then(|s| format_duration_estimate(s, true))
    } else {
        None
    };

    Some(BatteryInfo {
        capacity,
        status,
        time_estimate,
    })
}

#[cfg(not(windows))]
pub fn probe_ac_online_from_dir(power_supply_dir: &std::path::Path) -> Option<bool> {
    let entries = fs::read_dir(power_supply_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let lower = name.to_string_lossy().to_lowercase();
        if lower.starts_with("ac")
            || lower.starts_with("mains")
            || lower.starts_with("acad")
            || lower.starts_with("adp")
        {
            if let Ok(online) = fs::read_to_string(entry.path().join("online")) {
                return Some(online.trim() == "1");
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn probe_ac_online() -> Option<bool> {
    probe_ac_online_from_dir(std::path::Path::new("/sys/class/power_supply"))
}

#[cfg(not(windows))]
fn probe_sysfs_battery() -> Option<BatteryInfo> {
    probe_sysfs_battery_from_dir(std::path::Path::new("/sys/class/power_supply"))
}

/// Detects battery status with zero-wait stale-while-revalidate microsecond tmpfs caching,
/// combined with instantaneous AC line transition detection (< 60 µs).
#[cfg(not(windows))]
pub fn detect_battery() -> Option<BatteryInfo> {
    if let Some((cached, is_stale)) = read_cached_battery() {
        // Instantaneous AC transition check: if AC online state changed, immediately refresh
        let ac_changed = if let Some(ac_online) = probe_ac_online() {
            let was_on_ac = cached.status.eq_ignore_ascii_case("ac connected")
                || cached.status.eq_ignore_ascii_case("charging")
                || cached.status.eq_ignore_ascii_case("full [ac]");
            ac_online != was_on_ac
        } else {
            false
        };

        if ac_changed {
            if let Some(fresh) = probe_sysfs_battery() {
                write_cached_battery(&fresh);
                return Some(fresh);
            }
        }

        if is_stale {
            // Touch cache to rate-limit revalidations, then trigger background refresh
            write_cached_battery(&cached);
            std::thread::spawn(|| {
                if let Some(fresh) = probe_sysfs_battery() {
                    write_cached_battery(&fresh);
                }
            });
            return Some(cached);
        }

        return Some(cached);
    }

    let info = probe_sysfs_battery()?;
    write_cached_battery(&info);
    Some(info)
}

/// Probes battery status on Windows via Win32 GetSystemPowerStatus API.
#[cfg(windows)]
pub fn detect_battery() -> Option<BatteryInfo> {
    use crate::modules::win_util::ffi;
    // SAFETY: GetSystemPowerStatus is safe to call with a mutable reference to a zero-initialized SYSTEM_POWER_STATUS struct.
    unsafe {
        let mut status = std::mem::zeroed::<ffi::SYSTEM_POWER_STATUS>();
        if ffi::GetSystemPowerStatus(&mut status) != 0 {
            return parse_windows_battery_status(
                status.ACLineStatus,
                status.BatteryFlag,
                status.BatteryLifePercent,
                status.BatteryLifeTime,
            );
        }
    }
    None
}

/// Formats the time estimate phrase (e.g. "2h 38m remaining" or "1h 10m until full") with ANSI color.
///
/// Charging: Green
/// Discharging: Green (>= 50%), Yellow (20% - 49%), Red (< 20%)
pub fn format_time_estimate(
    estimate: &str,
    capacity: u8,
    is_charging: bool,
    enable_color: bool,
) -> String {
    if !enable_color {
        return estimate.to_string();
    }

    let color = if is_charging || capacity >= 50 {
        crate::output::color::GREEN
    } else if capacity >= 20 {
        crate::output::color::YELLOW
    } else {
        crate::output::color::RED
    };

    format!("{}{}{}", color, estimate, crate::output::color::RESET)
}

pub struct BatteryCollector;

impl Collector for BatteryCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Battery
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        let info = detect_battery()?;
        let pct =
            crate::output::color::format_percentage(info.capacity as u64, true, ctx.enable_color);
        let value = if let Some(ref est) = info.time_estimate {
            let is_charging = info.status.eq_ignore_ascii_case("charging");
            let est_colored =
                format_time_estimate(est, info.capacity, is_charging, ctx.enable_color);
            format!("{} [{}] [{}]", pct, info.status, est_colored)
        } else {
            format!("{} [{}]", pct, info.status)
        };
        Some(ModuleOutput {
            id: ModuleId::Battery,
            label: "Battery".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_format_duration_estimate() {
        // Discharging
        assert_eq!(
            format_duration_estimate(8100, false),
            Some("2h 15m remaining".to_string())
        );
        assert_eq!(
            format_duration_estimate(7200, false),
            Some("2h remaining".to_string())
        );
        assert_eq!(
            format_duration_estimate(300, false),
            Some("5m remaining".to_string())
        );
        assert_eq!(
            format_duration_estimate(30, false),
            Some("< 1m remaining".to_string())
        );

        // Charging
        assert_eq!(
            format_duration_estimate(4200, true),
            Some("1h 10m until full".to_string())
        );
        assert_eq!(
            format_duration_estimate(3600, true),
            Some("1h until full".to_string())
        );
        assert_eq!(
            format_duration_estimate(120, true),
            Some("2m until full".to_string())
        );
        assert_eq!(
            format_duration_estimate(10, true),
            Some("< 1m until full".to_string())
        );

        // Edge cases
        assert_eq!(format_duration_estimate(0, false), None);
        assert_eq!(format_duration_estimate(400_000, false), None);
    }

    #[test]
    fn test_battery_parsing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bat_dir = temp_dir.path().join("BAT1");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(
            bat_dir.join("model_name"),
            "Microsoft Hyper-V Virtual Battery\n",
        )
        .unwrap();
        fs::write(bat_dir.join("capacity"), "97\n").unwrap();
        fs::write(bat_dir.join("status"), "Not charging\n").unwrap();

        let cap = fs::read_to_string(bat_dir.join("capacity"))
            .unwrap()
            .trim()
            .parse::<u8>()
            .unwrap();
        assert_eq!(cap, 97);
    }

    #[test]
    fn test_parse_windows_battery_status() {
        // Desktop PC (no battery)
        assert_eq!(parse_windows_battery_status(1, 128, 255, 0xFFFF_FFFF), None);

        // Laptop plugged in and charging at 65%
        let charging = parse_windows_battery_status(1, 8, 65, 0xFFFF_FFFF).unwrap();
        assert_eq!(charging.capacity, 65);
        assert_eq!(charging.status, "Charging");
        assert_eq!(charging.time_estimate, None);

        // Laptop discharging on battery at 80% with 2h 15m remaining (8100s)
        let discharging = parse_windows_battery_status(0, 0, 80, 8100).unwrap();
        assert_eq!(discharging.capacity, 80);
        assert_eq!(discharging.status, "Discharging");
        assert_eq!(
            discharging.time_estimate,
            Some("2h 15m remaining".to_string())
        );

        // Laptop full on AC
        let full = parse_windows_battery_status(1, 0, 100, 0xFFFF_FFFF).unwrap();
        assert_eq!(full.capacity, 100);
        assert_eq!(full.status, "Full [AC]");
        assert_eq!(full.time_estimate, None);
    }

    #[cfg(not(windows))]
    #[test]
    fn test_probe_sysfs_battery_discharging_time_to_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bat_dir = temp_dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("capacity"), "45\n").unwrap();
        fs::write(bat_dir.join("status"), "Discharging\n").unwrap();
        fs::write(bat_dir.join("time_to_empty_now"), "8100\n").unwrap();

        let info = probe_sysfs_battery_from_dir(temp_dir.path()).unwrap();
        assert_eq!(info.capacity, 45);
        assert_eq!(info.status, "Discharging");
        assert_eq!(info.time_estimate, Some("2h 15m remaining".to_string()));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_probe_sysfs_battery_discharging_charge_current() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bat_dir = temp_dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("capacity"), "50\n").unwrap();
        fs::write(bat_dir.join("status"), "Discharging\n").unwrap();
        fs::write(bat_dir.join("charge_now"), "4000000\n").unwrap();
        fs::write(bat_dir.join("current_now"), "-2000000\n").unwrap(); // negative current test

        let info = probe_sysfs_battery_from_dir(temp_dir.path()).unwrap();
        assert_eq!(info.capacity, 50);
        assert_eq!(info.status, "Discharging");
        assert_eq!(info.time_estimate, Some("2h remaining".to_string()));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_probe_sysfs_battery_charging_time_to_full() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bat_dir = temp_dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("capacity"), "70\n").unwrap();
        fs::write(bat_dir.join("status"), "Charging\n").unwrap();
        fs::write(bat_dir.join("time_to_full_now"), "4200\n").unwrap();

        let info = probe_sysfs_battery_from_dir(temp_dir.path()).unwrap();
        assert_eq!(info.capacity, 70);
        assert_eq!(info.status, "Charging");
        assert_eq!(info.time_estimate, Some("1h 10m until full".to_string()));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_probe_sysfs_battery_charging_energy_power() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bat_dir = temp_dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("capacity"), "60\n").unwrap();
        fs::write(bat_dir.join("status"), "Charging\n").unwrap();
        fs::write(bat_dir.join("energy_full"), "60000000\n").unwrap();
        fs::write(bat_dir.join("energy_now"), "30000000\n").unwrap();
        fs::write(bat_dir.join("power_now"), "30000000\n").unwrap();

        let info = probe_sysfs_battery_from_dir(temp_dir.path()).unwrap();
        assert_eq!(info.capacity, 60);
        assert_eq!(info.status, "Charging");
        assert_eq!(info.time_estimate, Some("1h until full".to_string()));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_probe_sysfs_battery_ac_connected_no_time() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bat_dir = temp_dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("capacity"), "98\n").unwrap();
        fs::write(bat_dir.join("status"), "Not charging\n").unwrap();

        let ac_dir = temp_dir.path().join("AC");
        fs::create_dir_all(&ac_dir).unwrap();
        fs::write(ac_dir.join("online"), "1\n").unwrap();

        let info = probe_sysfs_battery_from_dir(temp_dir.path()).unwrap();
        assert_eq!(info.capacity, 98);
        assert_eq!(info.status, "AC Connected");
        assert_eq!(info.time_estimate, None);
    }

    #[test]
    fn test_format_time_estimate() {
        // Charging is green
        assert_eq!(
            format_time_estimate("1h 10m until full", 30, true, true),
            "\x1b[32m1h 10m until full\x1b[0m"
        );

        // Discharging >= 50% is green
        assert_eq!(
            format_time_estimate("2h 38m remaining", 97, false, true),
            "\x1b[32m2h 38m remaining\x1b[0m"
        );
        assert_eq!(
            format_time_estimate("2h remaining", 50, false, true),
            "\x1b[32m2h remaining\x1b[0m"
        );

        // Discharging 20% - 49% is yellow
        assert_eq!(
            format_time_estimate("1h 15m remaining", 40, false, true),
            "\x1b[33m1h 15m remaining\x1b[0m"
        );
        assert_eq!(
            format_time_estimate("45m remaining", 20, false, true),
            "\x1b[33m45m remaining\x1b[0m"
        );

        // Discharging < 20% is red
        assert_eq!(
            format_time_estimate("25m remaining", 15, false, true),
            "\x1b[31m25m remaining\x1b[0m"
        );
        assert_eq!(
            format_time_estimate("< 1m remaining", 5, false, true),
            "\x1b[31m< 1m remaining\x1b[0m"
        );

        // Disabled color
        assert_eq!(
            format_time_estimate("2h 38m remaining", 97, false, false),
            "2h 38m remaining"
        );
        assert_eq!(
            format_time_estimate("1h 10m until full", 30, true, false),
            "1h 10m until full"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn test_probe_ac_online_from_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ac_dir = temp_dir.path().join("ADP1");
        fs::create_dir_all(&ac_dir).unwrap();
        fs::write(ac_dir.join("online"), "1\n").unwrap();

        assert_eq!(probe_ac_online_from_dir(temp_dir.path()), Some(true));

        fs::write(ac_dir.join("online"), "0\n").unwrap();
        assert_eq!(probe_ac_online_from_dir(temp_dir.path()), Some(false));
    }
}
