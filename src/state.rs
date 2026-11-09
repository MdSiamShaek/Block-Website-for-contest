use chrono::Local;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Stores the active contest lockdown state on disk so that even if the app is
/// closed and reopened, the same session password set when locking is required to unlock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockdownSession {
    pub is_locked: bool,
    pub allowed_domain: String,
    pub password_sha256: String,
    pub selected_ide: String,
    pub started_at: String,
    pub recording_path: Option<String>,
}

impl Default for LockdownSession {
    fn default() -> Self {
        Self {
            is_locked: false,
            allowed_domain: "bapsoj.org".to_string(),
            password_sha256: String::new(),
            selected_ide: "Visual Studio Code".to_string(),
            started_at: String::new(),
            recording_path: None,
        }
    }
}

/// Computes an uppercase hex SHA-256 digest of a password string.
pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02X}", b)).collect()
}

/// Verifies whether a candidate plaintext password matches the stored SHA-256 hex digest.
pub fn verify_password(candidate: &str, expected_sha256: &str) -> bool {
    if expected_sha256.is_empty() {
        return false;
    }
    hash_password(candidate).eq_ignore_ascii_case(expected_sha256)
}

/// Normalizes a user-entered URL or domain into a clean domain string (e.g. "https://bapsoj.org/" -> "bapsoj.org").
pub fn normalize_domain(input: &str) -> String {
    let trimmed = input.trim();
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let without_path = without_scheme.split('/').next().unwrap_or(without_scheme);
    let without_www = without_path.strip_prefix("www.").unwrap_or(without_path);
    without_www.trim().to_lowercase()
}

/// Path to the persistent session state file next to the executable or working directory.
pub fn session_file_path() -> PathBuf {
    PathBuf::from("session_lock.json")
}

impl LockdownSession {
    /// Creates a new locked session using the dynamically entered password.
    pub fn new_locked(domain: &str, plain_password: &str, selected_ide: &str, recording_path: Option<String>) -> Self {
        let clean_domain = normalize_domain(domain);
        Self {
            is_locked: true,
            allowed_domain: if clean_domain.is_empty() {
                "bapsoj.org".to_string()
            } else {
                clean_domain
            },
            password_sha256: hash_password(plain_password),
            selected_ide: selected_ide.to_string(),
            started_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            recording_path,
        }
    }

    /// Loads any existing active session from `session_lock.json`.
    pub fn load() -> Self {
        let path = session_file_path();
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(session) = serde_json::from_str::<LockdownSession>(&data) {
                return session;
            }
        }
        Self::default()
    }

    /// Saves the current session state to `session_lock.json`.
    pub fn save(&self) -> Result<(), String> {
        let path = session_file_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Clears the persisted session lock file once unlocked.
    pub fn clear() -> Result<(), String> {
        let path = session_file_path();
        if path.exists() {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
