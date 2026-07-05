use anyhow::{Context, Result};
use serde::Deserialize;

/// Production Railway host — client auto-discovers ws URL from /config.
pub const DEFAULT_PRODUCTION_HOST: &str =
    "unique-transformation-production-44b4.up.railway.app";

#[derive(Debug, Deserialize)]
struct ServerConfig {
    #[serde(rename = "wsUrl")]
    ws_url: String,
}

fn is_placeholder(value: &str) -> bool {
    value.is_empty() || value.contains("your-app")
}

fn is_usable_ws_url(url: &str) -> bool {
    !is_placeholder(url) && (url.starts_with("ws://") || url.starts_with("wss://"))
}

async fn fetch_ws_url(api_host: &str) -> Result<String> {
    let config_url = if api_host.starts_with("http://") || api_host.starts_with("https://") {
        format!("{}/config", api_host.trim_end_matches('/'))
    } else {
        format!("https://{api_host}/config")
    };

    let config: ServerConfig = reqwest::get(&config_url)
        .await
        .with_context(|| format!("fetch {config_url}"))?
        .error_for_status()
        .with_context(|| format!("config endpoint returned error ({config_url})"))?
        .json()
        .await
        .with_context(|| format!("parse config JSON from {config_url}"))?;

    Ok(config.ws_url)
}

/// Resolve WebSocket URL: CLI flag → REALM_SERVER → auto-discover from production /config → localhost.
pub async fn resolve_server_url(cli_override: Option<String>) -> Result<String> {
    if let Some(url) = cli_override {
        if is_usable_ws_url(&url) {
            return Ok(url);
        }
    }

    if let Ok(url) = std::env::var("REALM_SERVER") {
        if is_usable_ws_url(&url) {
            return Ok(url);
        }
    }

    let api_host = std::env::var("REALM_API")
        .ok()
        .filter(|h| !is_placeholder(h))
        .unwrap_or_else(|| DEFAULT_PRODUCTION_HOST.to_string());

    if let Ok(ws_url) = fetch_ws_url(&api_host).await {
        eprintln!("→ Connecting to Realm of Echoes ({ws_url})");
        return Ok(ws_url);
    }

    eprintln!("→ Could not reach {api_host} — trying local server...");
    Ok("ws://localhost:4242/ws".into())
}