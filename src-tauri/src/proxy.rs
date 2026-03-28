use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error};

use crate::config::{decrypt_api_key, ProxyConfig};

#[derive(Clone)]
pub struct ProxyServer {
    pub config: ProxyConfig,
    http_client: Client,
    shutdown_signal: Arc<std::sync::atomic::AtomicBool>,
}

impl ProxyServer {
    pub fn new(config: ProxyConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            shutdown_signal: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn shutdown(&self) {
        self.shutdown_signal.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.port));
        let listener = TcpListener::bind(addr).await?;
        info!("Proxy server listening on {}", addr);

        // Get local IP for display
        if let Ok(local_ip) = local_ip_address::local_ip() {
            info!("Local IP: {}", local_ip);
            info!("Proxy URL: http://{}:{}/v1", local_ip, self.config.port);
        }

        loop {
            if self.shutdown_signal.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Shutdown signal received");
                break;
            }

            match listener.accept().await {
                Ok((mut stream, client_addr)) => {
                    let config = self.config.clone();
                    let client = self.http_client.clone();
                    let shutdown = self.shutdown_signal.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(client_addr, stream, &config, client, shutdown).await {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection(
    client_addr: SocketAddr,
    mut stream: tokio::net::TcpStream,
    config: &ProxyConfig,
    client: Client,
    _shutdown: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Connection from {}", client_addr);

    // Read HTTP request
    let mut buffer = [0u8; 8192];
    let n = stream.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..n]);
    let lines: Vec<&str> = request.lines().collect();

    if lines.is_empty() {
        return Ok(());
    }

    // Parse request line
    let request_line = lines.first().ok_or("Empty request")?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    info!("{} {} {}", method, path, client_addr);

    // Only handle /v1/* endpoints
    if !path.starts_with("/v1/") && path != "/v1" {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nOnly /v1/* endpoints are supported";
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    // Parse client headers
    let mut client_headers: Vec<(&str, &str)> = Vec::new();
    let mut body_start = 0;
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            continue; // skip request line
        }
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            client_headers.push((key.trim(), value.trim()));
        }
    }

    let body = lines[body_start..].join("\r\n");

    // Get API key
    let api_key = decrypt_api_key(&config.api_key)
        .map_err(|e| format!("Failed to decrypt API key: {}", e))?;

    // Build upstream URL, avoiding /v1 duplication
    let upstream_base = config.upstream_url.trim_end_matches('/');
    let forward_path = if upstream_base.ends_with("/v1") {
        // upstream already has /v1, strip /v1 prefix from client path
        path.strip_prefix("/v1").unwrap_or(path)
    } else {
        path
    };
    let upstream_url = format!("{}{}", upstream_base, forward_path);
    info!("Forwarding to: {}", upstream_url);

    // Build upstream request with client headers forwarded
    let auth_value = format!("Bearer {}", api_key);
    let mut req_builder = client
        .request(reqwest::Method::from_bytes(method.as_bytes()).unwrap(), &upstream_url);

    // Forward original client headers (skip hop-by-hop and host headers)
    let skip_headers = ["host", "connection", "transfer-encoding", "authorization"];
    for (key, value) in &client_headers {
        if !skip_headers.contains(&key.to_lowercase().as_str()) {
            req_builder = req_builder.header(*key, *value);
        }
    }

    // Always set Authorization with our decrypted API key
    req_builder = req_builder.header("Authorization", &auth_value);

    let upstream_response = req_builder
        .body(body)
        .send()
        .await?;

    let status = upstream_response.status();
    let headers = upstream_response.headers().clone();

    // Write response back to client
    let mut response = format!("HTTP/1.1 {} {}\r\n", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"));

    for (key, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            response.push_str(&format!("{}: {}\r\n", key, v));
        }
    }
    response.push_str("\r\n");

    stream.write_all(response.as_bytes()).await?;

    // Stream the body
    let body_bytes = upstream_response.bytes().await?;
    stream.write_all(&body_bytes).await?;

    info!("Response sent to {}", client_addr);

    Ok(())
}