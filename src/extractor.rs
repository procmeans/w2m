use crate::error::{Result, W2mError};
use scraper::{Html, Selector};
use url::Url;

#[derive(Debug)]
pub struct Article {
    pub title: String,
    pub content_html: String,
}

pub fn extract(html: &str, base_url: &Url, selector: Option<&str>) -> Result<Article> {
    if let Some(sel) = selector {
        return extract_with_selector(html, sel);
    }
    extract_with_readability(html, base_url)
}

fn extract_with_selector(html: &str, sel: &str) -> Result<Article> {
    let doc = Html::parse_document(html);
    let title = doc
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|t| t.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let parsed = Selector::parse(sel)
        .map_err(|e| W2mError::InvalidSelector(format!("{e:?}")))?;
    let node = doc.select(&parsed).next().ok_or(W2mError::ExtractionEmpty)?;
    let content_html = node.html();

    if content_html.trim().is_empty() {
        return Err(W2mError::ExtractionEmpty);
    }

    Ok(Article { title, content_html })
}

fn extract_with_readability(html: &str, base_url: &Url) -> Result<Article> {
    let mut bytes = std::io::Cursor::new(html.as_bytes().to_vec());
    let product = readability::extractor::extract(&mut bytes, base_url)
        .map_err(|e| W2mError::ExtractionFailed(format!("readability: {e}")))?;

    if product.content.trim().is_empty() {
        return Err(W2mError::ExtractionEmpty);
    }

    // Check if the body is empty (empty SPA shells return bare HTML structure)
    let doc = Html::parse_document(&product.content);
    let body_selector = Selector::parse("body").unwrap();
    if let Some(body) = doc.select(&body_selector).next() {
        let body_text = body.text().collect::<String>();
        if body_text.trim().is_empty() {
            return Err(W2mError::ExtractionEmpty);
        }
    }

    Ok(Article {
        title: product.title,
        content_html: product.content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url() -> Url { Url::parse("https://example.com/page").unwrap() }
    fn fixture(name: &str) -> String {
        std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap()
    }

    #[test]
    fn readability_finds_article_body() {
        let html = fixture("article.html");
        let a = extract(&html, &url(), None).unwrap();
        assert!(a.content_html.contains("paragraph of real content"));
        // navigation and footer should not appear
        assert!(!a.content_html.contains("Site footer"));
    }

    #[test]
    fn selector_override_picks_chosen_node() {
        let html = fixture("article.html");
        let a = extract(&html, &url(), Some("article")).unwrap();
        assert!(a.content_html.contains("paragraph of real content"));
        assert_eq!(a.title, "Test Article");
    }

    #[test]
    fn empty_spa_returns_extraction_empty() {
        let html = fixture("empty_spa.html");
        let err = extract(&html, &url(), None).unwrap_err();
        assert!(matches!(err, W2mError::ExtractionEmpty | W2mError::ExtractionFailed(_)));
    }
}
