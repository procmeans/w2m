use crate::assets;
use crate::converter::{self, AssetMap};
use crate::error::{Result, W2mError};
use crate::extractor;
use crate::fetcher;
use crate::output::{self, Meta, RenderMode};
use crate::renderer::{self, RenderOpts};
use chrono::Utc;
use scraper::{Html, Selector};
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

pub struct Opts {
    pub out_dir: PathBuf,
    pub force_render: bool,
    pub no_render: bool,
    pub selector: Option<String>,
    pub no_assets: bool,
    pub concurrency: usize,
    pub wait_ms: u64,
}

fn render_opts(wait_ms: u64) -> RenderOpts {
    RenderOpts {
        timeout: Duration::from_secs(30),
        settle: Duration::from_millis(wait_ms),
    }
}

pub async fn run(url: Url, opts: Opts) -> Result<()> {
    let render = render_opts(opts.wait_ms);

    let (html, render_mode) = if opts.force_render {
        let html = renderer::render_dynamic(&url, &render).await?;
        (html, RenderMode::Headless)
    } else {
        let static_html = fetcher::fetch_static(&url).await?;
        if !opts.no_render && looks_like_empty_spa(&static_html) {
            tracing::info!("static HTML looks like an SPA shell; falling back to headless");
            let html = renderer::render_dynamic(&url, &render).await?;
            (html, RenderMode::Headless)
        } else {
            (static_html, RenderMode::Static)
        }
    };

    let article = match extractor::extract(&html, &url, opts.selector.as_deref()) {
        Ok(a) => a,
        Err(W2mError::ExtractionEmpty) if !opts.force_render && !opts.no_render => {
            tracing::info!("extraction empty; retrying with headless render");
            let rendered = renderer::render_dynamic(&url, &render).await?;
            extractor::extract(&rendered, &url, opts.selector.as_deref())?
        }
        Err(e) => return Err(e),
    };

    let asset_map: AssetMap = if opts.no_assets {
        AssetMap::new()
    } else {
        let urls = converter::collect_image_urls(&article, &url);
        assets::download_all(urls, &opts.out_dir, opts.concurrency).await?
    };

    let md = converter::to_markdown(&article, &url, &asset_map);

    let meta = Meta {
        title: &article.title,
        source_url: url.as_str(),
        fetched_at: Utc::now(),
        render_mode,
    };
    output::write_bundle(&opts.out_dir, &md, &meta)?;
    Ok(())
}

pub(crate) fn looks_like_empty_spa(html: &str) -> bool {
    let body_text = body_text_chars(html);

    // Sparse body, regardless of mount point.
    if body_text < 200 {
        return true;
    }

    // Has typical SPA mount + sparse-ish content.
    let has_mount = html.contains(r#"id="root""#)
        || html.contains(r#"id="app""#)
        || html.contains(r#"id="__next""#);
    if has_mount && body_text < 500 {
        return true;
    }

    false
}

fn body_text_chars(html: &str) -> usize {
    let doc = Html::parse_document(html);
    let body_sel = Selector::parse("body").unwrap();
    let body = match doc.select(&body_sel).next() {
        Some(b) => b,
        None => return 0,
    };
    body.text()
        .flat_map(str::chars)
        .filter(|c| !c.is_whitespace())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap()
    }

    #[test]
    fn detects_empty_spa() {
        assert!(looks_like_empty_spa(&fixture("empty_spa.html")));
    }

    #[test]
    fn detects_nextjs_shell() {
        assert!(looks_like_empty_spa(&fixture("nextjs_shell.html")));
    }

    #[test]
    fn does_not_flag_real_article() {
        assert!(!looks_like_empty_spa(&fixture("article.html")));
    }
}
