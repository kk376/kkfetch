use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub modules: Option<Vec<String>>,
    pub disable: Option<Vec<String>>,
    pub logo: Option<String>,
    pub no_logo: Option<bool>,
    pub no_color: Option<bool>,
    pub disk_path: Option<String>,
    pub all_disks: Option<bool>,
    pub logo_variant: Option<String>,
    pub plugins: Option<Vec<PluginConfig>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginConfig {
    pub name: String,
    pub command: String,
}

impl Config {
    /// Loads configuration from default paths ($XDG_CONFIG_HOME/kkfetch/config.toml or ~/.config/kkfetch/config.toml).
    pub fn load_default() -> Self {
        if let Some(path) = get_default_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    return parse_config_toml(&content);
                }
            }
        }
        Config::default()
    }
}

pub fn get_default_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("kkfetch/config.toml"));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home).join(".config/kkfetch/config.toml"));
        }
    }
    None
}

/// Strips an unquoted `#` inline comment from a TOML value line.
fn strip_inline_comment(raw: &str) -> &str {
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;

    for (idx, ch) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_double {
            match ch {
                '\\' => escaped = true,
                '"' => in_double = false,
                _ => {}
            }
        } else if in_single {
            if ch == '\'' {
                in_single = false;
            }
        } else {
            match ch {
                '"' => in_double = true,
                '\'' => in_single = true,
                '#' => return raw[..idx].trim(),
                _ => {}
            }
        }
    }

    raw.trim()
}

/// Unquotes a single TOML string value after stripping any unquoted trailing comment.
fn parse_toml_scalar(raw: &str) -> String {
    let trimmed = strip_inline_comment(raw.trim());
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

/// Fast parser for kkfetch TOML config file.
pub fn parse_config_toml(content: &str) -> Config {
    let mut config = Config::default();
    let mut plugins = Vec::new();
    let mut in_plugin_section = false;
    let mut current_plugin_name = None;
    let mut current_plugin_command = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("[[plugins]]") || trimmed.starts_with("[[plugin]]") {
            if let (Some(n), Some(c)) = (current_plugin_name.take(), current_plugin_command.take())
            {
                plugins.push(PluginConfig {
                    name: n,
                    command: c,
                });
            }
            in_plugin_section = true;
            continue;
        } else if trimmed.starts_with('[') {
            if let (Some(n), Some(c)) = (current_plugin_name.take(), current_plugin_command.take())
            {
                plugins.push(PluginConfig {
                    name: n,
                    command: c,
                });
            }
            in_plugin_section = false;
            continue;
        }

        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim().to_lowercase();
            let raw_val = strip_inline_comment(v.trim());

            if in_plugin_section {
                let clean_val = parse_toml_scalar(v);
                if key == "name" || key == "label" {
                    current_plugin_name = Some(clean_val);
                } else if key == "command" || key == "cmd" || key == "exec" {
                    current_plugin_command = Some(clean_val);
                }
                continue;
            }

            match key.as_str() {
                "modules" => {
                    config.modules = parse_toml_string_array(raw_val);
                }
                "disable" => {
                    config.disable = parse_toml_string_array(raw_val);
                }
                "logo" => {
                    let clean = parse_toml_scalar(raw_val);
                    if !clean.is_empty() && clean != "auto" {
                        config.logo = Some(clean);
                    }
                }
                "no_logo" => {
                    config.no_logo = Some(raw_val.eq_ignore_ascii_case("true") || raw_val == "1");
                }
                "no_color" => {
                    config.no_color = Some(raw_val.eq_ignore_ascii_case("true") || raw_val == "1");
                }
                "disk_path" => {
                    let clean = parse_toml_scalar(raw_val);
                    if !clean.is_empty() {
                        config.disk_path = Some(clean);
                    }
                }
                "all_disks" | "all-disks" | "disk_all" | "disk-all" => {
                    config.all_disks = Some(raw_val.eq_ignore_ascii_case("true") || raw_val == "1");
                }
                "logo_variant" | "logo-variant" => {
                    let clean = parse_toml_scalar(raw_val);
                    if !clean.is_empty() {
                        config.logo_variant = Some(clean);
                    }
                }
                _ => {}
            }
        }
    }

    if let (Some(n), Some(c)) = (current_plugin_name.take(), current_plugin_command.take()) {
        plugins.push(PluginConfig {
            name: n,
            command: c,
        });
    }

    if !plugins.is_empty() {
        config.plugins = Some(plugins);
    }

    config
}

pub fn parse_toml_string_array(val: &str) -> Option<Vec<String>> {
    let trimmed = strip_inline_comment(val.trim());
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    let items: Vec<String> = inner
        .split(',')
        .map(parse_toml_scalar)
        .filter(|s| !s.is_empty())
        .collect();
    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_toml_standard() {
        let toml = r#"
# KKFetch config
modules = ["os", "kernel", "cpu", "memory"]
disable = ["gpu"]
logo = "fedora"
no_logo = false
no_color = false
disk_path = "/home"

[[plugins]]
name = "Spotify"
command = "playerctl metadata title"

[[plugins]]
name = "Git Branch"
command = "git branch --show-current"
"#;
        let config = parse_config_toml(toml);
        assert_eq!(
            config.modules,
            Some(vec![
                "os".to_string(),
                "kernel".to_string(),
                "cpu".to_string(),
                "memory".to_string(),
            ])
        );
        assert_eq!(config.disable, Some(vec!["gpu".to_string()]));
        assert_eq!(config.logo.as_deref(), Some("fedora"));
        assert_eq!(config.no_logo, Some(false));
        assert_eq!(config.disk_path.as_deref(), Some("/home"));

        let plugins = config.plugins.unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].name, "Spotify");
        assert_eq!(plugins[0].command, "playerctl metadata title");
        assert_eq!(plugins[1].name, "Git Branch");
        assert_eq!(plugins[1].command, "git branch --show-current");
    }

    #[test]
    fn test_parse_config_toml_logo_variant() {
        let toml_compact = "logo_variant = \"compact\"\n";
        let cfg1 = parse_config_toml(toml_compact);
        assert_eq!(cfg1.logo_variant.as_deref(), Some("compact"));

        let toml_full = "logo-variant = 'full'\n";
        let cfg2 = parse_config_toml(toml_full);
        assert_eq!(cfg2.logo_variant.as_deref(), Some("full"));
    }

    #[test]
    fn test_parse_config_toml_inline_comments_and_quotes() {
        let toml = r##"
modules = ["os", "kernel"] # active modules
logo = "arch" # distro override
no_logo = true # hide logo
disk_path = "/home#1" # preserve hash in quotes

[[plugins]]
name = "Tag #1" # comment
command = "echo ok # literal" # trailing comment
"##;
        let cfg = parse_config_toml(toml);
        assert_eq!(
            cfg.modules,
            Some(vec!["os".to_string(), "kernel".to_string()])
        );
        assert_eq!(cfg.logo.as_deref(), Some("arch"));
        assert_eq!(cfg.no_logo, Some(true));
        assert_eq!(cfg.disk_path.as_deref(), Some("/home#1"));
        let plugins = cfg.plugins.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "Tag #1");
        assert_eq!(plugins[0].command, "echo ok # literal");
    }
}
