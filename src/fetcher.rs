use crate::error::Result;
use url::Url;

pub async fn fetch_static(url: &Url) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("w2m/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client.get(url.clone()).send().await?.error_for_status()?;
    let body = resp.text().await?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetches_body_from_mock() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/p"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>hi</html>"))
            .mount(&server)
            .await;

        let url = Url::parse(&format!("{}/p", server.uri())).unwrap();
        let body = fetch_static(&url).await.unwrap();
        assert_eq!(body, "<html>hi</html>");
    }

    #[tokio::test]
    async fn http_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let url = Url::parse(&format!("{}/missing", server.uri())).unwrap();
        assert!(fetch_static(&url).await.is_err());
    }
}
