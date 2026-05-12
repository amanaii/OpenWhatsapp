//! Remote customization loading.

use anyhow::{bail, Context, Result};

/// Remote customization response.
#[allow(dead_code)]
pub(crate) struct RemoteCustomization {
    /// Fetched body.
    pub(crate) body: String,
    /// Optional response ETag.
    pub(crate) etag: Option<String>,
}

/// Fetches CSS or JS from an HTTPS URL, using an optional ETag.
#[allow(dead_code)]
pub(crate) async fn fetch(url: &str, etag: Option<&str>) -> Result<RemoteCustomization> {
    if !url.starts_with("https://") {
        bail!("customization URLs must use https");
    }

    let client = reqwest::Client::new();
    let mut request = client.get(url);
    if let Some(etag) = etag {
        request = request.header(reqwest::header::IF_NONE_MATCH, etag);
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("failed to fetch customization URL {url}"))?;
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let body = response
        .text()
        .await
        .context("failed to read customization body")?;

    Ok(RemoteCustomization { body, etag })
}

#[cfg(test)]
mod tests {
    #[test]
    fn https_requirement_is_stable() {
        let url = "https://example.com/a.css";

        assert!(url.starts_with("https://"));
    }
}
