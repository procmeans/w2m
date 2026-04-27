use std::fs;
use tempfile::TempDir;
use url::Url;
use w2m::error::W2mError;
use w2m::pipeline::{run, Opts, RenderStrategy};
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
        render: RenderStrategy::Disabled,
        selector: None,
        no_assets: false,
        concurrency: 2,
        wait_ms: 0,
    };

    let summary = run(url, opts).await.unwrap();
    assert!(summary.bytes_written > 0);
    assert_eq!(summary.images_downloaded, 1);
    assert_eq!(summary.images_attempted, 1);

    let md = fs::read_to_string(dir.path().join("index.md")).unwrap();
    assert!(md.contains("---\n"));
    assert!(md.contains("render_mode: static"));
    assert!(md.contains("Body content"));
    assert!(md.contains("assets/"));

    let mut entries = fs::read_dir(dir.path().join("assets")).unwrap();
    assert!(entries.next().is_some());
}

#[tokio::test]
async fn empty_spa_shell_with_no_render_returns_clear_error() {
    // The exact failure mode that used to surface as "try --selector": a
    // page that's an empty React shell, with rendering disabled.
    let server = MockServer::start().await;
    let shell = r#"<!doctype html><html><head><title>App</title></head>
        <body><div id="root"></div></body></html>"#;
    Mock::given(method("GET"))
        .and(path("/app"))
        .respond_with(ResponseTemplate::new(200).set_body_string(shell))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/app", server.uri())).unwrap();
    let dir = TempDir::new().unwrap();
    let opts = Opts {
        out_dir: dir.path().to_path_buf(),
        render: RenderStrategy::Disabled,
        selector: None,
        no_assets: false,
        concurrency: 2,
        wait_ms: 0,
    };

    let err = run(url, opts).await.unwrap_err();
    assert!(
        matches!(err, W2mError::EmptySpaWithoutRender),
        "got: {err:?}"
    );
}

#[tokio::test]
async fn precheck_blocks_run_before_any_download() {
    // Pre-populate the destination with index.md. The pipeline must reject
    // it before issuing any HTTP request — so a missing mock is fine.
    let server = MockServer::start().await;
    // Intentionally no mocks: if precheck fails to short-circuit, the
    // request will 404 and we'll get an Http error instead.
    let url = Url::parse(&format!("{}/page", server.uri())).unwrap();
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("index.md"), "old content").unwrap();

    let opts = Opts {
        out_dir: dir.path().to_path_buf(),
        render: RenderStrategy::Disabled,
        selector: None,
        no_assets: false,
        concurrency: 2,
        wait_ms: 0,
    };

    let err = run(url, opts).await.unwrap_err();
    assert!(matches!(err, W2mError::OutputExists(_)), "got: {err:?}");
    // Old content was not overwritten.
    let content = fs::read_to_string(dir.path().join("index.md")).unwrap();
    assert_eq!(content, "old content");
    // No assets dir was created.
    assert!(!dir.path().join("assets").exists());
}
