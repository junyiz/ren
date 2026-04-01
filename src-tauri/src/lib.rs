mod config;
mod proxy;

use config::{ProxyConfig, load_config, save_config, encrypt_api_key};
use proxy::ProxyServer;
use std::sync::Mutex;
use std::process::{Command, Stdio};
use std::time::Duration;
use tauri::State;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

struct AppState {
    server: Mutex<Option<ProxyServer>>,
    config: Mutex<ProxyConfig>,
    tunnel: Mutex<Option<TunnelState>>,
}

struct TunnelState {
    process: std::process::Child,
    public_url: String,
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
    let config = state.config.lock().unwrap().clone();
    let mut server_guard = state.server.lock().unwrap();

    // Stop tunnel if running
    let mut tunnel_guard = state.tunnel.lock().unwrap();
    if let Some(mut tunnel) = tunnel_guard.take() {
        info!("Stopping tunnel: {}", tunnel.public_url);
        tunnel.process.kill().ok();
    }
    drop(tunnel_guard);

    if let Some(server) = server_guard.take() {
        server.shutdown();

        // Wait for the server to release the port
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Verify port is released by trying to bind
        match std::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)) {
            Ok(listener) => {
                drop(listener);
                info!("Proxy stopped, port {} released", config.port);
            }
            Err(e) => {
                info!("Proxy stopped, port {} may still be in use: {}", config.port, e);
            }
        }

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

// Get the platform-specific tunelo binary, downloading if necessary
fn get_tunelo_binary() -> Result<std::path::PathBuf, String> {
    // First check if tunelo is available in PATH
    let tunelo_check = Command::new("tunelo")
        .arg("--version")
        .output();

    if let Ok(output) = tunelo_check {
        if output.status.success() {
            eprintln!("[DEBUG] tunelo found in PATH");
            return Ok(std::path::PathBuf::from("tunelo"));
        }
    }

    eprintln!("[DEBUG] tunelo not in PATH, will download");

    // Determine platform
    let (os, arch) = {
        #[cfg(target_os = "linux")]
        {
            ("linux", "amd64")
        }
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "aarch64")]
            {("macos", "arm64")}
            #[cfg(not(target_arch = "aarch64"))]
            {("macos", "amd64")}
        }
        #[cfg(target_os = "windows")]
        {
            ("windows", "amd64")
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            return Err("Unsupported platform".to_string());
        }
    };

    let binary_name = format!("tunelo-{}-{}", os, arch);
    #[cfg(target_os = "windows")]
    let binary_name = format!("{}.exe", binary_name);

    let download_url = format!("https://ren.im/tunelo/{}", binary_name);

    info!("Downloading tunelo from {}", download_url);

    // Create temp directory
    let temp_dir = std::env::temp_dir().join("ren-tunelo");
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let binary_path = temp_dir.join(&binary_name);

    eprintln!("[DEBUG] binary_path: {:?}", binary_path);

    // Download the binary using blocking reqwest
    let client = reqwest::blocking::Client::new();
    let response = client.get(&download_url)
        .send()
        .map_err(|e| format!("Failed to download tunelo: {}", e))?;

    let bytes = response.bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    std::fs::write(&binary_path, &bytes)
        .map_err(|e| format!("Failed to write binary: {}", e))?;

    // Set executable permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path)
            .map_err(|e| format!("Failed to get permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    info!("tunelo downloaded to {:?}", binary_path);
    Ok(binary_path)
}

#[tauri::command]
fn start_tunnel(state: State<AppState>, port: u16, relay: String) -> Result<String, String> {
    let mut tunnel_guard = state.tunnel.lock().unwrap();

    if tunnel_guard.is_some() {
        return Err("Tunnel already running".to_string());
    }

    info!("Starting tunelo tunnel for port {} via {}", port, relay);

    // Get tunelo binary (downloads if necessary)
    let tunelo_path = get_tunelo_binary()?;

    // Build tunelo command with relay
    let relay_arg = format!("{}:4433", relay);

    // Start tunelo process
    let mut child = Command::new(&tunelo_path)
        .args(["port", &port.to_string(), "--relay", &relay_arg])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start tunelo: {}", e))?;

    // Take stdout to read from it
    let mut stdout = child.stdout.take().ok_or("Failed to capture tunelo output")?;

    // Read tunelo output with timeout (in a separate thread to avoid blocking)
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = [0u8; 4096];
        let mut public_url = String::new();
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(30);

        loop {
            if start.elapsed() > timeout {
                break;
            }
            match stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                        // Log the output
                        for line in s.lines() {
                            info!("tunelo: {}", line);
                            if line.contains("Public URL:") {
                                public_url = line.split("Public URL:").nth(1)
                                    .map(|s| s.trim().to_string())
                                    .unwrap_or_default();
                                break;
                            }
                        }
                        if !public_url.is_empty() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = tx.send(public_url);
    });

    // Wait for the URL with a timeout
    let public_url = match rx.recv_timeout(Duration::from_secs(35)) {
        Ok(url) => url,
        Err(_) => {
            child.kill().ok();
            return Err("Timeout waiting for tunnel URL".to_string());
        }
    };

    if public_url.is_empty() {
        child.kill().ok();
        return Err("Failed to get public URL from tunelo".to_string());
    }

    tunnel_guard.replace(TunnelState {
        process: child,
        public_url: public_url.clone(),
    });

    info!("Tunnel started: {}", public_url);
    Ok(public_url)
}

#[tauri::command]
fn stop_tunnel(state: State<AppState>) -> Result<(), String> {
    let mut tunnel_guard = state.tunnel.lock().unwrap();

    if let Some(mut tunnel) = tunnel_guard.take() {
        info!("Stopping tunnel: {}", tunnel.public_url);
        tunnel.process.kill().map_err(|e| format!("Failed to stop tunnel: {}", e))?;
        info!("Tunnel stopped");
        Ok(())
    } else {
        Err("Tunnel not running".to_string())
    }
}

#[tauri::command]
fn get_tunnel_status(state: State<AppState>) -> Result<Option<String>, String> {
    let tunnel_guard = state.tunnel.lock().unwrap();
    Ok(tunnel_guard.as_ref().map(|t| t.public_url.clone()))
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
            tunnel: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_proxy_config,
            start_proxy,
            stop_proxy,
            get_proxy_status,
            get_local_ip,
            start_tunnel,
            stop_tunnel,
            get_tunnel_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
