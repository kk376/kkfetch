pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";

/// Applies an ANSI escape code to a string if color is enabled.
pub fn style(text: &str, ansi_code: &str, enabled: bool) -> String {
    if enabled && !ansi_code.is_empty() {
        format!("{}{}{}", ansi_code, text, RESET)
    } else {
        text.to_string()
    }
}

/// Applies bold styling to a string if color is enabled.
pub fn bold(text: &str, enabled: bool) -> String {
    if enabled {
        format!("{}{}{}", BOLD, text, RESET)
    } else {
        text.to_string()
    }
}

/// Formats a module label with bold primary color.
pub fn format_label(label: &str, primary_color: &str, enabled: bool) -> String {
    if enabled {
        format!("{}{}{}:{}", BOLD, primary_color, label, RESET)
    } else {
        format!("{}:", label)
    }
}

/// Formats a percentage with ANSI color codes based on threshold semantics.
///
/// For standard resource usage (`is_battery = false`):
/// - `< 50%`: Green (`\x1b[32m`)
/// - `50% - 80%`: Yellow (`\x1b[33m`)
/// - `> 80%`: Red (`\x1b[31m`)
///
/// For battery capacity (`is_battery = true`):
/// - `>= 50%`: Green (`\x1b[32m`)
/// - `20% - 49%`: Yellow (`\x1b[33m`)
/// - `< 20%`: Red (`\x1b[31m`)
pub fn format_percentage(percent: u64, is_battery: bool, enabled: bool) -> String {
    if !enabled {
        return format!("{}%", percent);
    }

    let color = if is_battery {
        if percent >= 50 {
            GREEN
        } else if percent >= 20 {
            YELLOW
        } else {
            RED
        }
    } else if percent < 50 {
        GREEN
    } else if percent <= 80 {
        YELLOW
    } else {
        RED
    };

    format!("{}{}%{}", color, percent, RESET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_enabled() {
        let s = style("hello", "\x1b[31m", true);
        assert_eq!(s, "\x1b[31mhello\x1b[0m");
    }

    #[test]
    fn test_style_disabled() {
        let s = style("hello", "\x1b[31m", false);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_bold_enabled() {
        let s = bold("text", true);
        assert_eq!(s, "\x1b[1mtext\x1b[0m");
    }

    #[test]
    fn test_bold_disabled() {
        let s = bold("text", false);
        assert_eq!(s, "text");
    }

    #[test]
    fn test_format_percentage_resource_usage() {
        // < 50% is green
        assert_eq!(format_percentage(0, false, true), "\x1b[32m0%\x1b[0m");
        assert_eq!(format_percentage(49, false, true), "\x1b[32m49%\x1b[0m");

        // 50% - 80% is yellow
        assert_eq!(format_percentage(50, false, true), "\x1b[33m50%\x1b[0m");
        assert_eq!(format_percentage(80, false, true), "\x1b[33m80%\x1b[0m");

        // > 80% is red
        assert_eq!(format_percentage(81, false, true), "\x1b[31m81%\x1b[0m");
        assert_eq!(format_percentage(100, false, true), "\x1b[31m100%\x1b[0m");

        // Disabled color
        assert_eq!(format_percentage(45, false, false), "45%");
        assert_eq!(format_percentage(75, false, false), "75%");
        assert_eq!(format_percentage(95, false, false), "95%");
    }

    #[test]
    fn test_format_percentage_battery() {
        // >= 50% is green
        assert_eq!(format_percentage(100, true, true), "\x1b[32m100%\x1b[0m");
        assert_eq!(format_percentage(50, true, true), "\x1b[32m50%\x1b[0m");

        // 20% - 49% is yellow
        assert_eq!(format_percentage(49, true, true), "\x1b[33m49%\x1b[0m");
        assert_eq!(format_percentage(20, true, true), "\x1b[33m20%\x1b[0m");

        // < 20% is red
        assert_eq!(format_percentage(19, true, true), "\x1b[31m19%\x1b[0m");
        assert_eq!(format_percentage(5, true, true), "\x1b[31m5%\x1b[0m");
        assert_eq!(format_percentage(0, true, true), "\x1b[31m0%\x1b[0m");

        // Disabled color
        assert_eq!(format_percentage(98, true, false), "98%");
        assert_eq!(format_percentage(35, true, false), "35%");
        assert_eq!(format_percentage(10, true, false), "10%");
    }
}
