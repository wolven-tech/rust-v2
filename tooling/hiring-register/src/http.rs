//! One HTTP client for every job board and careers page.

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::Value;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Careers sites behind bot protection serve a browser and refuse a scripted
/// client, so the register presents itself as one. Without this, a third of
/// the links in the file read as broken when they are not.
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
                          (KHTML, like Gecko) Chrome/140.0 Safari/537.36";

/// How many requests run at once. The original read twelve at a time and no
/// board complained, so the number is kept rather than re-derived.
pub const CONCURRENCY: usize = 12;

#[must_use]
pub fn client() -> Client {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
        .expect("a client with no TLS backend configured cannot be built")
}

pub async fn get_json(client: &Client, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| describe(&e))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }
    response
        .json()
        .await
        .map_err(|e| format!("unreadable JSON: {e}"))
}

pub async fn get_page(client: &Client, url: &str) -> Result<String, String> {
    let response = client
        .get(url)
        .header("Accept", "text/html")
        .send()
        .await
        .map_err(|e| describe(&e))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }
    response
        .text()
        .await
        .map_err(|e| format!("unreadable page: {e}"))
}

/// The status and the URL a request landed on, which is what a link check
/// records.
pub async fn head_or_get(client: &Client, url: &str) -> Result<(StatusCode, String), String> {
    let response = client.get(url).send().await.map_err(|e| describe(&e))?;
    let landed = response.url().to_string();
    Ok((response.status(), landed))
}

fn describe(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        "no response (timeout)".to_string()
    } else if err.is_connect() {
        "no response (connect)".to_string()
    } else if let Some(status) = err.status() {
        format!("HTTP {status}")
    } else {
        "no response".to_string()
    }
}
