# Block-Website-for-contest — Cross-Platform Rust Contest Lockdown & Proctoring Suite

A cross-platform **Rust Desktop GUI application** (`eframe` / `egui`) for programming contests and lab environments. It guides participants or proctors through a **4-Page Step-by-Step Setup Wizard**, styled with the **Neon Sunshine Bliss** color palette (`#0D3B66`, `#FAF0CA`, `#F4D35E`).

---

## 4-Page Step-by-Step Setup Flow (`src/main.rs`)

1. **Page 1 — Contest IDE Installer & Launcher (`src/ide.rs`)**
   - Choose between **Visual Studio Code**, **Code::Blocks**, **CP Editor**, **Sublime Text**, and **JetBrains CLion**.
   - Automatically detects if the selected IDE is installed across Windows (`Program Files`, `LocalAppData`, `PATH`), macOS (`/Applications`, `~/Applications`), and Linux (`PATH`).
   - One-click or automatic background installation (`IdeInstallTracker`) with a live progress spinner and elapsed timer, plus one-click or automatic IDE launch.

2. **Page 2 — Automated Desktop Video Recording (`src/recorder.rs`)**
   - Automatically records the full desktop into timestamped `.mp4` files inside `recordings/` using FFmpeg (`gdigrab` on Windows, dynamically detected `avfoundation` screen index on macOS, `x11grab` on Linux).
   - Displays a live `⏺ REC HH:MM:SS` badge in the top bar, provides an **Open Recordings Folder** button, optional audio capture, a one-click **Install FFmpeg** button, and crash-safe `Drop` cleanup (`q\n` over `stdin`).

3. **Page 3 — Website Whitelist & Browser Policy (`src/policy.rs`)**
   - Whitelists your primary contest domain (e.g. `bapsoj.org`) plus optional one-click reference domains (`cppreference.com`, `codeforces.com`, `atcoder.jp`, `vjudge.net`, or custom domains).
   - Enforces `URLBlocklist = ["*"]` and `URLAllowlist` across **Chrome, Edge, Brave, Vivaldi, and Mozilla Firefox** on Windows (Registry + `DisallowRun`), macOS (`/Library/Preferences` `.plist` + Firefox `policies.json`), and Linux (`/etc/opt/.../managed/whitelist.json` + `/etc/firefox/policies/policies.json`).
   - Includes a configurable **Auto-Unlock Contest Duration** (`Manual Only`, `1 Hour`, `2 Hours`, `3 Hours`, `5 Hours`).

4. **Page 4 — Set Session Password & Activate Lockdown (`src/state.rs`)**
   - Displays a **Session Configuration Summary** of Pages 1–3 with quick **Edit Step** links.
   - Prompts you to set and confirm a **Session Password** (with a `👁 Show` / `🙈 Hide` toggle and live **Password Strength Meter**).
   - Hashes the password in memory with **SHA-256** (`session_lock.json`) and activates the contest lockdown. That exact session password is required to unlock the browsers and stop recording.

---

## Color Palette — "Neon Sunshine Bliss" (`src/theme.rs`)

- **`#0D3B66` (Deep Yale Navy):** Primary structural canvas in **Navy Mode** and high-contrast typography/headers in **Cream Mode**.
- **`#FAF0CA` (Warm Lemon Cream):** Crisp foreground typography in **Navy Mode** and primary background canvas in **Cream Mode**.
- **`#F4D35E` (Sunshine Gold):** Primary interactive buttons, active page tabs, borders, and status highlights.

---

## Building & Running Locally

```bash
# Run in development mode
cargo run

# Build optimized release executable
cargo build --release
```
