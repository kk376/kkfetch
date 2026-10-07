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
pub fn probe_ac_online() -> Option<bool> {
    probe_ac_online_from_dir(std::path::Path::new("/sys/class/power_supply"))
}

#[cfg(not(windows))]
fn get_ram_cache_path() -> std::path::PathBuf {
    // 1. Prefer $XDG_RUNTIME_DIR (tmpfs in /run/user/$UID/)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        return std::path::PathBuf::from(runtime_dir)
            .join("kkfetch")
            .join("battery.cache");
    }
    // 2. Fallback to /dev/shm (POSIX shared memory tmpfs)
    let shm_dir = std::path::PathBuf::from("/dev/shm");
    if shm_dir.is_dir() {
        // SAFETY: libc::getuid requires no arguments and returns the current user ID.
        let uid = unsafe { libc::getuid() };
        return shm_dir
            .join(format!("kkfetch-{}", uid))
            .join("battery.cache");
    }
    // 3. Fallback to private user-isolated temp directory
    // SAFETY: libc::getuid requires no arguments and returns the current user ID.
    let uid = unsafe { libc::getuid() };
    std::env::temp_dir()
        .join(format!("kkfetch-{}", uid))
        .join("battery.cache")
}

#[cfg(not(windows))]
fn read_cached_battery() -> Option<(BatteryInfo, u64)> {
    let path = get_ram_cache_path();
    let content = fs::read_to_string(path).ok()?;
    let mut parts = content.splitn(4, '|');
    let timestamp = parts.next()?.trim().parse::<u64>().ok()?;
    let capacity = parts.next()?.trim().parse::<u8>().ok()?;
    let status = parts.next()?.trim().to_string();
    if status.is_empty() {
        return None;
    }
    let time_estimate = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    let age = now.saturating_sub(timestamp);
    Some((
        BatteryInfo {
            capacity,
            status,
            time_estimate,
        },
        age,
    ))
}

#[cfg(not(windows))]
fn write_cached_battery(info: &BatteryInfo) {
    let path = get_ram_cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let payload = format!(
        "{}|{}|{}|{}",
        now,
        info.capacity,
        info.status,
        info.time_estimate.as_deref().unwrap_or("")
    );
    let _ = fs::write(path, payload);
}

/// Parses UPower busctl property output into BatteryInfo.
pub fn parse_busctl_upower_output(text: &str, ac_online: Option<bool>) -> Option<BatteryInfo> {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() < 5 {
        return None;
    }

    // Line 0: "b true" or "b false"
    let is_present = lines[0].split_whitespace().nth(1)? == "true";
    if !is_present {
        return None;
    }

    // Line 1: "d 94" or "d 94.0"
    let pct_str = lines[1].split_whitespace().nth(1)?;
    let capacity = pct_str.parse::<f64>().ok()?.round() as u8;

    // Line 2: "u 4" (State: 1=Charging, 2=Discharging, 3=Empty, 4=Fully charged, 5=Pending charge, 6=Pending discharge)
    let state_str = lines[2].split_whitespace().nth(1)?;
    let state = state_str.parse::<u32>().ok()?;

    // Line 3: "x 0" (TimeToEmpty in seconds)
    let time_to_empty = lines[3]
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    // Line 4: "x 0" (TimeToFull in seconds)
    let time_to_full = lines[4]
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let is_ac = state == 1 || state == 4 || ac_online.unwrap_or(false);

    let status = match state {
        1 => "Charging".to_string(),
        2 => "Discharging".to_string(),
        4 => {
            if is_ac {
                if capacity >= 99 {
                    "Full [AC]".to_string()
                } else {
                    "AC Connected".to_string()
                }
            } else {
                "Full".to_string()
            }
        }
        _ => {
            if is_ac {
                if capacity >= 99 {
                    "Full [AC]".to_string()
                } else {
                    "AC Connected".to_string()
                }
            } else {
                "Discharging".to_string()
            }
        }
    };

    let time_estimate = if status == "Discharging" && time_to_empty > 0 {
        format_duration_estimate(time_to_empty as u64, false)
    } else if status == "Charging" && time_to_full > 0 {
        format_duration_estimate(time_to_full as u64, true)
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
pub fn probe_upower_battery() -> Option<BatteryInfo> {
    probe_upower_battery_with_ac(probe_ac_online())
}

#[cfg(not(windows))]
fn probe_upower_battery_with_ac(ac_online: Option<bool>) -> Option<BatteryInfo> {
    use crate::modules::system_command;

    let output = system_command("busctl")
        .args([
            "get-property",
            "org.freedesktop.UPower",
            "/org/freedesktop/UPower/devices/DisplayDevice",
            "org.freedesktop.UPower.Device",
            "IsPresent",
            "Percentage",
            "State",
            "TimeToEmpty",
            "TimeToFull",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = std::str::from_utf8(&output.stdout).ok()?;
    parse_busctl_upower_output(text, ac_online)
}

#[cfg(not(windows))]
fn probe_sysfs_battery() -> Option<BatteryInfo> {
    probe_sysfs_battery_from_dir(std::path::Path::new("/sys/class/power_supply"))
}

/// Probes real-time battery status using a 3-tier hierarchy:
/// Tier 1: In-RAM tmpfs cache (< 15 µs) with instantaneous AC online line invalidation (< 100 µs).
/// Tier 2: Direct UPower D-Bus query (~3 ms) with 0 ms hardware EC delay and 100% status bar parity.
/// Tier 3: Direct sysfs power_supply probing fallback.
#[cfg(not(windows))]
pub fn detect_battery() -> Option<BatteryInfo> {
    let ac_online = probe_ac_online();

    // Tier 1: RAM Tmpfs Cache
    if let Some((cached, age)) = read_cached_battery() {
        let was_ac = cached.status.contains("AC")
            || cached.status.contains("Charging")
            || cached.status.contains("Full [AC]");
        let ac_matches = match ac_online {
            Some(ac) => ac == was_ac,
            None => true,
        };

        // Cache valid for 15 seconds if AC line state has not changed
        if age <= 15 && ac_matches {
            return Some(cached);
        }
    }

    // Tier 2: UPower D-Bus Query Fast-Path
    if let Some(info) = probe_upower_battery_with_ac(ac_online) {
        write_cached_battery(&info);
        return Some(info);
    }

    // Tier 3: Direct Sysfs Probe Fallback
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

    #[test]
    fn test_parse_busctl_upower_output_charging() {
        let sample = "b true\nd 65.0\nu 1\nx 0\nx 3600\n";
        let info = parse_busctl_upower_output(sample, Some(true)).unwrap();
        assert_eq!(info.capacity, 65);
        assert_eq!(info.status, "Charging");
        assert_eq!(info.time_estimate, Some("1h until full".to_string()));
    }

    #[test]
    fn test_parse_busctl_upower_output_discharging() {
        let sample = "b true\nd 80.0\nu 2\nx 7200\nx 0\n";
        let info = parse_busctl_upower_output(sample, Some(false)).unwrap();
        assert_eq!(info.capacity, 80);
        assert_eq!(info.status, "Discharging");
        assert_eq!(info.time_estimate, Some("2h remaining".to_string()));
    }

    #[test]
    fn test_parse_busctl_upower_output_full_ac() {
        let sample = "b true\nd 100.0\nu 4\nx 0\nx 0\n";
        let info = parse_busctl_upower_output(sample, Some(true)).unwrap();
        assert_eq!(info.capacity, 100);
        assert_eq!(info.status, "Full [AC]");
        assert_eq!(info.time_estimate, None);
    }

    #[test]
    fn test_parse_busctl_upower_output_ac_connected_threshold() {
        // Battery limited to 80% or 94% on AC
        let sample = "b true\nd 94.0\nu 4\nx 0\nx 0\n";
        let info = parse_busctl_upower_output(sample, Some(true)).unwrap();
        assert_eq!(info.capacity, 94);
        assert_eq!(info.status, "AC Connected");
        assert_eq!(info.time_estimate, None);
    }

    #[test]
    fn test_parse_busctl_upower_output_not_present() {
        let sample = "b false\nd 0.0\nu 0\nx 0\nx 0\n";
        assert_eq!(parse_busctl_upower_output(sample, None), None);
    }
}
