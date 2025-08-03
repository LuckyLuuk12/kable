// secure_token.rs
// Cross-platform AES encryption for sensitive tokens
// Key is generated per user and stored in the Kable app directory with restricted permissions

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

const KEY_FILENAME: &str = "token.key";
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

/// Get the path to the key file in the Kable app directory
fn get_key_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    #[cfg(target_os = "windows")]
    let kable_dir = home_dir
        .join("AppData")
        .join("Roaming")
        .join("kable-launcher");
    #[cfg(target_os = "macos")]
    let kable_dir = home_dir
        .join("Library")
        .join("Application Support")
        .join("kable-launcher");
    #[cfg(target_os = "linux")]
    let kable_dir = home_dir.join(".kable-launcher");
    kable_dir.join(KEY_FILENAME)
}

/// Generate and store a new random key if not present
fn get_or_create_key() -> std::io::Result<[u8; KEY_SIZE]> {
    let key_path = get_key_path();
    if key_path.exists() {
        let mut key = [0u8; KEY_SIZE];
        let mut file = OpenOptions::new().read(true).open(&key_path)?;
        file.read_exact(&mut key)?;
        Ok(key)
    } else {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&key_path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            file.set_permissions(perms)?;
        }
        file.write_all(&key)?;
        Ok(key)
    }
}

/// Encrypt a token string
pub fn encrypt_token(token: &str) -> Result<String, String> {
    let key_bytes = get_or_create_key().map_err(|e| format!("Key error: {}", e))?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, token.as_bytes())
        .map_err(|e| format!("Encrypt error: {}", e))?;
    // Store nonce + ciphertext as base64
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

/// Decrypt a token string
pub fn decrypt_token(data: &str) -> Result<String, String> {
    let key_bytes = get_or_create_key().map_err(|e| format!("Key error: {}", e))?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let combined = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    if combined.len() < NONCE_SIZE {
        return Err("Invalid encrypted token format".to_string());
    }
    let nonce = Nonce::from_slice(&combined[..NONCE_SIZE]);
    let ciphertext = &combined[NONCE_SIZE..];
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decrypt error: {}", e))?;
    String::from_utf8(plaintext).map_err(|e| format!("UTF8 error: {}", e))
}
