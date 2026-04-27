use std::fs;
use tempfile::TempDir;
use url::Url;
use w2m::pipeline::{run, Opts};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn end_to_end_static_with_image_download() {
    let server = MockServer::start().await;

    let html = r#"<!doctype html><html><head><title>Hi</title></head><body>
       <main><article>
         <h1>Hi</h1>
         <p>Body content with enough words to look like a real article so
            readability picks it up. Body body body body body body.</p>
         <p><img src="/img.png" alt="x"></p>
       </article></main>
     </body></html>"#;

    Mock::given(method("GET"))
        .and(path("/page"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/img.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 4]))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/page", server.uri())).unwrap();
    let dir = TempDir::new().unwrap();
    let opts = Opts {
        out_dir: dir.path().to_path_buf(),
        force_render: false,
        no_render: true,
        selector: None,
        no_assets: false,
        concurrency: 2,
        wait_ms: 0,
    };

    run(url, opts).await.unwrap();

    let md = fs::read_to_string(dir.path().join("index.md")).unwrap();
    assert!(md.contains("---\n"));
    assert!(md.contains("render_mode: static"));
    assert!(md.contains("Body content"));
    assert!(md.contains("assets/"));

    let mut entries = fs::read_dir(dir.path().join("assets")).unwrap();
    assert!(entries.next().is_some());
}
