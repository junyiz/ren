mod config;
mod proxy;

use config::{ProxyConfig, load_config, save_config, encrypt_api_key};
use proxy::ProxyServer;
use std::sync::Mutex;
use tauri::State;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

struct AppState {
    server: Mutex<Option<ProxyServer>>,
    config: Mutex<ProxyConfig>,
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Result<ProxyConfig, String> {
    let config = state.config.lock().unwrap().clone();
    Ok(config)
}

#[tauri::command]
fn save_proxy_config(
    state: State<AppState>,
    provider: String,
    api_key: String,
    port: u16,
    upstream_url: String,
) -> Result<(), String> {
    let encrypted_key = encrypt_api_key(&api_key).map_err(|e| e.to_string())?;

    let config = ProxyConfig {
        provider,
        api_key: encrypted_key,
        port,
        upstream_url,
    };

    save_config(&config).map_err(|e| e.to_string())?;
    *state.config.lock().unwrap() = config;

    info!("Configuration saved");
    Ok(())
}

#[tauri::command]
fn start_proxy(state: State<AppState>) -> Result<String, String> {
    let config = state.config.lock().unwrap().clone();

    if config.api_key.is_empty() {
        return Err("API key not configured".to_string());
    }

    let mut server_guard = state.server.lock().unwrap();

    if server_guard.is_some() {
        return Err("Proxy already running".to_string());
    }

    // First, try to bind to verify port is available
    let test_socket = std::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .map_err(|e| format!("Port {} is already in use: {}", config.port, e))?;
    test_socket.set_nonblocking(true).ok();
    drop(test_socket); // Release the port immediately

    let server = ProxyServer::new(config);
    let server_clone = std::sync::Arc::new(server);

    // Get local IP for display
    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "localhost".to_string());

    let proxy_url = format!("http://{}:{}/v1", local_ip, server_clone.config.port);

    // Start the server in background
    let server_ref = server_clone.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = server_ref.run().await {
                error!("Proxy error: {}", e);
            }
        });
    });

    // Wait a bit for server to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    *server_guard = Some((*server_clone).clone());

    info!("Proxy started at {}", proxy_url);
    Ok(proxy_url)
}

#[tauri::command]
fn stop_proxy(state: State<AppState>) -> Result<(), String> {
    let mut server_guard = state.server.lock().unwrap();

    if let Some(server) = server_guard.take() {
        server.shutdown();
        info!("Proxy stopped");
        Ok(())
    } else {
        Err("Proxy not running".to_string())
    }
}

#[tauri::command]
fn get_proxy_status(state: State<AppState>) -> Result<bool, String> {
    let server_guard = state.server.lock().unwrap();
    Ok(server_guard.is_some())
}

#[tauri::command]
fn get_local_ip() -> Result<String, String> {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .map_err(|e| e.to_string())
}

fn setup_logging() {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ren-desktop")
        .join("logs");

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "ren-desktop.log",
    );

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(file_appender)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).ok();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logging();
    info!("Ren Desktop starting...");

    let config = load_config().unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            server: Mutex::new(None),
            config: Mutex::new(config),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_proxy_config,
            start_proxy,
            stop_proxy,
            get_proxy_status,
            get_local_ip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}