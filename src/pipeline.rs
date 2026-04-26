use scraper::{Html, Selector};

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
