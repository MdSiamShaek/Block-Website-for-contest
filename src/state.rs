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
    #[serde(default)]
    pub extra_domains: Vec<String>,
    pub password_sha256: String,
    pub selected_ide: String,
    #[serde(default)]
    pub selected_ides: Vec<String>,
    pub started_at: String,
    #[serde(default)]
    pub started_unix_secs: u64,
    #[serde(default)]
    pub duration_minutes: u32,
    #[serde(default)]
    pub block_all_downloads: bool,
    pub recording_path: Option<String>,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
}

fn default_dark_mode() -> bool {
    true
}

impl Default for LockdownSession {
    fn default() -> Self {
        Self {
            is_locked: false,
            allowed_domain: "bapsoj.org".to_string(),
            extra_domains: Vec::new(),
            password_sha256: String::new(),
            selected_ide: "Visual Studio Code".to_string(),
            selected_ides: vec!["Visual Studio Code".to_string()],
            started_at: String::new(),
            started_unix_secs: 0,
            duration_minutes: 0,
            block_all_downloads: false,
            recording_path: None,
            dark_mode: true,
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

/// Evaluates password strength on a 0..=4 scale with a human-readable label.
pub fn evaluate_password_strength(password: &str) -> (u8, &'static str) {
    if password.is_empty() {
        return (0, "None");
    }
    let len = password.chars().count();
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    let variety = [has_lower, has_upper, has_digit, has_symbol]
        .iter()
        .filter(|&&b| b)
        .count();

    if len < 4 {
        (1, "Weak")
    } else if len < 8 || variety <= 1 {
        (2, "Fair")
    } else if len >= 8 && variety >= 3 {
        (4, "Strong")
    } else {
        (3, "Good")
    }
}

/// Normalizes a user-entered URL or domain into a clean domain string (e.g. "https://bapsoj.org/" -> "bapsoj.org").
pub fn normalize_domain(input: &str) -> String {
    let trimmed = input.trim();
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let without_path = without_scheme.split('/').next().unwrap_or(without_scheme);
    let without_port = without_path.split(':').next().unwrap_or(without_path);
    let without_www = without_port.strip_prefix("www.").unwrap_or(without_port);
    without_www.trim().to_lowercase()
}

/// Path to the persistent session state file next to the executable or working directory.
pub fn session_file_path() -> PathBuf {
    PathBuf::from("session_lock.json")
}

impl LockdownSession {
    /// Creates a new locked session using the dynamically entered password and multiple selected IDEs.
    #[allow(clippy::too_many_arguments)]
    pub fn new_locked(
        domain: &str,
        extra_domains: Vec<String>,
        plain_password: &str,
        selected_ides: Vec<String>,
        duration_minutes: u32,
        block_all_downloads: bool,
        recording_path: Option<String>,
        dark_mode: bool,
    ) -> Self {
        let clean_domain = normalize_domain(domain);
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let primary_ide = selected_ides
            .first()
            .cloned()
            .unwrap_or_else(|| "Visual Studio Code".to_string());

        Self {
            is_locked: true,
            allowed_domain: if clean_domain.is_empty() {
                "bapsoj.org".to_string()
            } else {
                clean_domain
            },
            extra_domains,
            password_sha256: hash_password(plain_password),
            selected_ide: primary_ide,
            selected_ides,
            started_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            started_unix_secs: now_secs,
            duration_minutes,
            block_all_downloads,
            recording_path,
            dark_mode,
        }
    }

    /// Returns the list of selected IDE names (falling back to `selected_ide` for older session files).
    pub fn all_selected_ides(&self) -> Vec<String> {
        if !self.selected_ides.is_empty() {
            self.selected_ides.clone()
        } else if !self.selected_ide.is_empty() {
            vec![self.selected_ide.clone()]
        } else {
            vec!["Visual Studio Code".to_string()]
        }
    }

    /// Returns all whitelisted domains (primary + extras, deduplicated).
    pub fn all_allowed_domains(&self) -> Vec<String> {
        let mut list = vec![self.allowed_domain.clone()];
        for d in &self.extra_domains {
            let clean = normalize_domain(d);
            if !clean.is_empty() && !list.contains(&clean) {
                list.push(clean);
            }
        }
        list
    }

    /// Returns remaining seconds if a session timer was configured (`duration_minutes > 0`).
    pub fn remaining_timer_secs(&self) -> Option<u64> {
        if !self.is_locked || self.duration_minutes == 0 || self.started_unix_secs == 0 {
            return None;
        }
        let total_secs = (self.duration_minutes as u64) * 60;
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(self.started_unix_secs);
        let elapsed = now_secs.saturating_sub(self.started_unix_secs);
        Some(total_secs.saturating_sub(elapsed))
    }

    /// Loads any existing active session from `session_lock.json`.
    pub fn load() -> Self {
        let path = session_file_path();
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(mut session) = serde_json::from_str::<LockdownSession>(&data) {
                if session.selected_ides.is_empty() && !session.selected_ide.is_empty() {
                    session.selected_ides = vec![session.selected_ide.clone()];
                }
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
