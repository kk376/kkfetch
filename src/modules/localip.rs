use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(unix)]
use std::ffi::CStr;
#[cfg(unix)]
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalIpEntry {
    pub label: String,
    pub ip: String,
    pub cidr: u8,
}

/// Classifies a network interface name into a clean user-facing category.
pub fn classify_interface_label(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.starts_with("wl")
        || lower.contains("wlan")
        || lower.contains("wifi")
        || lower.contains("wi-fi")
    {
        "Local IP (Wi-Fi)"
    } else if lower.starts_with("en")
        || lower.starts_with("eth")
        || lower.starts_with("em")
        || lower.contains("ethernet")
    {
        "Local IP (Ethernet)"
    } else if lower.starts_with("ww")
        || lower.contains("cellular")
        || lower.contains("mobile")
        || lower.contains("lte")
    {
        "Local IP (Cellular)"
    } else if lower.starts_with("lo") || lower.contains("loopback") {
        "Local IP (Loopback)"
    } else {
        "Local IP"
    }
}

/// Probes all active local IPv4 addresses and categorizes them with CIDR netmask notation.
pub fn detect_local_ips() -> Vec<LocalIpEntry> {
    #[cfg(unix)]
    // SAFETY: libc::getifaddrs safely allocates a linked list of network interfaces. We safely iterate over it and free it with libc::freeifaddrs.
    unsafe {
        let mut ifaddrs: *mut libc::ifaddrs = std::ptr::null_mut();
        if libc::getifaddrs(&mut ifaddrs) == 0 && !ifaddrs.is_null() {
            let mut physical_entries = Vec::new();
            let mut loopback_entry: Option<LocalIpEntry> = None;
            let mut fallback_virtual: Option<LocalIpEntry> = None;

            let mut curr = ifaddrs;
            while !curr.is_null() {
                let ifa = *curr;
                if !ifa.ifa_addr.is_null()
                    && (*ifa.ifa_addr).sa_family == libc::AF_INET as libc::sa_family_t
                {
                    let name = CStr::from_ptr(ifa.ifa_name).to_string_lossy();
                    let flags = ifa.ifa_flags as i32;

                    let is_up = (flags & libc::IFF_UP) != 0;
                    let is_loopback = (flags & libc::IFF_LOOPBACK) != 0;

                    if is_up {
                        let sin = ifa.ifa_addr as *const libc::sockaddr_in;
                        let ip_raw = u32::from_be((*sin).sin_addr.s_addr);
                        let ip = Ipv4Addr::from(ip_raw);

                        let cidr = if !ifa.ifa_netmask.is_null() {
                            let sin_mask = ifa.ifa_netmask as *const libc::sockaddr_in;
                            let mask_raw = u32::from_be((*sin_mask).sin_addr.s_addr);
                            mask_raw.count_ones() as u8
                        } else {
                            24
                        };

                        if is_loopback || ip.is_loopback() {
                            if loopback_entry.is_none() {
                                loopback_entry = Some(LocalIpEntry {
                                    label: "Local IP (Loopback)".to_string(),
                                    ip: ip.to_string(),
                                    cidr,
                                });
                            }
                        } else if !ip.is_link_local() {
                            let label = classify_interface_label(&name).to_string();
                            let is_virtual = name.starts_with("docker")
                                || name.starts_with("veth")
                                || name.starts_with("virbr")
                                || name.starts_with("br-");

                            if !is_virtual {
                                if !physical_entries
                                    .iter()
                                    .any(|e: &LocalIpEntry| e.ip == ip.to_string())
                                {
                                    physical_entries.push(LocalIpEntry {
                                        label,
                                        ip: ip.to_string(),
                                        cidr,
                                    });
                                }
                            } else if fallback_virtual.is_none() {
                                fallback_virtual = Some(LocalIpEntry {
                                    label,
                                    ip: ip.to_string(),
                                    cidr,
                                });
                            }
                        }
                    }
                }
                curr = ifa.ifa_next;
            }

            libc::freeifaddrs(ifaddrs);

            if !physical_entries.is_empty() {
                return physical_entries;
            } else if let Some(virt) = fallback_virtual {
                return vec![virt];
            } else if let Some(lb) = loopback_entry {
                return vec![lb];
            }
        }
    }

    // Cross-platform UDP routing table query fallback (works on Windows, Linux, macOS)
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                let ip = local_addr.ip();
                if !ip.is_loopback() && !ip.is_unspecified() {
                    return vec![LocalIpEntry {
                        label: "Local IP".to_string(),
                        ip: ip.to_string(),
                        cidr: 24,
                    }];
                }
            }
        }
    }

    Vec::new()
}

/// Retrieves the primary local IPv4 address.
pub fn detect_local_ip() -> Option<String> {
    detect_local_ips().into_iter().next().map(|e| e.ip)
}

pub struct LocalIpCollector;

impl Collector for LocalIpCollector {
    fn id(&self) -> ModuleId {
        ModuleId::LocalIp
    }

    fn collect_multiple(&self, _ctx: &FetchContext) -> Vec<ModuleOutput> {
        let entries = detect_local_ips();
        entries
            .into_iter()
            .map(|e| ModuleOutput {
                id: ModuleId::LocalIp,
                label: e.label,
                value: format!("{}/{}", e.ip, e.cidr),
                custom_rendered: None,
            })
            .collect()
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        self.collect_multiple(ctx).into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_interface_label() {
        assert_eq!(classify_interface_label("wlp4s0"), "Local IP (Wi-Fi)");
        assert_eq!(classify_interface_label("wlan0"), "Local IP (Wi-Fi)");
        assert_eq!(classify_interface_label("enp3s0"), "Local IP (Ethernet)");
        assert_eq!(classify_interface_label("eth0"), "Local IP (Ethernet)");
        assert_eq!(classify_interface_label("wwan0"), "Local IP (Cellular)");
        assert_eq!(classify_interface_label("lo"), "Local IP (Loopback)");
    }

    #[test]
    fn test_detect_local_ips_live() {
        let ips = detect_local_ips();
        for entry in &ips {
            assert!(!entry.ip.is_empty());
            assert!(entry.cidr > 0 && entry.cidr <= 32);
            assert!(entry.label.starts_with("Local IP"));
        }
    }
}
