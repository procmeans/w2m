use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// On-disk shape: ~/.config/w2m/config.toml
///
/// ```toml
/// [defaults]
/// wait_ms = 2500
/// concurrency = 8
///
/// [hosts."open.oceanengine.com"]
/// render = true
/// wait_ms = 5000
/// selector = ".doc-content-body"
/// ```
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: HostRules,
    #[serde(default)]
    pub hosts: HashMap<String, HostRules>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct HostRules {
    pub render: Option<bool>,
    pub no_render: Option<bool>,
    pub selector: Option<String>,
    pub no_assets: Option<bool>,
    pub concurrency: Option<usize>,
    pub wait_ms: Option<u64>,
}

impl Config {
    /// Resolve the config path. Tries, in order:
    ///   1. `$HOME/.config/w2m/config.toml`           (preferred, used by gh/nvim/etc.)
    ///   2. platform-native location via `dirs::config_dir`
    ///      (`~/Library/Application Support/w2m/config.toml` on macOS,
    ///       `$XDG_CONFIG_HOME/w2m/config.toml` on Linux,
    ///       `%APPDATA%\w2m\config.toml` on Windows)
    /// Returns the first one that exists; otherwise the preferred path
    /// (so a future write goes there).
    pub fn default_path() -> Option<PathBuf> {
        let preferred = dirs::home_dir().map(|h| h.join(".config").join("w2m").join("config.toml"));
        let native = dirs::config_dir().map(|d| d.join("w2m").join("config.toml"));

        match (&preferred, &native) {
            (Some(p), _) if p.exists() => Some(p.clone()),
            (_, Some(n)) if n.exists() => Some(n.clone()),
            (Some(p), _) => Some(p.clone()),
            (None, n) => n.clone(),
        }
    }

    /// Load from `path`. If `path` doesn't exist, returns `Config::default()`.
    /// Parse errors are surfaced.
    pub fn load_from(path: &std::path::Path) -> std::io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("config parse error in {}: {e}", path.display()),
                )
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    /// Merge `defaults` then the host-specific rules (if any) into a single
    /// `HostRules`. Host-specific values win over defaults.
    pub fn rules_for(&self, host: &str) -> HostRules {
        let mut merged = self.defaults.clone();
        if let Some(host_rules) = self.hosts.get(host) {
            if host_rules.render.is_some()       { merged.render = host_rules.render; }
            if host_rules.no_render.is_some()    { merged.no_render = host_rules.no_render; }
            if host_rules.selector.is_some()     { merged.selector = host_rules.selector.clone(); }
            if host_rules.no_assets.is_some()    { merged.no_assets = host_rules.no_assets; }
            if host_rules.concurrency.is_some()  { merged.concurrency = host_rules.concurrency; }
            if host_rules.wait_ms.is_some()      { merged.wait_ms = host_rules.wait_ms; }
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn missing_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let cfg = Config::load_from(&dir.path().join("nope.toml")).unwrap();
        assert!(cfg.hosts.is_empty());
        assert!(cfg.defaults.wait_ms.is_none());
    }

    #[test]
    fn parses_full_example() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("c.toml");
        std::fs::write(
            &path,
            r#"
[defaults]
wait_ms = 2500
concurrency = 4

[hosts."open.oceanengine.com"]
render = true
wait_ms = 5000
selector = ".doc-content-body"
"#,
        )
        .unwrap();
        let cfg = Config::load_from(&path).unwrap();
        assert_eq!(cfg.defaults.wait_ms, Some(2500));
        assert_eq!(cfg.defaults.concurrency, Some(4));

        let rules = cfg.rules_for("open.oceanengine.com");
        // host wait_ms wins over defaults
        assert_eq!(rules.wait_ms, Some(5000));
        // defaults still apply where host doesn't override
        assert_eq!(rules.concurrency, Some(4));
        assert_eq!(rules.render, Some(true));
        assert_eq!(rules.selector.as_deref(), Some(".doc-content-body"));
    }

    #[test]
    fn unknown_host_falls_back_to_defaults() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("c.toml");
        std::fs::write(&path, "[defaults]\nwait_ms = 1234\n").unwrap();
        let cfg = Config::load_from(&path).unwrap();
        let rules = cfg.rules_for("not-listed.com");
        assert_eq!(rules.wait_ms, Some(1234));
        assert!(rules.render.is_none());
    }

    #[test]
    fn invalid_toml_is_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "this is not = valid toml [[").unwrap();
        let err = Config::load_from(&path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
