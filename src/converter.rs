use crate::extractor::Article;
use lol_html::{element, rewrite_str, RewriteStrSettings};
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
    htmd::convert(&rewritten).unwrap_or_else(|e| format!("<!-- htmd error: {e} -->\n{rewritten}"))
}

/// Rewrite the `src` attribute of every real `<img>` element to its locally
/// downloaded path. Uses lol_html so the rewrite is scoped to actual element
/// attributes — text inside `<code>`/`<pre>` blocks, or stray substrings that
/// happen to look like `src="…"`, are left untouched.
fn rewrite_image_srcs(html: &str, base_url: &Url, asset_map: &AssetMap) -> String {
    let result = rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("img[src]", |el| {
                if let Some(src) = el.get_attribute("src") {
                    if !src.starts_with("data:") {
                        if let Ok(resolved) = base_url.join(&src) {
                            if let Some(local) = asset_map.get(resolved.as_str()) {
                                el.set_attribute("src", local).ok();
                            }
                        }
                    }
                }
                Ok(())
            })],
            ..RewriteStrSettings::default()
        },
    );
    result.unwrap_or_else(|_| html.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url() -> Url {
        Url::parse("https://example.com/page").unwrap()
    }

    fn article(html: &str) -> Article {
        Article {
            title: "T".into(),
            content_html: html.into(),
        }
    }

    #[test]
    fn collects_unique_absolute_image_urls() {
        let a =
            article(r#"<p><img src="/a.png"><img src="/a.png"><img src="https://cdn/x.jpg"></p>"#);
        let urls = collect_image_urls(&a, &url());
        assert_eq!(urls.len(), 2);
        assert!(urls
            .iter()
            .any(|u| u.as_str() == "https://example.com/a.png"));
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

    #[test]
    fn does_not_rewrite_src_inside_code_samples() {
        // A real <img> that should be rewritten plus a <code> block whose text
        // happens to contain the same `src="/a.png"` substring. The encoded
        // text inside <code> must survive untouched.
        let html = concat!(
            r#"<p><img src="/a.png" alt="real"></p>"#,
            r#"<pre><code>&lt;img src=&quot;/a.png&quot;&gt;</code></pre>"#,
        );
        let mut map = AssetMap::new();
        map.insert("https://example.com/a.png".into(), "assets/0-a.png".into());
        let md = to_markdown(&article(html), &url(), &map);

        // Real img got rewritten.
        assert!(md.contains("assets/0-a.png"), "got: {md}");
        // Code sample preserves the original literal src.
        assert!(
            md.contains(r#"<img src="/a.png">"#),
            "code sample was mangled; got: {md}"
        );
    }

    #[test]
    fn does_not_rewrite_other_attributes_with_same_url() {
        // An <a href="/a.png"> sharing the URL of a real <img src="/a.png">
        // must not be touched.
        let html = concat!(
            r#"<p><img src="/a.png" alt="real"></p>"#,
            r#"<p><a href="/a.png">link</a></p>"#,
        );
        let mut map = AssetMap::new();
        map.insert("https://example.com/a.png".into(), "assets/0-a.png".into());
        let md = to_markdown(&article(html), &url(), &map);

        assert!(md.contains("assets/0-a.png"));
        // The link must still point at the original URL.
        assert!(md.contains("/a.png)"), "got: {md}");
    }
}
