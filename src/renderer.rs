use crate::error::{Result, W2mError};
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use std::time::Duration;
use url::Url;

pub struct RenderOpts {
    pub timeout: Duration,
    pub settle: Duration,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            settle: Duration::from_millis(2000),
        }
    }
}

pub async fn render_dynamic(url: &Url, opts: &RenderOpts) -> Result<String> {
    let mut builder = BrowserConfig::builder();
    if let Ok(path) = std::env::var("CHROME_PATH") {
        builder = builder.chrome_executable(path);
    }
    let config = builder
        .request_timeout(opts.timeout)
        .build()
        .map_err(|e| W2mError::Render(format!("config: {e}")))?;

    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("Could not find") || msg.contains("No such file") {
                W2mError::ChromeNotFound
            } else {
                W2mError::Render(msg)
            }
        })?;

    let handler_task = tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });

    let result = async {
        let page = browser
            .new_page(url.as_str())
            .await
            .map_err(|e| W2mError::Render(e.to_string()))?;
        page.wait_for_navigation()
            .await
            .map_err(|e| W2mError::Render(e.to_string()))?;
        tokio::time::sleep(opts.settle).await;
        let html = page
            .content()
            .await
            .map_err(|e| W2mError::Render(e.to_string()))?;
        Ok::<_, W2mError>(html)
    }
    .await;

    let _ = browser.close().await;
    handler_task.abort();
    result
}
