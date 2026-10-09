use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfo {
    pub name: Option<String>,
    pub resolution: String,
    pub refresh_rate: Option<u32>,
    pub size_inches: Option<u32>,
    pub display_type: Option<String>,
    pub scale: Option<f64>,
}

/// Parses raw 128-byte EDID binary block into structured DisplayInfo.
pub fn parse_edid_binary(data: &[u8], connector_name: &str) -> Option<DisplayInfo> {
    if data.len() < 128 {
        return None;
    }
    // Verify standard EDID header magic (00 FF FF FF FF FF FF 00)
    if data[0..8] != [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00] {
        return None;
    }

    // 1. Manufacturer Code (PNP ID)
    let mfg_id_raw = ((data[8] as u16) << 8) | (data[9] as u16);
    let c1 = (((mfg_id_raw >> 10) & 0x1F) as u8 + b'A' - 1) as char;
    let c2 = (((mfg_id_raw >> 5) & 0x1F) as u8 + b'A' - 1) as char;
    let c3 = ((mfg_id_raw & 0x1F) as u8 + b'A' - 1) as char;
    let prod_code = (data[10] as u16) | ((data[11] as u16) << 8);
    let mfg_code = format!("{}{}{}{:04X}", c1, c2, c3, prod_code);

    let mut name = Some(mfg_code);

    // 2. Physical screen size in inches from byte 21 (w_cm) and byte 22 (h_cm)
    let w_cm = data[21] as f64;
    let h_cm = data[22] as f64;
    let size_inches = if w_cm > 0.0 && h_cm > 0.0 {
        let diag_cm = (w_cm * w_cm + h_cm * h_cm).sqrt();
        let inches = (diag_cm / 2.54).round() as u32;
        if inches > 0 {
            Some(inches)
        } else {
            None
        }
    } else {
        None
    };

    // 3. Scan 4 descriptor blocks (offsets 54, 72, 90, 108) for timings and monitor name
    let mut best_hz: Option<u32> = None;
    let mut best_res: Option<String> = None;

    for &offset in &[54, 72, 90, 108] {
        if offset + 18 > data.len() {
            break;
        }
        let block = &data[offset..offset + 18];
        if block[0] == 0x00 && block[1] == 0x00 {
            let tag = block[3];
            if tag == 0xFC || tag == 0xFE {
                let text_bytes = &block[5..18];
                let mut text = String::with_capacity(13);
                for &b in text_bytes {
                    if b == b'\n' || b == b'\r' || b == 0 {
                        break;
                    }
                    if b.is_ascii_graphic() || b == b' ' {
                        text.push(b as char);
                    }
                }
                let clean = text.trim();
                if !clean.is_empty() && (tag == 0xFC || name.is_none()) {
                    name = Some(clean.to_string());
                }
            }
        } else {
            let pixel_clock_10khz = (block[0] as u32) | ((block[1] as u32) << 8);
            let h_active = (block[2] as u32) | (((block[4] >> 4) as u32) << 8);
            let h_blank = (block[3] as u32) | (((block[4] & 0x0F) as u32) << 8);
            let v_active = (block[5] as u32) | (((block[7] >> 4) as u32) << 8);
            let v_blank = (block[6] as u32) | (((block[7] & 0x0F) as u32) << 8);

            let h_total = h_active + h_blank;
            let v_total = v_active + v_blank;

            if pixel_clock_10khz > 0 && h_total > 0 && v_total > 0 && h_active > 0 && v_active > 0 {
                let refresh = ((pixel_clock_10khz as f64 * 10000.0)
                    / (h_total as f64 * v_total as f64))
                    .round() as u32;
                let res = format!("{}x{}", h_active, v_active);
                match best_hz {
                    None => {
                        best_hz = Some(refresh);
                        best_res = Some(res);
                    }
                    Some(cur) => {
                        if refresh > cur {
                            best_hz = Some(refresh);
                            best_res = Some(res);
                        }
                    }
                }
            }
        }
    }

    let (res, hz) = match (best_res, best_hz) {
        (Some(r), h) => (r, h),
        (None, _) => ("1920x1080".to_string(), None),
    };

    // 5. Display type [Built-in] vs [External]
    let conn_lower = connector_name.to_lowercase();
    let display_type = if conn_lower.contains("edp")
        || conn_lower.contains("lvds")
        || conn_lower.contains("dsi")
    {
        Some("[Built-in]".to_string())
    } else if conn_lower.contains("hdmi")
        || conn_lower.contains("dp")
        || conn_lower.contains("vga")
        || conn_lower.contains("dvi")
    {
        Some("[External]".to_string())
    } else {
        None
    };

    Some(DisplayInfo {
        name,
        resolution: res,
        refresh_rate: hz,
        size_inches,
        display_type,
        scale: None,
    })
}

fn extract_xml_scale(text: &str) -> Option<f64> {
    let (s_start, s_end) = (text.find("<scale>")?, text.find("</scale>")?);
    let s_val = text[s_start + 7..s_end].trim();
    s_val.parse::<f64>().ok().filter(|&scale| scale > 0.0)
}

/// Parses display scaling factor from GNOME / Mutter monitors.xml.
pub fn parse_monitors_xml_scale(content: &str, connector: Option<&str>) -> Option<f64> {
    let conn_suffix = connector.and_then(|c| c.split('-').next_back());

    for block in content.split("<logicalmonitor>") {
        if !block.contains("</logicalmonitor>") {
            continue;
        }
        let lm = block.split("</logicalmonitor>").next().unwrap_or("");
        let matched = match (connector, conn_suffix) {
            (Some(c), Some(s)) => lm.contains(c) || lm.contains(s),
            (Some(c), None) => lm.contains(c),
            (None, _) => true,
        };

        if matched {
            if let Some(scale) = extract_xml_scale(lm) {
                return Some(scale);
            }
        }
    }

    extract_xml_scale(content)
}

/// Parses display scaling factor from `hyprctl monitors` or `hyprctl -j monitors` output.
pub fn parse_hyprctl_scale(content: &str, connector: Option<&str>) -> Option<f64> {
    let conn_name = connector.map(|c| {
        if let Some(pos) = c.find('-') {
            &c[pos + 1..]
        } else {
            c
        }
    });

    let mut in_monitor = connector.is_none();

    for line in content.lines() {
        let trimmed = line.trim();

        // 1. Plain text format: "Monitor eDP-1 (ID 0):"
        if let Some(rest) = trimmed.strip_prefix("Monitor ") {
            if let Some(mon_name) = rest.split_whitespace().next() {
                if let Some(conn) = connector {
                    in_monitor = mon_name == conn || conn_name == Some(mon_name);
                } else {
                    in_monitor = true;
                }
            }
        } else if trimmed.starts_with("\"name\":") {
            // 2. JSON format: "\"name\": \"eDP-1\","
            // Only consider top-level monitor names (exclude nested workspace objects)
            if !line.starts_with("        ") {
                let name_val = trimmed
                    .trim_start_matches("\"name\":")
                    .trim()
                    .trim_matches(|c| c == '"' || c == ',' || c == ' ');
                if let Some(conn) = connector {
                    in_monitor = name_val == conn || conn_name == Some(name_val);
                } else {
                    in_monitor = true;
                }
            }
        }

        if in_monitor {
            if let Some(rest) = trimmed.strip_prefix("scale:") {
                if let Ok(scale) = rest.trim().parse::<f64>() {
                    if scale > 0.0 {
                        return Some(scale);
                    }
                }
            } else if let Some(rest) = trimmed.strip_prefix("\"scale\":") {
                let trimmed_num = rest.trim_start();
                let end = trimmed_num
                    .find(|c: char| !c.is_ascii_digit() && c != '.')
                    .unwrap_or(trimmed_num.len());
                if let Ok(scale) = trimmed_num[..end].parse::<f64>() {
                    if scale > 0.0 {
                        return Some(scale);
                    }
                }
            }
        }
    }

    None
}

/// Parses display scaling factor from `xrdb -query` output (e.g. `Xft.dpi: 144`).
pub fn parse_xrdb_scale(content: &str) -> Option<f64> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Xft.dpi:") {
            if let Ok(dpi) = rest.trim().parse::<f64>() {
                if dpi > 0.0 {
                    let scale = dpi / 96.0;
                    return Some(scale);
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
static HYPRLAND_MONITORS_CACHE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

#[cfg(not(windows))]
fn query_hyprland_monitors_socket() -> Option<String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let sock_path = format!("{}/hypr/{}/.socket.sock", runtime_dir, sig);

    let mut stream = UnixStream::connect(sock_path).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_millis(50)))
        .ok()?;
    stream
        .set_write_timeout(Some(Duration::from_millis(50)))
        .ok()?;
    stream.write_all(b"j/monitors").ok()?;

    let mut response = Vec::with_capacity(4096);
    stream.read_to_end(&mut response).ok()?;
    String::from_utf8(response).ok()
}

#[cfg(not(windows))]
fn get_hyprland_monitors_json() -> Option<&'static str> {
    HYPRLAND_MONITORS_CACHE
        .get_or_init(query_hyprland_monitors_socket)
        .as_deref()
}

#[cfg(not(windows))]
static SWAY_OUTPUTS_CACHE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

#[cfg(not(windows))]
fn query_sway_outputs_socket() -> Option<String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let sock_path = std::env::var_os("SWAYSOCK")?;
    let mut stream = UnixStream::connect(sock_path).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_millis(50)))
        .ok()?;
    stream
        .set_write_timeout(Some(Duration::from_millis(50)))
        .ok()?;

    // i3 and Sway IPC wire protocol: "i3-ipc" followed by payload length (u32 LE) and type (u32 LE 4 = IPC_GET_OUTPUTS)
    let mut msg = Vec::with_capacity(14);
    msg.extend_from_slice(b"i3-ipc");
    msg.extend_from_slice(&0u32.to_le_bytes());
    msg.extend_from_slice(&4u32.to_le_bytes());
    stream.write_all(&msg).ok()?;

    let mut header = [0u8; 14];
    stream.read_exact(&mut header).ok()?;
    if &header[..6] != b"i3-ipc" {
        return None;
    }
    let payload_len = u32::from_le_bytes([header[6], header[7], header[8], header[9]]) as usize;
    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload).ok()?;
    String::from_utf8(payload).ok()
}

#[cfg(not(windows))]
fn get_sway_outputs_json() -> Option<&'static str> {
    SWAY_OUTPUTS_CACHE
        .get_or_init(query_sway_outputs_socket)
        .as_deref()
}

/// Detects display scaling factor from active compositor, GNOME monitors.xml, KDE config, or environment variables.
pub fn detect_display_scale(connector: Option<&str>) -> Option<f64> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let is_hyprland =
        desktop.contains("hyprland") || std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some();

    // 1. Hyprland: query direct Unix socket if active, fallback to hyprctl
    if is_hyprland {
        #[cfg(not(windows))]
        if let Some(text) = get_hyprland_monitors_json() {
            if let Some(scale) = parse_hyprctl_scale(text, connector) {
                return Some(scale);
            }
        }

        if let Ok(output) = crate::modules::system_command("hyprctl")
            .args(["-j", "monitors"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(scale) = parse_hyprctl_scale(&text, connector) {
                    return Some(scale);
                }
            }
        }
    }

    // 2. Sway / wlroots: query direct Sway IPC socket if active, fallback to swaymsg
    let is_sway = desktop.contains("sway") || std::env::var_os("SWAYSOCK").is_some();
    if is_sway {
        #[cfg(not(windows))]
        if let Some(text) = get_sway_outputs_json() {
            if let Some(scale) = parse_hyprctl_scale(text, connector) {
                return Some(scale);
            }
        }

        if let Ok(output) = crate::modules::system_command("swaymsg")
            .args(["-t", "get_outputs", "-r"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(scale) = parse_hyprctl_scale(&text, connector) {
                    return Some(scale);
                }
            }
        }
    }

    let config_dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".config")));

    // 3. GNOME / Mutter / Cinnamon: read monitors.xml strictly only when running GNOME
    let is_gnome = desktop.contains("gnome")
        || desktop.contains("mutter")
        || desktop.contains("cinnamon")
        || desktop.contains("unity")
        || desktop.contains("pantheon")
        || desktop.contains("budgie");

    if is_gnome {
        if let Some(ref dir) = config_dir {
            let path = dir.join("monitors.xml");
            if let Ok(xml) = fs::read_to_string(&path) {
                if let Some(scale) = parse_monitors_xml_scale(&xml, connector) {
                    return Some(scale);
                }
            }
        }
    }

    // 4. KDE: read kdeglobals strictly only when running KDE/Plasma
    let is_kde = desktop.contains("kde") || desktop.contains("plasma");
    if is_kde {
        if let Some(ref dir) = config_dir {
            let kdeglobals = dir.join("kdeglobals");
            if let Ok(content) = fs::read_to_string(&kdeglobals) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(rest) = trimmed.strip_prefix("ScaleFactor=") {
                        if let Ok(scale) = rest.trim().parse::<f64>() {
                            if scale > 0.0 {
                                return Some(scale);
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. X11: query xrdb for Xft.dpi when in X11 session
    if std::env::var_os("DISPLAY").is_some() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        if let Ok(output) = crate::modules::system_command("xrdb")
            .arg("-query")
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(scale) = parse_xrdb_scale(&text) {
                    return Some(scale);
                }
            }
        }
    }

    // 6. Toolkit environment variables fallback
    for var in &["GDK_SCALE", "QT_SCALE_FACTOR", "ELM_SCALE"] {
        if let Ok(val) = std::env::var(var) {
            if let Ok(scale) = val.trim().parse::<f64>() {
                if scale > 0.0 {
                    return Some(scale);
                }
            }
        }
    }

    None
}

/// Parses xrandr standard output for current resolution and refresh rate.
pub fn parse_xrandr_output(output: &str) -> Option<DisplayInfo> {
    for line in output.lines() {
        if line.contains('*') {
            // e.g. "   1920x1080     59.96*+"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(res) = parts.first() {
                if res.contains('x') {
                    let mut rate: Option<u32> = None;
                    for part in parts.iter().skip(1) {
                        if part.contains('*') {
                            let clean_rate = part.replace(['*', '+'], "");
                            if let Ok(hz) = clean_rate.trim().parse::<f64>() {
                                rate = Some(hz.round() as u32);
                                break;
                            }
                        }
                    }
                    return Some(DisplayInfo {
                        name: None,
                        resolution: res.to_string(),
                        refresh_rate: rate,
                        size_inches: None,
                        display_type: None,
                        scale: None,
                    });
                }
            }
        }
    }
    None
}

/// Parses wlr-randr output for current resolution and refresh rate.
pub fn parse_wlr_randr_output(output: &str) -> Option<DisplayInfo> {
    for line in output.lines() {
        if line.contains("current") || line.contains("Hz") {
            // e.g. "  1366x768 px, 60.000000 Hz (current)"
            let trimmed = line.trim();
            if let Some(px_idx) = trimmed.find("px") {
                let res = trimmed[..px_idx].trim().to_string();
                let hz_part = trimmed[px_idx + 2..].trim();
                let hz = hz_part
                    .split_whitespace()
                    .next()
                    .and_then(|h| h.trim_end_matches(',').parse::<f64>().ok())
                    .map(|f| f.round() as u32);
                if res.contains('x') {
                    return Some(DisplayInfo {
                        name: None,
                        resolution: res,
                        refresh_rate: hz,
                        size_inches: None,
                        display_type: None,
                        scale: None,
                    });
                }
            }
        }
    }
    None
}

fn extract_json_str(block: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let p = block.find(&pattern)?;
    let rest = block[p + pattern.len()..].trim_start();
    if let Some(rest_after_quote) = rest.strip_prefix('"') {
        let end = rest_after_quote.find('"')?;
        let s = rest_after_quote[..end].trim();
        if !s.is_empty() {
            return Some(s.to_string());
        }
    }
    None
}

fn extract_json_num(block: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    let p = block.find(&pattern)?;
    let rest = block[p + pattern.len()..].trim_start();
    let mut end = 0;
    let bytes = rest.as_bytes();
    while end < bytes.len()
        && (bytes[end].is_ascii_digit() || bytes[end] == b'.' || bytes[end] == b'-')
    {
        end += 1;
    }
    if end > 0 {
        rest[..end].parse::<f64>().ok()
    } else {
        None
    }
}

fn extract_json_bool(block: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    let p = block.find(&pattern)?;
    let rest = block[p + pattern.len()..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn probe_drm_edid_name(connector: &str) -> Option<String> {
    let entries = fs::read_dir("/sys/class/drm").ok()?;
    let suffix = format!("-{}", connector);
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == connector || name_str.ends_with(&suffix) {
            let edid_path = entry.path().join("edid");
            if let Ok(data) = fs::read(edid_path) {
                if let Some(info) = parse_edid_binary(&data, connector) {
                    if let Some(n) = info.name {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

fn extract_json_objects(json: &str) -> Vec<&str> {
    let mut depth = 0;
    let mut start = None;
    let mut blocks = Vec::new();

    for (i, c) in json.char_indices() {
        if c == '{' {
            if depth == 0 {
                start = Some(i + 1);
            }
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                if let Some(s) = start {
                    blocks.push(&json[s..i]);
                    start = None;
                }
            }
        }
    }
    blocks
}

pub fn parse_hyprland_monitors(json: &str) -> Vec<DisplayInfo> {
    let mut monitors = Vec::new();
    let trimmed = json.trim();
    if !trimmed.starts_with('[') {
        return monitors;
    }

    for block in extract_json_objects(trimmed) {
        if extract_json_bool(block, "disabled") == Some(true) {
            continue;
        }

        let name = extract_json_str(block, "name");
        let desc = extract_json_str(block, "description");
        let model = extract_json_str(block, "model");
        let width = extract_json_num(block, "width").map(|n| n as u32);
        let height = extract_json_num(block, "height").map(|n| n as u32);
        let refresh_rate = extract_json_num(block, "refreshRate").map(|r| r.round() as u32);
        let scale = extract_json_num(block, "scale");
        let pw = extract_json_num(block, "physicalWidth");
        let ph = extract_json_num(block, "physicalHeight");

        let size_inches = match (pw, ph) {
            (Some(w_mm), Some(h_mm)) if w_mm > 0.0 && h_mm > 0.0 => {
                let diag = (w_mm * w_mm + h_mm * h_mm).sqrt();
                let inches = (diag / 25.4).round() as u32;
                if inches > 0 {
                    Some(inches)
                } else {
                    None
                }
            }
            _ => None,
        };

        let conn_name = name.clone().unwrap_or_default();
        let conn_lower = conn_name.to_lowercase();
        let display_type = if conn_lower.contains("edp")
            || conn_lower.contains("lvds")
            || conn_lower.contains("dsi")
        {
            Some("[Built-in]".to_string())
        } else if conn_lower.contains("hdmi")
            || conn_lower.contains("dp")
            || conn_lower.contains("vga")
            || conn_lower.contains("dvi")
        {
            Some("[External]".to_string())
        } else {
            None
        };

        let mut monitor_name = None;
        if !conn_name.is_empty() {
            monitor_name = probe_drm_edid_name(&conn_name);
        }

        if monitor_name.is_none() {
            if let Some(ref m) = model {
                if !m.is_empty() && !m.starts_with("0x") {
                    monitor_name = Some(m.clone());
                }
            }
        }
        if monitor_name.is_none() {
            if let Some(ref d) = desc {
                if !d.is_empty() {
                    monitor_name = Some(d.clone());
                }
            }
        }
        if monitor_name.is_none() {
            monitor_name = name;
        }

        let resolution = match (width, height) {
            (Some(w), Some(h)) => format!("{}x{}", w, h),
            _ => continue,
        };

        monitors.push(DisplayInfo {
            name: monitor_name,
            resolution,
            refresh_rate,
            size_inches,
            display_type,
            scale,
        });
    }

    monitors
}

pub fn parse_sway_outputs(json: &str) -> Vec<DisplayInfo> {
    let mut monitors = Vec::new();
    let trimmed = json.trim();
    if !trimmed.starts_with('[') {
        return monitors;
    }

    for block in extract_json_objects(trimmed) {
        if extract_json_bool(block, "active") == Some(false) {
            continue;
        }

        let name = extract_json_str(block, "name");
        let model = extract_json_str(block, "model");
        let scale = extract_json_num(block, "scale");
        let width = extract_json_num(block, "width").map(|n| n as u32);
        let height = extract_json_num(block, "height").map(|n| n as u32);
        let refresh_rate = extract_json_num(block, "refresh").map(|r| {
            if r > 1000.0 {
                (r / 1000.0).round() as u32
            } else {
                r.round() as u32
            }
        });

        let conn_name = name.clone().unwrap_or_default();
        let conn_lower = conn_name.to_lowercase();
        let display_type = if conn_lower.contains("edp")
            || conn_lower.contains("lvds")
            || conn_lower.contains("dsi")
        {
            Some("[Built-in]".to_string())
        } else if conn_lower.contains("hdmi")
            || conn_lower.contains("dp")
            || conn_lower.contains("vga")
            || conn_lower.contains("dvi")
        {
            Some("[External]".to_string())
        } else {
            None
        };

        let monitor_name = if !conn_name.is_empty() {
            probe_drm_edid_name(&conn_name)
        } else {
            None
        }
        .or(model)
        .or(name);

        let resolution = match (width, height) {
            (Some(w), Some(h)) => format!("{}x{}", w, h),
            _ => continue,
        };

        monitors.push(DisplayInfo {
            name: monitor_name,
            resolution,
            refresh_rate,
            size_inches: None,
            display_type,
            scale,
        });
    }

    monitors
}

/// Probes all active display monitors across Hyprland, Sway, DRM sysfs, or fallback display servers.
pub fn detect_displays() -> Vec<DisplayInfo> {
    // 1. Hyprland and Sway direct Unix socket IPC (<0.1ms)
    #[cfg(not(windows))]
    {
        let desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();
        let is_hyprland = desktop.contains("hyprland")
            || std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some();

        if is_hyprland {
            if let Some(json) = get_hyprland_monitors_json() {
                let displays = parse_hyprland_monitors(json);
                if !displays.is_empty() {
                    return displays;
                }
            }
        }

        if std::env::var_os("SWAYSOCK").is_some() {
            if let Some(json) = get_sway_outputs_json() {
                let displays = parse_sway_outputs(json);
                if !displays.is_empty() {
                    return displays;
                }
            }
        }
    }

    // 3. Sysfs DRM modes + EDID (<0.1ms) for multi-monitor Linux (Wayland, X11, KMS, TTY)
    let drm_dir = "/sys/class/drm";
    if let Ok(entries) = fs::read_dir(drm_dir) {
        let mut drm_displays = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(status) = fs::read_to_string(path.join("status")) {
                if status.trim() == "connected" {
                    let conn_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let scale = detect_display_scale(Some(&conn_name));

                    let resolution_from_modes = fs::read_to_string(path.join("modes"))
                        .ok()
                        .and_then(|modes| {
                            modes.lines().next().and_then(|first_mode| {
                                let clean = first_mode.trim();
                                if clean.contains('x') {
                                    Some(clean.to_string())
                                } else {
                                    None
                                }
                            })
                        });

                    if let Ok(edid_bytes) = fs::read(path.join("edid")) {
                        if let Some(mut info) = parse_edid_binary(&edid_bytes, &conn_name) {
                            if let Some(ref res) = resolution_from_modes {
                                info.resolution = res.clone();
                            }
                            info.scale = scale;
                            drm_displays.push(info);
                            continue;
                        }
                    }

                    if let Some(res) = resolution_from_modes {
                        drm_displays.push(DisplayInfo {
                            name: None,
                            resolution: res,
                            refresh_rate: None,
                            size_inches: None,
                            display_type: None,
                            scale,
                        });
                    }
                }
            }
        }
        if !drm_displays.is_empty() {
            return drm_displays;
        }
    }

    // 4. Fallback: Query xrandr or wlr-randr if graphical display session is active and DRM sysfs is unavailable
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(output) = crate::modules::system_command("xrandr").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(mut info) = parse_xrandr_output(&text) {
                    info.scale = detect_display_scale(None);
                    return vec![info];
                }
            }
        }

        if let Ok(output) = crate::modules::system_command("wlr-randr").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(mut info) = parse_wlr_randr_output(&text) {
                    info.scale = detect_display_scale(None);
                    return vec![info];
                }
            }
        }
    }

    Vec::new()
}

/// Probes primary display resolution, refresh rate, size, and monitor name.
pub fn detect_display() -> Option<DisplayInfo> {
    detect_displays().into_iter().next()
}

pub struct DisplayCollector;

impl Collector for DisplayCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Display
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        self.collect_multiple(ctx).into_iter().next()
    }

    fn collect_multiple(&self, _ctx: &FetchContext) -> Vec<ModuleOutput> {
        let displays = detect_displays();
        let mut outputs = Vec::with_capacity(displays.len());

        for info in displays {
            let label = match info.name {
                Some(ref n) => format!("Display ({})", n),
                None => "Display".to_string(),
            };

            let mut main_str = info.resolution;
            if let Some(scale) = info.scale {
                if (scale - 1.0).abs() > 0.01 {
                    main_str.push_str(&format!(" @ {:.2}x", scale));
                }
            }
            if let Some(inch) = info.size_inches {
                main_str.push_str(&format!(" in {}\"", inch));
            }
            let mut sub_parts = vec![main_str];
            if let Some(hz) = info.refresh_rate {
                sub_parts.push(format!("{} Hz", hz));
            }
            let mut value = sub_parts.join(", ");
            if let Some(ref dtype) = info.display_type {
                value.push(' ');
                value.push_str(dtype);
            }

            outputs.push(ModuleOutput {
                id: ModuleId::Display,
                label,
                value,
                custom_rendered: None,
            });
        }

        outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xrandr_output_standard() {
        let sample = r#"
Screen 0: minimum 16 x 16, current 1920 x 1080, maximum 32767 x 32767
rdp-0 connected 1920x1080+0+0 (normal left inverted right x axis y axis) 0mm x 0mm
   1920x1080     59.96*+
   1440x1080     59.99  
"#;
        let info = parse_xrandr_output(sample).unwrap();
        assert_eq!(info.resolution, "1920x1080");
        assert_eq!(info.refresh_rate, Some(60));
    }

    #[test]
    fn test_parse_edid_binary_builtin() {
        let mut edid = [0u8; 128];
        // Standard header
        edid[0..8].copy_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
        // AUO manufacturer code (0x06, 0xAF)
        edid[8] = 0x06;
        edid[9] = 0xAF;
        // Product code: 0xD0A2
        edid[10] = 0xA2;
        edid[11] = 0xD0;
        // Physical size: 34cm x 19cm (15 inch diagonal)
        edid[21] = 34;
        edid[22] = 19;

        // Pixel clock: 33742 (337.42 MHz = 0x83CE)
        edid[54] = 0xCE;
        edid[55] = 0x83;
        // H active: 1920 (0x780), H blank: 160 (0x0A0) -> H total: 2080
        edid[56] = 0x80;
        edid[57] = 0xA0;
        edid[58] = 0x70;
        // V active: 1080 (0x438), V blank: 45 (0x02D) -> V total: 1125
        edid[59] = 0x38;
        edid[60] = 0x2D;
        edid[61] = 0x40;

        let info = parse_edid_binary(&edid, "card1-eDP-1").unwrap();
        assert_eq!(info.name.as_deref(), Some("AUOD0A2"));
        assert_eq!(info.resolution, "1920x1080");
        assert_eq!(info.size_inches, Some(15));
        assert_eq!(info.refresh_rate, Some(144));
        assert_eq!(info.display_type.as_deref(), Some("[Built-in]"));
    }

    #[test]
    fn test_parse_monitors_xml_scale() {
        let sample = r#"
<monitors version="2">
  <configuration>
    <layoutmode>logical</layoutmode>
    <logicalmonitor>
      <x>0</x>
      <y>0</y>
      <scale>1.3333333730697632</scale>
      <primary>yes</primary>
      <monitor>
        <monitorspec>
          <connector>eDP-1</connector>
          <vendor>AUO</vendor>
          <product>0xd0a2</product>
        </monitorspec>
      </monitor>
    </logicalmonitor>
  </configuration>
</monitors>
"#;
        let scale = parse_monitors_xml_scale(sample, Some("card1-eDP-1"));
        assert!(scale.is_some());
        let s = scale.unwrap();
        assert!((s - 1.3333333730697632).abs() < 1e-6);
    }

    #[test]
    fn test_parse_hyprctl_scale() {
        let sample = r#"[
{
    "id": 0,
    "name": "eDP-1",
    "description": "AU Optronics 0xD0A2",
    "width": 1920,
    "height": 1080,
    "scale": 1.25,
    "focused": true
}
]"#;
        let scale = parse_hyprctl_scale(sample, Some("card1-eDP-1"));
        assert!(scale.is_some());
        let s = scale.unwrap();
        assert!((s - 1.25).abs() < 1e-6);

        let scale_direct = parse_hyprctl_scale(sample, Some("eDP-1"));
        assert!(scale_direct.is_some());
        assert!((scale_direct.unwrap() - 1.25).abs() < 1e-6);

        let scale_unmatched = parse_hyprctl_scale(sample, Some("HDMI-A-1"));
        assert!(scale_unmatched.is_none());
    }

    #[test]
    fn test_parse_xrdb_scale() {
        let sample = "Xcursor.size:\t24\nXft.dpi:\t144\nXft.antialias:\t1\n";
        let scale = parse_xrdb_scale(sample);
        assert!(scale.is_some());
        assert!((scale.unwrap() - 1.5).abs() < 1e-6);

        let default_dpi = "Xft.dpi: 96\n";
        let s_default = parse_xrdb_scale(default_dpi);
        assert!(s_default.is_some());
        assert!((s_default.unwrap() - 1.0).abs() < 1e-6);

        let missing = "Xcursor.size: 24\n";
        assert!(parse_xrdb_scale(missing).is_none());
    }

    #[test]
    fn test_parse_sway_outputs_scale() {
        let sample = r#"[
  {
    "id": 48,
    "name": "eDP-1",
    "rect": {
      "x": 0,
      "y": 0,
      "width": 1920,
      "height": 1080
    },
    "scale": 1.333333,
    "active": true
  }
]"#;
        let scale = parse_hyprctl_scale(sample, Some("eDP-1"));
        assert!(scale.is_some());
        assert!((scale.unwrap() - 1.333333).abs() < 1e-6);
    }

    #[test]
    fn test_parse_hyprland_monitors_multi() {
        let sample = r#"[
{
    "id": 0,
    "name": "eDP-1",
    "description": "AU Optronics 0xD0A2",
    "make": "AU Optronics",
    "model": "0xD0A2",
    "width": 1920,
    "height": 1080,
    "physicalWidth": 340,
    "physicalHeight": 190,
    "refreshRate": 144.42000,
    "scale": 1.25,
    "disabled": false
},
{
    "id": 1,
    "name": "DP-1",
    "description": "LG Electronics LG ULTRAGEAR",
    "make": "LG Electronics",
    "model": "LG ULTRAGEAR",
    "width": 1920,
    "height": 1080,
    "physicalWidth": 530,
    "physicalHeight": 300,
    "refreshRate": 180.00000,
    "scale": 1.0,
    "disabled": false
}
]"#;
        let monitors = parse_hyprland_monitors(sample);
        assert_eq!(monitors.len(), 2);

        assert_eq!(monitors[0].resolution, "1920x1080");
        assert_eq!(monitors[0].refresh_rate, Some(144));
        assert_eq!(monitors[0].scale, Some(1.25));
        assert_eq!(monitors[0].size_inches, Some(15));
        assert_eq!(monitors[0].display_type.as_deref(), Some("[Built-in]"));

        assert_eq!(monitors[1].resolution, "1920x1080");
        assert_eq!(monitors[1].refresh_rate, Some(180));
        assert_eq!(monitors[1].scale, Some(1.0));
        assert_eq!(monitors[1].name.as_deref(), Some("LG ULTRAGEAR"));
        assert_eq!(monitors[1].display_type.as_deref(), Some("[External]"));
    }

    #[test]
    fn test_parse_sway_outputs_multi() {
        let sample = r#"[
  {
    "name": "eDP-1",
    "model": "0xD0A2",
    "active": true,
    "scale": 1.25,
    "width": 1920,
    "height": 1080,
    "refresh": 144420
  },
  {
    "name": "DP-1",
    "model": "LG ULTRAGEAR",
    "active": true,
    "scale": 1.0,
    "width": 1920,
    "height": 1080,
    "refresh": 180000
  }
]"#;
        let monitors = parse_sway_outputs(sample);
        assert_eq!(monitors.len(), 2);
        assert_eq!(monitors[0].refresh_rate, Some(144));
        assert_eq!(monitors[1].refresh_rate, Some(180));
        assert_eq!(monitors[1].name.as_deref(), Some("LG ULTRAGEAR"));
    }
}
