use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::Rng;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Config not found")]
    NotFound,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProxyConfig {
    pub provider: String,        // "openai", "anthropic", "ollama"
    pub api_key: String,         // Encrypted
    pub port: u16,
    pub upstream_url: String,    // e.g., "https://api.openai.com/v1"
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: String::new(),
            port: 8080,
            upstream_url: "https://api.openai.com".to_string(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ren-desktop")
        .join("config.json")
}

fn get_key() -> [u8; 32] {
    // Derive key from machine-specific data
    let mut key = [0u8; 32];
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "default".to_string());
    let key_input = format!("ren-desktop-{}", hostname);
    key[..key_input.len().min(32)].copy_from_slice(&key_input.as_bytes()[..key_input.len().min(32)]);
    key
}

pub fn encrypt_api_key(api_key: &str) -> Result<String, ConfigError> {
    let key = get_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ConfigError::Encryption(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, api_key.as_bytes())
        .map_err(|e| ConfigError::Encryption(e.to_string()))?;

    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(BASE64.encode(&result))
}

pub fn decrypt_api_key(encrypted: &str) -> Result<String, ConfigError> {
    let key = get_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ConfigError::Encryption(e.to_string()))?;

    let data = BASE64.decode(encrypted)
        .map_err(|e| ConfigError::Encryption(e.to_string()))?;

    if data.len() < 12 {
        return Err(ConfigError::Encryption("Invalid encrypted data".to_string()));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| ConfigError::Encryption(e.to_string()))?;

    String::from_utf8(plaintext)
        .map_err(|e| ConfigError::Encryption(e.to_string()))
}

pub fn load_config() -> Result<ProxyConfig, ConfigError> {
    let path = get_config_path();
    if !path.exists() {
        return Ok(ProxyConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: ProxyConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &ProxyConfig) -> Result<(), ConfigError> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let api_key = "sk-test123";
        let encrypted = encrypt_api_key(api_key).unwrap();
        let decrypted = decrypt_api_key(&encrypted).unwrap();
        assert_eq!(decrypted, api_key);
    }
}