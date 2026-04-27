use crate::converter::AssetMap;
use crate::error::Result;
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use url::Url;

pub async fn download_all(urls: Vec<Url>, out_dir: &Path, concurrency: usize) -> Result<AssetMap> {
    if urls.is_empty() {
        return Ok(AssetMap::new());
    }
    let assets_dir = out_dir.join("assets");
    tokio::fs::create_dir_all(&assets_dir).await?;

    let client = reqwest::Client::builder()
        .user_agent(concat!("w2m/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let assets_dir_owned = assets_dir.clone();
    let results: Vec<(String, Option<String>)> = stream::iter(urls.into_iter().enumerate())
        .map(|(idx, u)| {
            let client = client.clone();
            let dir = assets_dir_owned.clone();
            async move {
                let key = u.to_string();
                let local = match download_one(&client, &u, &dir, idx).await {
                    Ok(p) => Some(p),
                    Err(e) => {
                        tracing::warn!("asset download failed for {}: {}", u, e);
                        None
                    }
                };
                (key, local)
            }
        })
        .buffer_unordered(concurrency.max(1))
        .collect()
        .await;

    let mut map = AssetMap::new();
    for (k, v) in results {
        if let Some(local) = v {
            map.insert(k, local);
        }
    }
    Ok(map)
}

async fn download_one(
    client: &reqwest::Client,
    url: &Url,
    assets_dir: &Path,
    idx: usize,
) -> Result<String> {
    let resp = client.get(url.clone()).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    let filename = guess_filename(url, idx);
    let path: PathBuf = assets_dir.join(&filename);
    tokio::fs::write(&path, &bytes).await?;
    Ok(format!("assets/{filename}"))
}

fn guess_filename(url: &Url, idx: usize) -> String {
    let last_seg = url
        .path_segments()
        .and_then(|mut s| s.next_back())
        .filter(|s| !s.is_empty())
        .map(sanitize)
        .unwrap_or_default();
    if last_seg.is_empty() || !last_seg.contains('.') {
        format!("image-{idx}.bin")
    } else {
        format!("{idx}-{last_seg}")
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn filename_keeps_extension() {
        let u = Url::parse("https://x/y/foo.png").unwrap();
        assert_eq!(guess_filename(&u, 0), "0-foo.png");
    }

    #[test]
    fn filename_falls_back_when_no_extension() {
        let u = Url::parse("https://x/y/").unwrap();
        assert_eq!(guess_filename(&u, 3), "image-3.bin");
    }

    #[test]
    fn filename_sanitizes_unsafe_chars() {
        let u = Url::parse("https://x/a%20b!c.png").unwrap();
        let f = guess_filename(&u, 0);
        assert!(f.ends_with(".png"));
        assert!(!f.contains('!'));
    }

    #[tokio::test]
    async fn downloads_image_and_maps_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/img.png"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8, 1, 2]))
            .mount(&server)
            .await;
        let dir = TempDir::new().unwrap();
        let urls = vec![Url::parse(&format!("{}/img.png", server.uri())).unwrap()];
        let map = download_all(urls.clone(), dir.path(), 2).await.unwrap();
        let local = map.get(urls[0].as_str()).unwrap();
        assert!(local.starts_with("assets/"));
        assert!(dir.path().join(local).exists());
    }

    #[tokio::test]
    async fn failures_are_skipped_not_propagated() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let dir = TempDir::new().unwrap();
        let urls = vec![Url::parse(&format!("{}/missing", server.uri())).unwrap()];
        let map = download_all(urls, dir.path(), 2).await.unwrap();
        assert!(map.is_empty());
    }
}
