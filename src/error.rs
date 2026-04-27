use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, W2mError>;

#[derive(Debug, thiserror::Error)]
pub enum W2mError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Chrome not found. Set CHROME_PATH or install Chrome/Chromium.")]
    ChromeNotFound,

    #[error("Chrome rendering failed: {0}")]
    Render(String),

    #[error("could not extract main content (try --selector)")]
    ExtractionEmpty,

    #[error(
        "page looks like an empty SPA shell and rendering is disabled; \
         remove --no-render (or `no_render = true` in config), or pass --render"
    )]
    EmptySpaWithoutRender,

    #[error("output path already exists and is not empty: {0}")]
    OutputExists(PathBuf),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("invalid CSS selector: {0}")]
    InvalidSelector(String),

    #[error("extraction failed: {0}")]
    ExtractionFailed(String),
}

impl W2mError {
    /// Map error variants to process exit codes per the spec.
    pub fn exit_code(&self) -> i32 {
        match self {
            W2mError::Http(_) => 2,
            W2mError::ChromeNotFound | W2mError::Render(_) => 3,
            W2mError::ExtractionEmpty
            | W2mError::ExtractionFailed(_)
            | W2mError::EmptySpaWithoutRender => 4,
            _ => 1,
        }
    }
}
