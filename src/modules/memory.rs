//! Physical RAM & Virtual Swap Space Collector
//!
//! Parses `/proc/meminfo` and `/proc/swaps` on Linux and `GlobalMemoryStatusEx` on Windows.
//! Automatically discovers in-memory ZRAM compression algorithms (`LZ4`, `ZSTD`, `LZO`), suggested by @Laynsb.

use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub percent: u64,
}

/// Parses `/proc/meminfo` to calculate total and active used RAM.
/// On Linux kernels >= 3.14, `MemAvailable` provides the kernel's accurate estimate of available memory
/// without triggering swapping (accounting for page cache and `SReclaimable` slab while reserving watermarks).
pub fn parse_meminfo(content: &str) -> Option<MemoryInfo> {
    let mut mem_total: Option<u64> = None;
    let mut mem_available: Option<u64> = None;
    let mut mem_free: Option<u64> = None;
    let mut buffers: Option<u64> = None;
    let mut cached: Option<u64> = None;
    let mut sreclaimable: Option<u64> = None;
    let mut shmem: Option<u64> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            let val_str = v.split_whitespace().next().unwrap_or("");
            let val: u64 = val_str.parse().unwrap_or(0);

            match key {
                "MemTotal" => mem_total = Some(val),
                "MemAvailable" => mem_available = Some(val),
                "MemFree" => mem_free = Some(val),
                "Buffers" => buffers = Some(val),
                "Cached" => cached = Some(val),
                "SReclaimable" => sreclaimable = Some(val),
                "Shmem" => shmem = Some(val),
                _ => {}
            }
        }
    }

    let total = mem_total?;
    if total == 0 {
        return None;
    }

    let used = if let Some(avail) = mem_available {
        total.saturating_sub(avail)
    } else {
        // Legacy fallback for Linux < 3.14 lacking MemAvailable
        let free = mem_free.unwrap_or(0);
        let buf = buffers.unwrap_or(0);
        let cach = cached.unwrap_or(0);
        let srec = sreclaimable.unwrap_or(0);
        let shm = shmem.unwrap_or(0);

        let non_used = free + buf + cach + srec;
        total.saturating_sub(non_used) + shm
    };

    let used = used.min(total);
    let percent = ((used as f64 / total as f64) * 100.0).round().min(100.0) as u64;

    Some(MemoryInfo {
        total_kb: total,
        used_kb: used,
        percent,
    })
}

/// Formats memory in GiB or MiB.
pub fn format_memory(info: &MemoryInfo, enable_color: bool) -> String {
    let one_gib_kb = 1024.0 * 1024.0;
    let pct = crate::output::color::format_percentage(info.percent, false, enable_color);
    if info.total_kb as f64 >= one_gib_kb {
        let used_gib = info.used_kb as f64 / one_gib_kb;
        let total_gib = info.total_kb as f64 / one_gib_kb;
        format!("{:.2} GiB / {:.2} GiB ({})", used_gib, total_gib, pct)
    } else {
        let used_mib = info.used_kb as f64 / 1024.0;
        let total_mib = info.total_kb as f64 / 1024.0;
        format!("{:.0} MiB / {:.0} MiB ({})", used_mib, total_mib, pct)
    }
}

#[cfg(not(windows))]
pub fn get_memory_info() -> Option<MemoryInfo> {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        return parse_meminfo(&content);
    }
    None
}

#[cfg(windows)]
pub fn get_memory_info() -> Option<MemoryInfo> {
    use crate::modules::win_util::ffi;
    unsafe {
        let mut status = std::mem::zeroed::<ffi::MEMORYSTATUSEX>();
        status.dwLength = std::mem::size_of::<ffi::MEMORYSTATUSEX>() as u32;
        if ffi::GlobalMemoryStatusEx(&mut status) != 0 {
            let total_kb = status.ullTotalPhys / 1024;
            let avail_kb = status.ullAvailPhys / 1024;
            if total_kb > 0 {
                let used_kb = total_kb.saturating_sub(avail_kb);
                let percent = ((used_kb as f64 / total_kb as f64) * 100.0)
                    .round()
                    .min(100.0) as u64;
                return Some(MemoryInfo {
                    total_kb,
                    used_kb,
                    percent,
                });
            }
        }
    }
    None
}

pub struct MemoryCollector;

impl Collector for MemoryCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Memory
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        let mem = get_memory_info()?;
        Some(ModuleOutput {
            id: ModuleId::Memory,
            label: "Memory".to_string(),
            value: format_memory(&mem, ctx.enable_color),
            custom_rendered: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub percent: u64,
}

pub fn parse_swapinfo(content: &str) -> Option<SwapInfo> {
    let mut swap_total: Option<u64> = None;
    let mut swap_free: Option<u64> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            let val_str = v.split_whitespace().next().unwrap_or("");
            let val: u64 = val_str.parse().unwrap_or(0);

            match key {
                "SwapTotal" => swap_total = Some(val),
                "SwapFree" => swap_free = Some(val),
                _ => {}
            }
        }
    }

    let total = swap_total?;
    if total == 0 {
        return None;
    }

    let free = swap_free.unwrap_or(total);
    let used = total.saturating_sub(free);
    let percent = ((used as f64 / total as f64) * 100.0).round().min(100.0) as u64;

    Some(SwapInfo {
        total_kb: total,
        used_kb: used,
        percent,
    })
}

#[cfg(not(windows))]
pub fn get_swap_info() -> Option<SwapInfo> {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        return parse_swapinfo(&content);
    }
    None
}

/// Parses the active ZRAM compression algorithm from `/proc/swaps` and `/sys/block/zram*/comp_algorithm`.
/// Algorithmic string format: `lzo lzo-rle [lz4] lz4hc zstd` where bracketed entry is active.
#[cfg(not(windows))]
pub fn detect_zram_algorithm() -> Option<String> {
    let swaps = fs::read_to_string("/proc/swaps").ok()?;
    if !swaps.contains("zram") {
        return None;
    }

    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.starts_with("zram") {
                let comp_path = entry.path().join("comp_algorithm");
                if let Ok(content) = fs::read_to_string(comp_path) {
                    if let Some(start) = content.find('[') {
                        if let Some(end) = content[start..].find(']') {
                            let active = &content[start + 1..start + end];
                            let trimmed = active.trim();
                            if !trimmed.is_empty() {
                                return Some(trimmed.to_uppercase());
                            }
                        }
                    }
                    if let Some(first) = content.split_whitespace().next() {
                        let trimmed = first.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_uppercase());
                        }
                    }
                }
            }
        }
    }

    Some("ZRAM".to_string())
}

#[cfg(windows)]
pub fn detect_zram_algorithm() -> Option<String> {
    None
}

#[cfg(windows)]
pub fn get_swap_info() -> Option<SwapInfo> {
    use crate::modules::win_util::ffi;
    unsafe {
        let mut status = std::mem::zeroed::<ffi::MEMORYSTATUSEX>();
        status.dwLength = std::mem::size_of::<ffi::MEMORYSTATUSEX>() as u32;
        if ffi::GlobalMemoryStatusEx(&mut status) != 0 {
            let total_page_kb = status.ullTotalPageFile / 1024;
            let total_phys_kb = status.ullTotalPhys / 1024;
            let avail_page_kb = status.ullAvailPageFile / 1024;

            let swap_total_kb = total_page_kb.saturating_sub(total_phys_kb);
            if swap_total_kb > 0 {
                let total_avail_kb = status.ullAvailPhys / 1024;
                let swap_avail_kb = avail_page_kb.saturating_sub(total_avail_kb);
                let swap_used_kb = swap_total_kb
                    .saturating_sub(swap_avail_kb)
                    .min(swap_total_kb);
                let percent = ((swap_used_kb as f64 / swap_total_kb as f64) * 100.0)
                    .round()
                    .min(100.0) as u64;
                return Some(SwapInfo {
                    total_kb: swap_total_kb,
                    used_kb: swap_used_kb,
                    percent,
                });
            }
        }
    }
    None
}

/// Formats swap memory with optional ZRAM compression algorithm tag.
pub fn format_swap(info: &SwapInfo, zram_algo: Option<&str>, enable_color: bool) -> String {
    let one_gib_kb = 1024.0 * 1024.0;
    let pct = crate::output::color::format_percentage(info.percent, false, enable_color);
    let base = if info.total_kb as f64 >= one_gib_kb {
        let used_gib = info.used_kb as f64 / one_gib_kb;
        let total_gib = info.total_kb as f64 / one_gib_kb;
        format!("{:.2} GiB / {:.2} GiB ({})", used_gib, total_gib, pct)
    } else {
        let used_mib = info.used_kb as f64 / 1024.0;
        let total_mib = info.total_kb as f64 / 1024.0;
        format!("{:.0} MiB / {:.0} MiB ({})", used_mib, total_mib, pct)
    };

    if let Some(algo) = zram_algo {
        if !algo.is_empty() {
            return format!("{} - {}", base, algo);
        }
    }
    base
}

pub struct SwapCollector;

impl Collector for SwapCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Swap
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        let swap = get_swap_info()?;
        let zram_algo = detect_zram_algorithm();
        let value = format_swap(&swap, zram_algo.as_deref(), ctx.enable_color);

        Some(ModuleOutput {
            id: ModuleId::Swap,
            label: "Swap".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_meminfo_standard() {
        let fixture = r#"
MemTotal:       16281600 kB
MemFree:         4385100 kB
MemAvailable:   11550000 kB
Buffers:          345600 kB
Cached:          7450000 kB
"#;
        let info = parse_meminfo(fixture).unwrap();
        assert_eq!(info.total_kb, 16281600);
        assert_eq!(info.used_kb, 16281600 - 11550000);
        assert_eq!(info.percent, 29);
        let formatted = format_memory(&info, false);
        assert!(formatted.contains("GiB"));
        assert!(formatted.contains("29%"));

        let colored = format_memory(&info, true);
        assert!(colored.contains("\x1b[32m29%\x1b[0m"));
    }

    #[test]
    fn test_parse_meminfo_under_1gib() {
        let fixture = r#"
MemTotal:         512000 kB
MemFree:          100000 kB
MemAvailable:     300000 kB
"#;
        let info = parse_meminfo(fixture).unwrap();
        let formatted = format_memory(&info, false);
        assert!(formatted.contains("MiB"));
    }

    #[test]
    fn test_parse_meminfo_empty_or_malformed() {
        assert_eq!(parse_meminfo(""), None);
        assert_eq!(parse_meminfo("   \n\n\t "), None);
        assert_eq!(parse_meminfo("SomeGarbageLine: without total"), None);
        assert_eq!(parse_meminfo("MemTotal: 0 kB"), None);
    }

    #[test]
    fn test_parse_meminfo_legacy_fallback() {
        let fixture = r#"
MemTotal:        8192000 kB
MemFree:         1024000 kB
Buffers:          512000 kB
Cached:          3072000 kB
SReclaimable:     512000 kB
Shmem:            256000 kB
"#;
        let info = parse_meminfo(fixture).unwrap();
        assert_eq!(info.total_kb, 8192000);
        assert!(info.used_kb > 0);
        assert!(info.percent > 0);
    }

    #[test]
    fn test_parse_swapinfo() {
        let fixture = r#"
SwapTotal:       4194304 kB
SwapFree:        4194304 kB
"#;
        let info = parse_swapinfo(fixture).unwrap();
        assert_eq!(info.total_kb, 4194304);
        assert_eq!(info.used_kb, 0);
        assert_eq!(info.percent, 0);

        let formatted = format_swap(&info, Some("LZ4"), false);
        assert_eq!(formatted, "0.00 GiB / 4.00 GiB (0%) - LZ4");

        let formatted_traditional = format_swap(&info, None, false);
        assert_eq!(formatted_traditional, "0.00 GiB / 4.00 GiB (0%)");

        let formatted_colored = format_swap(&info, Some("LZ4"), true);
        assert_eq!(
            formatted_colored,
            "0.00 GiB / 4.00 GiB (\x1b[32m0%\x1b[0m) - LZ4"
        );
    }
}
