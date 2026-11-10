use std::process::Command;

/// Applies the website whitelist lockdown across supported browsers on Windows, macOS, and Linux,
/// blocking all websites (`*`) except the specified `allowed_domains` and their subdomains,
/// and opens the primary contest website in the browser.
pub fn enable_browser_lockdown(
    allowed_domains: &[String],
    block_all_downloads: bool,
) -> Result<String, String> {
    if allowed_domains.is_empty() {
        return Err("At least one allowed domain must be specified.".to_string());
    }

    let download_restriction_val = if block_all_downloads { 3 } else { 1 };

    if cfg!(target_os = "windows") {
        enable_windows_lockdown(allowed_domains, download_restriction_val)?;
    } else if cfg!(target_os = "macos") {
        enable_macos_lockdown(allowed_domains, download_restriction_val)?;
    } else if cfg!(target_os = "linux") {
        enable_linux_lockdown(allowed_domains, download_restriction_val)?;
    } else {
        return Err("Unsupported operating system.".to_string());
    }

    let _ = open_allowed_website(&allowed_domains[0]);
    let summary = allowed_domains
        .iter()
        .map(|d| format!("https://{}", d))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "Browser policy lockdown enabled. Allowed: {}",
        summary
    ))
}

/// Removes all browser policy restrictions and restores normal web access across platforms.
pub fn disable_browser_lockdown() -> Result<String, String> {
    if cfg!(target_os = "windows") {
        disable_windows_lockdown()?;
    } else if cfg!(target_os = "macos") {
        disable_macos_lockdown()?;
    } else if cfg!(target_os = "linux") {
        disable_linux_lockdown()?;
    } else {
        return Err("Unsupported operating system.".to_string());
    }

    Ok("All browser lockdown policies removed. Normal internet access restored.".to_string())
}

/// Opens a whitelisted contest URL in the system default or Chromium browser.
pub fn open_allowed_website(allowed_domain: &str) -> Result<(), String> {
    let url = if allowed_domain.starts_with("http://") || allowed_domain.starts_with("https://") {
        allowed_domain.to_string()
    } else {
        format!("https://{}", allowed_domain)
    };

    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    } else {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ============================================================================
// WINDOWS IMPLEMENTATION
// ============================================================================
fn enable_windows_lockdown(domains: &[String], download_restriction: u32) -> Result<(), String> {
    let chromium_paths = [
        r"HKLM\SOFTWARE\Policies\Google\Chrome",
        r"HKLM\SOFTWARE\Policies\Microsoft\Edge",
        r"HKLM\SOFTWARE\Policies\BraveSoftware\Brave",
        r"HKLM\SOFTWARE\Policies\Vivaldi",
    ];

    for path in chromium_paths {
        let _ = run_reg(&["delete", &format!(r"{}\URLBlocklist", path), "/f"]);
        let _ = run_reg(&["delete", &format!(r"{}\URLAllowlist", path), "/f"]);

        run_reg(&[
            "add",
            &format!(r"{}\URLBlocklist", path),
            "/v",
            "1",
            "/t",
            "REG_SZ",
            "/d",
            "*",
            "/f",
        ])?;

        let mut idx = 1;
        for domain in domains {
            run_reg(&[
                "add",
                &format!(r"{}\URLAllowlist", path),
                "/v",
                &idx.to_string(),
                "/t",
                "REG_SZ",
                "/d",
                domain,
                "/f",
            ])?;
            idx += 1;

            run_reg(&[
                "add",
                &format!(r"{}\URLAllowlist", path),
                "/v",
                &idx.to_string(),
                "/t",
                "REG_SZ",
                "/d",
                &format!("*.{}", domain),
                "/f",
            ])?;
            idx += 1;
        }

        run_reg(&[
            "add",
            path,
            "/v",
            "DownloadRestrictions",
            "/t",
            "REG_DWORD",
            "/d",
            &download_restriction.to_string(),
            "/f",
        ])?;
    }

    // Firefox WebsiteFilter & Download policies
    let ff = r"HKLM\SOFTWARE\Policies\Mozilla\Firefox";
    let _ = run_reg(&["delete", &format!(r"{}\WebsiteFilter", ff), "/f"]);
    if download_restriction == 3 {
        let _ = run_reg(&[
            "add",
            ff,
            "/v",
            "DisableDownloads",
            "/t",
            "REG_DWORD",
            "/d",
            "1",
            "/f",
        ]);
    } else {
        let _ = run_reg(&["delete", ff, "/v", "DisableDownloads", "/f"]);
    }
    let _ = run_reg(&[
        "add",
        &format!(r"{}\WebsiteFilter", ff),
        "/v",
        "Enabled",
        "/t",
        "REG_DWORD",
        "/d",
        "1",
        "/f",
    ]);
    let _ = run_reg(&[
        "add",
        &format!(r"{}\WebsiteFilter\Block", ff),
        "/v",
        "1",
        "/t",
        "REG_SZ",
        "/d",
        "<all_urls>",
        "/f",
    ]);

    let mut ff_idx = 1;
    for domain in domains {
        let _ = run_reg(&[
            "add",
            &format!(r"{}\WebsiteFilter\Exceptions", ff),
            "/v",
            &ff_idx.to_string(),
            "/t",
            "REG_SZ",
            "/d",
            &format!("*://*.{}/*", domain),
            "/f",
        ]);
        ff_idx += 1;
        let _ = run_reg(&[
            "add",
            &format!(r"{}\WebsiteFilter\Exceptions", ff),
            "/v",
            &ff_idx.to_string(),
            "/t",
            "REG_SZ",
            "/d",
            &format!("*://{}/*", domain),
            "/f",
        ]);
        ff_idx += 1;
    }

    // Block portable & unauthorized browsers via DisallowRun for current user + all standard user profiles
    let exp = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer";
    let _ = run_reg(&["add", exp, "/v", "DisallowRun", "/t", "REG_DWORD", "/d", "1", "/f"]);
    let blocked_exes = [
        "opera.exe",
        "tor.exe",
        "waterfox.exe",
        "maxthon.exe",
        "IronPortable.exe",
        "FirefoxPortable.exe",
        "GoogleChromePortable.exe",
        "OperaPortable.exe",
        "BravePortable.exe",
    ];
    for (idx, exe) in blocked_exes.iter().enumerate() {
        let _ = run_reg(&[
            "add",
            &format!(r"{}\DisallowRun", exp),
            "/v",
            &(idx + 1).to_string(),
            "/t",
            "REG_SZ",
            "/d",
            exe,
            "/f",
        ]);
    }

    // Restart running browsers so policies apply immediately
    let _ = Command::new("taskkill")
        .args([
            "/F",
            "/IM",
            "chrome.exe",
            "/IM",
            "msedge.exe",
            "/IM",
            "brave.exe",
            "/IM",
            "vivaldi.exe",
            "/IM",
            "firefox.exe",
            "/IM",
            "opera.exe",
        ])
        .output();

    Ok(())
}

fn disable_windows_lockdown() -> Result<(), String> {
    let chromium_paths = [
        r"HKLM\SOFTWARE\Policies\Google\Chrome",
        r"HKLM\SOFTWARE\Policies\Microsoft\Edge",
        r"HKLM\SOFTWARE\Policies\BraveSoftware\Brave",
        r"HKLM\SOFTWARE\Policies\Vivaldi",
    ];

    for path in chromium_paths {
        let _ = run_reg(&["delete", &format!(r"{}\URLBlocklist", path), "/f"]);
        let _ = run_reg(&["delete", &format!(r"{}\URLAllowlist", path), "/f"]);
        let _ = run_reg(&["delete", path, "/v", "DownloadRestrictions", "/f"]);
    }

    let _ = run_reg(&[
        "delete",
        r"HKLM\SOFTWARE\Policies\Mozilla\Firefox\WebsiteFilter",
        "/f",
    ]);
    let _ = run_reg(&[
        "delete",
        r"HKLM\SOFTWARE\Policies\Mozilla\Firefox",
        "/v",
        "DisableDownloads",
        "/f",
    ]);
    let _ = run_reg(&[
        "delete",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer",
        "/v",
        "DisallowRun",
        "/f",
    ]);
    let _ = run_reg(&[
        "delete",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer\DisallowRun",
        "/f",
    ]);

    Ok(())
}

fn run_reg(args: &[&str]) -> Result<(), String> {
    let status = Command::new("reg")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute reg.exe: {}", e))?;

    if status.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Registry modification failed (please run as Administrator): {}",
            String::from_utf8_lossy(&status.stderr)
        ))
    }
}

// ============================================================================
// MACOS IMPLEMENTATION
// ============================================================================
fn enable_macos_lockdown(domains: &[String], download_restriction: u32) -> Result<(), String> {
    let mut plist_allow_args = Vec::new();
    let mut ff_exceptions = Vec::new();
    for d in domains {
        plist_allow_args.push(format!("'{}'", d));
        plist_allow_args.push(format!("'*.{}'", d));
        ff_exceptions.push(format!("\"*://{0}/*\",\"*://*.{0}/*\"", d));
    }
    let allow_array_str = plist_allow_args.join(" ");
    let ff_json = format!(
        r#"{{"policies":{{"WebsiteFilter":{{"Block":["<all_urls>"],"Exceptions":[{}]}}}}}}"#,
        ff_exceptions.join(",")
    );

    let script = format!(
        "for DOMAIN in com.google.Chrome com.brave.Browser com.microsoft.edgemac com.vivaldi.Vivaldi; do \
         defaults write /Library/Preferences/$DOMAIN URLBlocklist -array '*'; \
         defaults write /Library/Preferences/$DOMAIN URLAllowlist -array {0}; \
         defaults write /Library/Preferences/$DOMAIN DownloadRestrictions -int {1}; \
         done; \
         if [ -d '/Applications/Firefox.app/Contents/Resources' ]; then \
         mkdir -p '/Applications/Firefox.app/Contents/Resources/distribution'; \
         echo '{2}' > '/Applications/Firefox.app/Contents/Resources/distribution/policies.json'; \
         fi; \
         killall 'Google Chrome' 'Brave Browser' 'Microsoft Edge' 'Vivaldi' 'firefox' 2>/dev/null || true",
        allow_array_str, download_restriction, ff_json
    );

    run_macos_privileged(&script)
}

fn disable_macos_lockdown() -> Result<(), String> {
    let script = "for DOMAIN in com.google.Chrome com.brave.Browser com.microsoft.edgemac com.vivaldi.Vivaldi; do \
         defaults delete /Library/Preferences/$DOMAIN URLBlocklist 2>/dev/null || true; \
         defaults delete /Library/Preferences/$DOMAIN URLAllowlist 2>/dev/null || true; \
         defaults delete /Library/Preferences/$DOMAIN DownloadRestrictions 2>/dev/null || true; \
         done; \
         rm -f '/Applications/Firefox.app/Contents/Resources/distribution/policies.json' 2>/dev/null || true; \
         killall 'Google Chrome' 'Brave Browser' 'Microsoft Edge' 'Vivaldi' 'firefox' 2>/dev/null || true";

    run_macos_privileged(script)
}

fn run_macos_privileged(shell_cmd: &str) -> Result<(), String> {
    let osa = format!(
        "do shell script \"{}\" with administrator privileges",
        shell_cmd.replace('\"', "\\\"")
    );
    let output = Command::new("osascript")
        .args(["-e", &osa])
        .output()
        .map_err(|e| format!("Failed to invoke osascript: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Administrator permission denied or command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

// ============================================================================
// LINUX IMPLEMENTATION
// ============================================================================
fn enable_linux_lockdown(domains: &[String], download_restriction: u32) -> Result<(), String> {
    let mut allow_entries = Vec::new();
    let mut ff_exceptions = Vec::new();
    for d in domains {
        allow_entries.push(format!("\"{}\"", d));
        allow_entries.push(format!("\"*.{}\"", d));
        ff_exceptions.push(format!("\"*://{0}/*\",\"*://*.{0}/*\"", d));
    }

    let json = format!(
        r#"{{"URLBlocklist":["*"],"URLAllowlist":[{}],"DownloadRestrictions":{}}}"#,
        allow_entries.join(","),
        download_restriction
    );
    let ff_json = format!(
        r#"{{"policies":{{"WebsiteFilter":{{"Block":["<all_urls>"],"Exceptions":[{}]}}}}}}"#,
        ff_exceptions.join(",")
    );

    let script = format!(
        "for DIR in /etc/opt/chrome/policies/managed /etc/chromium/policies/managed /etc/brave/policies/managed /etc/opt/edge/policies/managed; do \
         mkdir -p \"$DIR\" && echo '{0}' > \"$DIR/whitelist.json\"; \
         done; \
         mkdir -p /etc/firefox/policies && echo '{1}' > /etc/firefox/policies/policies.json; \
         killall chrome google-chrome chromium brave brave-browser microsoft-edge firefox 2>/dev/null || true",
        json, ff_json
    );

    run_linux_privileged(&script)
}

fn disable_linux_lockdown() -> Result<(), String> {
    let script = "for DIR in /etc/opt/chrome/policies/managed /etc/chromium/policies/managed /etc/brave/policies/managed /etc/opt/edge/policies/managed; do \
         rm -f \"$DIR/whitelist.json\"; \
         done; \
         rm -f /etc/firefox/policies/policies.json 2>/dev/null || true";

    run_linux_privileged(script)
}

fn run_linux_privileged(shell_cmd: &str) -> Result<(), String> {
    let output = Command::new("pkexec")
        .args(["sh", "-c", shell_cmd])
        .output()
        .or_else(|_| Command::new("sudo").args(["sh", "-c", shell_cmd]).output())
        .map_err(|e| format!("Failed to execute root policy command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Linux policy change failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
