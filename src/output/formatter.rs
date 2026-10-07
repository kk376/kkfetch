use crate::modules::ModuleOutput;
use crate::output::color::format_label;
use crate::output::logo::Logo;

/// Returns the column display width of a Unicode character (1 for standard, 2 for wide CJK/emojis).
/// Adheres to Unicode Standard Annex #11 (East Asian Width) for terminal cell occupancy.
pub fn char_width(c: char) -> usize {
    match c as u32 {
        0x1100..=0x115F
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE10..=0xFE19
        | 0xFE30..=0xFE6F
        | 0xFF01..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F9FF => 2,
        _ => 1,
    }
}

/// Calculates the visible printable width of a string, ignoring ANSI escape sequences.
/// Necessary so colored text does not distort column alignment calculations in side-by-side rendering.
pub fn visible_width(s: &str) -> usize {
    let mut in_escape = false;
    let mut len = 0;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            // ANSI CSI sequences end with an alphabetic character (e.g. 'm' for SGR)
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            len += char_width(c);
        }
    }
    len
}

/// Renders the complete fetch layout with logo and module info lines.
pub fn render_layout(
    logo: Option<&Logo>,
    outputs: &[ModuleOutput],
    term_width: u16,
    enable_color: bool,
) -> String {
    let distro_color = logo.map(|l| l.distro_color).unwrap_or("\x1b[38;5;208m");

    // Flatten module outputs into display lines
    let mut info_lines: Vec<String> = Vec::new();
    for out in outputs {
        if let Some(ref custom) = out.custom_rendered {
            for line in custom.lines() {
                info_lines.push(sanitize_terminal_string(line));
            }
        } else if !out.value.is_empty() {
            let label = format_label(&out.label, distro_color, enable_color);
            let clean_val = sanitize_terminal_string(&out.value);
            info_lines.push(format!("{} {}", label, clean_val));
        }
    }

    // When logo is absent, render info lines directly
    let logo = match logo {
        Some(l) => l,
        None => return info_lines.join("\n"),
    };

    let logo_lines = logo.render_lines(enable_color);

    // Narrow terminal breakpoint: switch to vertical stacked layout below 60 columns to prevent line-wrapping
    if term_width < 60 {
        let mut full_output = Vec::new();
        full_output.extend(logo_lines);
        if !info_lines.is_empty() {
            full_output.push(String::new());
            full_output.extend(info_lines);
        }
        return full_output.join("\n");
    }

    // Side-by-side two-column layout: calculate max logo width to align the info column
    let max_logo_width = logo_lines
        .iter()
        .map(|l| visible_width(l))
        .max()
        .unwrap_or(0);

    let row_count = std::cmp::max(logo_lines.len(), info_lines.len());
    let mut rows: Vec<String> = Vec::with_capacity(row_count);

    for i in 0..row_count {
        let has_logo = i < logo_lines.len();
        let has_info = i < info_lines.len();

        let l_line = if has_logo { &logo_lines[i] } else { "" };
        let vis_len = if has_logo {
            visible_width(&logo_lines[i])
        } else {
            0
        };

        // Pad shorter logo lines to match max_logo_width before inserting 3-space separator
        let pad_len = max_logo_width.saturating_sub(vis_len);
        let padding = " ".repeat(pad_len);

        let i_line = if has_info { &info_lines[i] } else { "" };

        if has_info && !i_line.is_empty() {
            rows.push(format!("{}{}{}{}", l_line, padding, "   ", i_line));
        } else if has_logo {
            rows.push(l_line.to_string());
        } else {
            rows.push(String::new());
        }
    }

    rows.join("\n")
}

/// Sanitizes untrusted strings for safe human-readable terminal rendering by stripping
/// dangerous OSC terminal manipulation sequences and raw unprintable ASCII control characters (F5).
pub fn sanitize_terminal_string(s: &str) -> String {
    let mut clean = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&']') {
                // Strip OSC sequences (e.g. \x1b]...\x07 or \x1b]...\x1b\)
                chars.next(); // consume ']'
                while let Some(osc_c) = chars.next() {
                    if osc_c == '\x07' {
                        break;
                    }
                    if osc_c == '\x1b' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
                continue;
            }
            // Preserve standard ANSI styling sequences (\x1b[...m)
            clean.push(c);
        } else if (c as u32) < 0x20 && c != '\n' && c != '\t' {
            // Drop raw non-printable C0 control characters
            continue;
        } else if (c as u32) == 0x7F {
            // Drop DEL
            continue;
        } else {
            clean.push(c);
        }
    }
    clean
}

/// Escapes a string for valid JSON output.
pub fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Renders collected module outputs into a formatted JSON string with full key/value escaping.
pub fn render_json(outputs: &[ModuleOutput]) -> String {
    use crate::modules::ModuleId;

    let mut fields: Vec<String> = Vec::new();
    for out in outputs {
        if out.id == ModuleId::Colors {
            continue;
        }
        if !out.value.is_empty() {
            let key = if out.label.is_empty() {
                out.id.as_str().to_string()
            } else {
                out.label.to_lowercase().replace(' ', "_")
            };
            let escaped_key = escape_json_string(&key);
            let escaped_val = escape_json_string(&out.value);
            fields.push(format!("  \"{}\": \"{}\"", escaped_key, escaped_val));
        }
    }

    format!("{{\n{}\n}}", fields.join(",\n"))
}

/// Renders execution timings in a compact multi-column grid to maintain compactness
/// and prevent vertical scrolling on typical terminal window sizes.
pub fn render_timings_grid(
    timings: &[(crate::modules::ModuleId, std::time::Duration)],
    total_elapsed: std::time::Duration,
    term_width: u16,
    enable_color: bool,
) -> String {
    if timings.is_empty() {
        return String::new();
    }

    let cyan = if enable_color { "\x1b[1;36m" } else { "" };
    let reset = if enable_color { "\x1b[0m" } else { "" };

    // Format each module timing into a fixed-width cell: "  <module>       :   <time>"
    let formatted_cells: Vec<String> = timings
        .iter()
        .map(|(mod_id, dur)| {
            let micros = dur.as_micros();
            let dur_str = if micros < 1000 {
                format!("{} µs", micros)
            } else {
                format!("{:.2} ms", dur.as_secs_f64() * 1000.0)
            };
            format!("  {:<13} : {:>7}", mod_id.as_str(), dur_str)
        })
        .collect();

    // Determine number of columns based on terminal width.
    // Each cell is 25 chars wide. With 2 spaces between columns:
    // 3 columns: 25 * 3 + 4 = 79 chars (fits in standard 80-col terminals)
    // 2 columns: 25 * 2 + 2 = 52 chars (fits in >= 54 col terminals)
    // 1 column: 27 chars
    let num_cols = if term_width >= 80 {
        3
    } else if term_width >= 54 {
        2
    } else {
        1
    };

    let cell_width: usize = 25;
    let col_gap = "  ";
    let n = formatted_cells.len();
    let num_rows = n.div_ceil(num_cols);

    let total_width = if num_cols == 3 {
        cell_width * 3 + 4
    } else if num_cols == 2 {
        cell_width * 2 + 2
    } else {
        cell_width + 2
    };

    let mut lines = Vec::new();
    lines.push(format!(
        "\n{}=== Module Execution Timings ==={}",
        cyan, reset
    ));

    for row in 0..num_rows {
        let mut row_str = String::new();
        for col in 0..num_cols {
            let idx = col * num_rows + row;
            if idx < n {
                let cell = &formatted_cells[idx];
                let is_last_col = col + 1 == num_cols || (col + 1) * num_rows + row >= n;
                if is_last_col {
                    row_str.push_str(cell);
                } else {
                    let pad = cell_width.saturating_sub(cell.len());
                    row_str.push_str(cell);
                    row_str.push_str(&" ".repeat(pad));
                    row_str.push_str(col_gap);
                }
            }
        }
        lines.push(row_str);
    }

    let divider = "-".repeat(total_width);
    lines.push(format!("{}{}{}", cyan, divider, reset));

    let total_str = format!(
        "  {:<13} : {:>7.2} ms (parallel wall clock)",
        "Total Time",
        total_elapsed.as_secs_f64() * 1000.0
    );
    lines.push(total_str);

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ModuleId;

    #[test]
    fn test_visible_width_plain() {
        assert_eq!(visible_width("hello world"), 11);
    }

    #[test]
    fn test_visible_width_with_ansi() {
        let text = "\x1b[38;5;208mhello\x1b[0m world";
        assert_eq!(visible_width(text), 11);
    }

    #[test]
    fn test_render_layout_no_logo() {
        let outputs = vec![
            ModuleOutput {
                id: ModuleId::Os,
                label: "OS".to_string(),
                value: "Debian GNU/Linux 12".to_string(),
                custom_rendered: None,
            },
            ModuleOutput {
                id: ModuleId::Kernel,
                label: "Kernel".to_string(),
                value: "6.1.0".to_string(),
                custom_rendered: None,
            },
        ];

        let rendered = render_layout(None, &outputs, 80, false);
        assert_eq!(rendered, "OS: Debian GNU/Linux 12\nKernel: 6.1.0");
    }

    #[test]
    fn test_render_layout_vertical_narrow() {
        let logo = Logo {
            name: "test",
            raw_lines: &["AA", "BB"],
            distro_color: "",
            outer_color: "",
        };

        let outputs = vec![ModuleOutput {
            id: ModuleId::Os,
            label: "OS".to_string(),
            value: "Linux".to_string(),
            custom_rendered: None,
        }];

        let rendered = render_layout(Some(&logo), &outputs, 50, false);
        assert_eq!(rendered, "AA\nBB\n\nOS: Linux");
    }

    #[test]
    fn test_render_layout_side_by_side() {
        let logo = Logo {
            name: "test",
            raw_lines: &["A", "BBB"],
            distro_color: "",
            outer_color: "",
        };

        let outputs = vec![
            ModuleOutput {
                id: ModuleId::Os,
                label: "OS".to_string(),
                value: "Linux".to_string(),
                custom_rendered: None,
            },
            ModuleOutput {
                id: ModuleId::Kernel,
                label: "Kernel".to_string(),
                value: "6.1".to_string(),
                custom_rendered: None,
            },
        ];

        let rendered = render_layout(Some(&logo), &outputs, 80, false);
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "A     OS: Linux");
        assert_eq!(lines[1], "BBB   Kernel: 6.1");
    }

    #[test]
    fn test_visible_width_wide_cjk() {
        // CJK characters take 2 columns each
        assert_eq!(visible_width("こんにちは"), 10);
        assert_eq!(visible_width("你好世界"), 8);
        assert_eq!(visible_width("\x1b[31m你好\x1b[0m"), 4);
    }

    #[test]
    fn test_sanitize_terminal_string_strips_osc_and_c0() {
        // OSC sequence title injection
        let malicious = "Fedora Linux\x1b]0;hacked_title\x07 (Workstation)";
        assert_eq!(
            sanitize_terminal_string(malicious),
            "Fedora Linux (Workstation)"
        );

        // Retains regular ANSI color styling
        let styled = "\x1b[38;5;208mFerris\x1b[0m";
        assert_eq!(
            sanitize_terminal_string(styled),
            "\x1b[38;5;208mFerris\x1b[0m"
        );

        // Drops raw non-printable C0 control characters (like bell \x07, backspace \x08, etc.)
        let c0_ctrl = "Hello\x07\x08World\x7f";
        assert_eq!(sanitize_terminal_string(c0_ctrl), "HelloWorld");
    }

    #[test]
    fn test_render_json_escapes_keys_and_values() {
        let outputs = vec![ModuleOutput {
            id: ModuleId::Plugin,
            label: "custom \"key\"\nwith newline".to_string(),
            value: "value with \"quotes\" and \t tabs".to_string(),
            custom_rendered: None,
        }];
        let json = render_json(&outputs);
        assert!(json.contains(
            "\"custom_\\\"key\\\"\\nwith_newline\": \"value with \\\"quotes\\\" and \\t tabs\""
        ));
    }

    #[test]
    fn test_render_timings_grid_columns() {
        let timings = vec![
            (ModuleId::Os, std::time::Duration::from_micros(120)),
            (ModuleId::Kernel, std::time::Duration::from_micros(80)),
            (ModuleId::Cpu, std::time::Duration::from_millis(2)),
            (ModuleId::Memory, std::time::Duration::from_micros(95)),
        ];
        let total = std::time::Duration::from_millis(3);

        // 3-column layout on wide terminal (width >= 80)
        let wide = render_timings_grid(&timings, total, 100, false);
        assert!(wide.contains("Module Execution Timings"));
        assert!(wide.contains("os"));
        assert!(wide.contains("kernel"));
        assert!(wide.contains("cpu"));
        assert!(wide.contains("memory"));
        assert!(wide.contains("Total Time"));

        // 2-column layout on medium terminal (width 60)
        let medium = render_timings_grid(&timings, total, 60, false);
        assert!(medium.contains("os"));

        // 1-column layout on narrow terminal (width 40)
        let narrow = render_timings_grid(&timings, total, 40, false);
        assert!(narrow.contains("os"));
    }
}
