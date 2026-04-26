use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "w2m", version, about = "Download a web page and convert it to Markdown")]
pub struct Cli {
    /// URL of the page to convert.
    pub url: String,

    /// Output directory. Defaults to a slug derived from the URL.
    #[arg(short, long)]
    pub out: Option<PathBuf>,

    /// Force headless Chrome rendering.
    #[arg(long, conflicts_with = "no_render")]
    pub render: bool,

    /// Disable headless fallback (static fetch only).
    #[arg(long)]
    pub no_render: bool,

    /// CSS selector for the main content (overrides readability).
    #[arg(long)]
    pub selector: Option<String>,

    /// Skip image downloads, keep original URLs in Markdown.
    #[arg(long)]
    pub no_assets: bool,

    /// Parallel image downloads.
    #[arg(long, default_value_t = 8)]
    pub concurrency: usize,

    /// Increase verbosity (-v debug, -vv trace).
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_minimal() {
        let cli = Cli::try_parse_from(["w2m", "https://example.com"]).unwrap();
        assert_eq!(cli.url, "https://example.com");
        assert!(cli.out.is_none());
        assert!(!cli.render);
        assert_eq!(cli.concurrency, 8);
    }

    #[test]
    fn parses_full_flags() {
        let cli = Cli::try_parse_from([
            "w2m", "https://example.com", "-o", "out", "--selector", "main",
            "--no-assets", "--concurrency", "4", "-vv",
        ])
        .unwrap();
        assert_eq!(cli.out.as_deref(), Some(std::path::Path::new("out")));
        assert_eq!(cli.selector.as_deref(), Some("main"));
        assert!(cli.no_assets);
        assert_eq!(cli.concurrency, 4);
        assert_eq!(cli.verbose, 2);
    }

    #[test]
    fn render_and_no_render_conflict() {
        let result = Cli::try_parse_from(["w2m", "https://example.com", "--render", "--no-render"]);
        assert!(result.is_err());
    }
}
