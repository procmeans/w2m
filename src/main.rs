use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing_subscriber::EnvFilter;
use url::Url;
use w2m::cli::Cli;
use w2m::pipeline::{run, Opts};

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

    let out_dir = cli.out.unwrap_or_else(|| default_out_dir(&url));

    let opts = Opts {
        out_dir,
        force_render: cli.render,
        no_render: cli.no_render,
        selector: cli.selector,
        no_assets: cli.no_assets,
        concurrency: cli.concurrency,
    };

    match run(url, opts).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(e.exit_code() as u8)
        }
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
