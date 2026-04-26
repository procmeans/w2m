use crate::extractor::Article;
use scraper::{Html, Selector};
use std::collections::HashMap;
use url::Url;

/// Map from original absolute image URL → local relative path (e.g. "assets/foo.png").
pub type AssetMap = HashMap<String, String>;

pub fn collect_image_urls(article: &Article, base_url: &Url) -> Vec<Url> {
    let doc = Html::parse_fragment(&article.content_html);
    let sel = Selector::parse("img[src]").unwrap();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for el in doc.select(&sel) {
        let src = match el.value().attr("src") {
            Some(s) => s,
            None => continue,
        };
        if src.starts_with("data:") {
            continue;
        }
        let resolved = match base_url.join(src) {
            Ok(u) => u,
            Err(_) => continue,
        };
        let key = resolved.to_string();
        if seen.insert(key) {
            out.push(resolved);
        }
    }
    out
}

pub fn to_markdown(article: &Article, base_url: &Url, asset_map: &AssetMap) -> String {
    let rewritten = rewrite_image_srcs(&article.content_html, base_url, asset_map);
    htmd::convert(&rewritten)
        .unwrap_or_else(|e| format!("<!-- htmd error: {e} -->\n{rewritten}"))
}

fn rewrite_image_srcs(html: &str, base_url: &Url, asset_map: &AssetMap) -> String {
    // Lightweight regex-free rewrite: parse, modify, re-serialize.
    // scraper does not allow easy mutation, so we do a string replacement
    // pass per <img> element discovered during parsing.
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("img[src]").unwrap();
    let mut out = html.to_string();
    for el in doc.select(&sel) {
        let original = match el.value().attr("src") {
            Some(s) => s,
            None => continue,
        };
        if original.starts_with("data:") {
            continue;
        }
        let resolved = match base_url.join(original) {
            Ok(u) => u.to_string(),
            Err(_) => continue,
        };
        if let Some(local) = asset_map.get(&resolved) {
            // Replace the *original* attribute value as it appears in source HTML.
            // This is a best-effort string substitution; if the same src appears
            // with both quote styles we replace each occurrence.
            for quote in ['"', '\''] {
                let needle = format!("src={q}{s}{q}", q = quote, s = original);
                let replacement = format!("src={q}{l}{q}", q = quote, l = local);
                out = out.replace(&needle, &replacement);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url() -> Url { Url::parse("https://example.com/page").unwrap() }

    fn article(html: &str) -> Article {
        Article { title: "T".into(), content_html: html.into() }
    }

    #[test]
    fn collects_unique_absolute_image_urls() {
        let a = article(
            r#"<p><img src="/a.png"><img src="/a.png"><img src="https://cdn/x.jpg"></p>"#,
        );
        let urls = collect_image_urls(&a, &url());
        assert_eq!(urls.len(), 2);
        assert!(urls.iter().any(|u| u.as_str() == "https://example.com/a.png"));
        assert!(urls.iter().any(|u| u.as_str() == "https://cdn/x.jpg"));
    }

    #[test]
    fn skips_data_uri_images() {
        let a = article(r#"<p><img src="data:image/png;base64,AAAA"></p>"#);
        let urls = collect_image_urls(&a, &url());
        assert!(urls.is_empty());
    }

    #[test]
    fn rewrites_known_assets_in_markdown() {
        let a = article(r#"<p><img src="/a.png" alt="A"></p>"#);
        let mut map = AssetMap::new();
        map.insert("https://example.com/a.png".into(), "assets/a.png".into());
        let md = to_markdown(&a, &url(), &map);
        assert!(md.contains("assets/a.png"), "got: {md}");
        assert!(!md.contains("/a.png\""));
    }

    #[test]
    fn leaves_unmapped_images_alone() {
        let a = article(r#"<p><img src="https://cdn/y.jpg"></p>"#);
        let md = to_markdown(&a, &url(), &AssetMap::new());
        assert!(md.contains("cdn/y.jpg"));
    }
}
