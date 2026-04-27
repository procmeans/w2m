use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing_subscriber::EnvFilter;
use url::Url;
use w2m::cli::Cli;
use w2m::config::{Config, HostRules};
use w2m::output::RenderMode;
use w2m::pipeline::{run, Opts, Summary};

const DEFAULT_CONCURRENCY: usize = 8;
const DEFAULT_WAIT_MS: u64 = 2000;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let url = match Url::parse(&cli.url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("error: invalid URL '{}': {e}", cli.url);
            return ExitCode::from(1);
        }
    };

    let config = match load_config(cli.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };

    let host = url.host_str().unwrap_or("").to_string();
    let rules = config.rules_for(&host);

    let opts = resolve_opts(&cli, &rules, &url);

    match run(url, opts).await {
        Ok(summary) => {
            print_summary(&summary);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

fn print_summary(s: &Summary) {
    let render = match (s.render_mode, s.render_duration) {
        (RenderMode::Static, _) => "static".to_string(),
        (RenderMode::Headless, Some(d)) => format!("headless ({:.1}s)", d.as_secs_f64()),
        (RenderMode::Headless, None) => "headless".to_string(),
    };
    let skipped = s.images_attempted.saturating_sub(s.images_downloaded);
    let images = if s.images_attempted == 0 {
        "none".to_string()
    } else if skipped == 0 {
        format!("{} downloaded", s.images_downloaded)
    } else {
        format!("{} downloaded, {} skipped", s.images_downloaded, skipped)
    };

    eprintln!("\n\u{2713} Saved to {}", s.out_dir.display());
    eprintln!("  title    {}", s.title);
    eprintln!("  render   {render}");
    eprintln!("  size     {}", human_size(s.bytes_written));
    eprintln!("  images   {images}");
}

fn human_size(bytes: u64) -> String {
    const K: f64 = 1024.0;
    const M: f64 = K * 1024.0;
    let b = bytes as f64;
    if b >= M {
        format!("{:.1} MB", b / M)
    } else if b >= K {
        format!("{:.1} KB", b / K)
    } else {
        format!("{bytes} B")
    }
}

fn load_config(explicit: Option<&std::path::Path>) -> std::io::Result<Config> {
    if let Some(p) = explicit {
        return Config::load_from(p);
    }
    if let Some(p) = Config::default_path() {
        return Config::load_from(&p);
    }
    Ok(Config::default())
}

fn resolve_opts(cli: &Cli, rules: &HostRules, url: &Url) -> Opts {
    // Priority for each field: CLI flag > config rule > built-in default.
    let force_render = cli.render || rules.render.unwrap_or(false);
    let no_render = cli.no_render || rules.no_render.unwrap_or(false);
    let selector = cli
        .selector
        .clone()
        .or_else(|| rules.selector.clone());
    let no_assets = cli.no_assets || rules.no_assets.unwrap_or(false);
    let concurrency = cli
        .concurrency
        .or(rules.concurrency)
        .unwrap_or(DEFAULT_CONCURRENCY);
    let wait_ms = cli
        .wait_ms
        .or(rules.wait_ms)
        .unwrap_or(DEFAULT_WAIT_MS);
    let out_dir = cli
        .out
        .clone()
        .unwrap_or_else(|| default_out_dir(url));

    Opts {
        out_dir,
        force_render,
        no_render,
        selector,
        no_assets,
        concurrency,
        wait_ms,
    }
}

fn init_tracing(verbosity: u8) {
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("w2m={level}")));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

fn default_out_dir(url: &Url) -> PathBuf {
    let host = url.host_str().unwrap_or("page");
    let path = url
        .path()
        .trim_matches('/')
        .replace('/', "-");
    let slug = if path.is_empty() {
        host.to_string()
    } else {
        format!("{host}-{path}")
    };
    PathBuf::from(slug)
}
