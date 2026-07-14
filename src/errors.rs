use thiserror::Error;

#[derive(Debug, Error)]
pub enum RssError {
    #[error("Error requesting rss feed {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Error parsing rss feed {0}")]
    ParseError(#[from] rss::Error),
    #[error("Error parsing html in contents {0}")]
    HtmlError(#[from] html2text::Error),
}

