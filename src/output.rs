use crate::error::{Result, W2mError};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

pub struct Meta<'a> {
    pub title: &'a str,
    pub source_url: &'a str,
    pub fetched_at: DateTime<Utc>,
    pub render_mode: RenderMode,
}

#[derive(Debug, Clone, Copy)]
pub enum RenderMode {
    Static,
    Headless,
}

impl RenderMode {
    fn as_str(self) -> &'static str {
        match self {
            RenderMode::Static => "static",
            RenderMode::Headless => "headless",
        }
    }
}

pub fn write_bundle(dir: &Path, body_md: &str, meta: &Meta) -> Result<()> {
    if dir.exists() {
        let mut entries = fs::read_dir(dir)?;
        if entries.next().is_some() {
            return Err(W2mError::OutputExists(dir.to_path_buf()));
        }
    } else {
        fs::create_dir_all(dir)?;
    }

    let frontmatter = format!(
        "---\ntitle: {}\nsource_url: {}\nfetched_at: {}\nrender_mode: {}\n---\n\n",
        yaml_escape(meta.title),
        meta.source_url,
        meta.fetched_at.to_rfc3339(),
        meta.render_mode.as_str(),
    );

    let mut full = String::with_capacity(frontmatter.len() + body_md.len());
    full.push_str(&frontmatter);
    full.push_str(body_md);

    fs::write(dir.join("index.md"), full)?;
    Ok(())
}

fn yaml_escape(s: &str) -> String {
    // Simple, safe quoting: always wrap in double quotes and escape "/\.
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn meta() -> (String, String) {
        ("Hello".to_string(), "https://example.com/x".to_string())
    }

    #[test]
    fn writes_index_with_frontmatter() {
        let dir = TempDir::new().unwrap();
        let (title, url) = meta();
        let m = Meta {
            title: &title,
            source_url: &url,
            fetched_at: "2026-04-25T22:00:00Z".parse().unwrap(),
            render_mode: RenderMode::Static,
        };
        write_bundle(dir.path(), "# Body\n", &m).unwrap();

        let content = std::fs::read_to_string(dir.path().join("index.md")).unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("title: \"Hello\""));
        assert!(content.contains("render_mode: static"));
        assert!(content.ends_with("# Body\n"));
    }

    #[test]
    fn errors_when_dir_non_empty() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("existing.txt"), "x").unwrap();
        let (title, url) = meta();
        let m = Meta {
            title: &title,
            source_url: &url,
            fetched_at: Utc::now(),
            render_mode: RenderMode::Static,
        };
        let err = write_bundle(dir.path(), "x", &m).unwrap_err();
        matches!(err, W2mError::OutputExists(_));
    }

    #[test]
    fn yaml_escape_handles_quotes() {
        assert_eq!(yaml_escape(r#"a"b\c"#), r#""a\"b\\c""#);
    }
}
