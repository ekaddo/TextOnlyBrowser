use anyhow::{anyhow, Context, Result};
use reqwest::{Client, redirect};
use std::time::Duration;

pub fn build_client(timeout_secs: u64, max_redirects: usize) -> Result<Client> {
    Client::builder()
        .user_agent("TextOnlyBrowser/0.1 (text-only news reader)")
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(redirect::Policy::limited(max_redirects))
        .build()
        .context("failed to build HTTP client")
}

/// Returns (final_url, html_body).
pub async fn fetch(client: &Client, url: &str) -> Result<(String, String)> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to fetch {url}"))?;

    let final_url = response.url().to_string();

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.contains("text/html") && !content_type.is_empty() {
        return Err(anyhow!(
            "cannot display '{content_type}' — only text/html is supported"
        ));
    }

    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!("HTTP {} for {url}", status));
    }

    let body = response
        .text()
        .await
        .with_context(|| format!("failed to read response body from {url}"))?;

    Ok((final_url, body))
}
