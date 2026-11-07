use crate::context::ColorLevel;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";

/// Standard 16-color ANSI escape sequences.
pub const ANSI_BLACK: &str = "\x1b[30m";
pub const ANSI_RED: &str = "\x1b[31m";
pub const ANSI_GREEN: &str = "\x1b[32m";
pub const ANSI_YELLOW: &str = "\x1b[33m";
pub const ANSI_BLUE: &str = "\x1b[34m";
pub const ANSI_MAGENTA: &str = "\x1b[35m";
pub const ANSI_CYAN: &str = "\x1b[36m";
pub const ANSI_WHITE: &str = "\x1b[37m";

pub const ANSI_BRIGHT_BLACK: &str = "\x1b[90m";
pub const ANSI_BRIGHT_RED: &str = "\x1b[91m";
pub const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";
pub const ANSI_BRIGHT_YELLOW: &str = "\x1b[93m";
pub const ANSI_BRIGHT_BLUE: &str = "\x1b[94m";
pub const ANSI_BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const ANSI_BRIGHT_CYAN: &str = "\x1b[96m";
pub const ANSI_BRIGHT_WHITE: &str = "\x1b[97m";

/// Coordinates of standard 16 ANSI colors in sRGB space.
const ANSI_16_PALETTE: [(u8, u8, u8, &str); 16] = [
    (0, 0, 0, ANSI_BLACK),
    (205, 0, 0, ANSI_RED),
    (0, 205, 0, ANSI_GREEN),
    (205, 205, 0, ANSI_YELLOW),
    (0, 0, 238, ANSI_BLUE),
    (205, 0, 205, ANSI_MAGENTA),
    (0, 205, 205, ANSI_CYAN),
    (229, 229, 229, ANSI_WHITE),
    (127, 127, 127, ANSI_BRIGHT_BLACK),
    (255, 0, 0, ANSI_BRIGHT_RED),
    (0, 255, 0, ANSI_BRIGHT_GREEN),
    (255, 255, 0, ANSI_BRIGHT_YELLOW),
    (92, 92, 255, ANSI_BRIGHT_BLUE),
    (255, 0, 255, ANSI_BRIGHT_MAGENTA),
    (0, 255, 255, ANSI_BRIGHT_CYAN),
    (255, 255, 255, ANSI_BRIGHT_WHITE),
];

const CUBE_STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Maps a 256-color index `n` (0..=255) to its closest standard 16-color ANSI code.
pub fn color256_to_ansi16(n: u8) -> &'static str {
    match n {
        0..=7 => match n {
            0 => ANSI_BLACK,
            1 => ANSI_RED,
            2 => ANSI_GREEN,
            3 => ANSI_YELLOW,
            4 => ANSI_BLUE,
            5 => ANSI_MAGENTA,
            6 => ANSI_CYAN,
            _ => ANSI_WHITE,
        },
        8..=15 => match n {
            8 => ANSI_BRIGHT_BLACK,
            9 => ANSI_BRIGHT_RED,
            10 => ANSI_BRIGHT_GREEN,
            11 => ANSI_BRIGHT_YELLOW,
            12 => ANSI_BRIGHT_BLUE,
            13 => ANSI_BRIGHT_MAGENTA,
            14 => ANSI_BRIGHT_CYAN,
            _ => ANSI_BRIGHT_WHITE,
        },
        16..=231 => {
            let idx = n - 16;
            let r_idx = (idx / 36) as usize;
            let g_idx = ((idx % 36) / 6) as usize;
            let b_idx = (idx % 6) as usize;

            let r = CUBE_STEPS[r_idx];
            let g = CUBE_STEPS[g_idx];
            let b = CUBE_STEPS[b_idx];

            find_closest_ansi16(r, g, b)
        }
        232..=255 => {
            let gray = 8 + (n - 232) * 10;
            find_closest_ansi16(gray, gray, gray)
        }
    }
}

/// Finds the nearest standard 16 ANSI color using squared Euclidean distance in RGB space.
pub fn find_closest_ansi16(r: u8, g: u8, b: u8) -> &'static str {
    let mut min_dist = u64::MAX;
    let mut closest = ANSI_WHITE;

    for &(ar, ag, ab, seq) in &ANSI_16_PALETTE {
        let dr = (r as i64) - (ar as i64);
        let dg = (g as i64) - (ag as i64);
        let db = (b as i64) - (ab as i64);
        let dist = (dr * dr + dg * dg + db * db) as u64;

        if dist < min_dist {
            min_dist = dist;
            closest = seq;
        }
    }

    closest
}

/// Resolves an ANSI color string according to the target terminal color capability.
/// Degrades 256-color sequences to 16-color ANSI on `Basic16` and strips them completely on `None`.
pub fn resolve_color(code: &'static str, level: ColorLevel) -> &'static str {
    match level {
        ColorLevel::None => "",
        ColorLevel::Basic16 => {
            if let Some(inner) = code
                .strip_prefix("\x1b[38;5;")
                .and_then(|s| s.strip_suffix('m'))
            {
                if let Ok(n) = inner.parse::<u8>() {
                    return color256_to_ansi16(n);
                }
            }
            code
        }
        ColorLevel::Color256 | ColorLevel::TrueColor => code,
    }
}

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

    #[test]
    fn test_color256_to_ansi16_standard_and_bright() {
        assert_eq!(color256_to_ansi16(1), ANSI_RED);
        assert_eq!(color256_to_ansi16(2), ANSI_GREEN);
        assert_eq!(color256_to_ansi16(4), ANSI_BLUE);
        assert_eq!(color256_to_ansi16(9), ANSI_BRIGHT_RED);
        assert_eq!(color256_to_ansi16(10), ANSI_BRIGHT_GREEN);
        assert_eq!(color256_to_ansi16(14), ANSI_BRIGHT_CYAN);
    }

    #[test]
    fn test_color256_to_ansi16_cube_fallbacks() {
        // Red 196
        assert_eq!(color256_to_ansi16(196), ANSI_BRIGHT_RED);
        // Orange 208 maps to ANSI yellow/orange
        assert_eq!(color256_to_ansi16(208), ANSI_YELLOW);
        // White 231 maps to bright white
        assert_eq!(color256_to_ansi16(231), ANSI_BRIGHT_WHITE);
        // Lime green 46 maps to bright green
        assert_eq!(color256_to_ansi16(46), ANSI_BRIGHT_GREEN);
        // Sky blue 39 maps to cyan (closest distance in sRGB)
        assert_eq!(color256_to_ansi16(39), ANSI_CYAN);
    }

    #[test]
    fn test_resolve_color_levels() {
        let orange_256 = "\x1b[38;5;208m";

        // None strips color
        assert_eq!(resolve_color(orange_256, ColorLevel::None), "");

        // Basic16 degrades 256-color code to 16-color ANSI sequence
        assert_eq!(resolve_color(orange_256, ColorLevel::Basic16), ANSI_YELLOW);

        // Color256 and TrueColor preserve full fidelity
        assert_eq!(resolve_color(orange_256, ColorLevel::Color256), orange_256);
        assert_eq!(resolve_color(orange_256, ColorLevel::TrueColor), orange_256);
    }
}
