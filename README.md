# Block-Website-for-contest — Cross-Platform Rust Contest Lockdown & Proctoring Suite

A cross-platform **Rust Desktop GUI application** (`eframe` / `egui`) for programming contests and lab environments. It restricts web browsers to a chosen contest domain (plus optional reference sites like `cppreference.com`), installs and launches a chosen IDE, starts full-screen desktop video recording automatically, and requires a **dynamic per-session password** (hashed with SHA-256) set at lock time to unlock.

---

## Color-Theory Driven GUI (`src/theme.rs`)

The interface is designed around the **60-30-10 Rule**, **Split-Complementary Color Harmony**, and **WCAG AAA Contrast Ratios**:
- **60% Dominant Canvas:** Deep Midnight Slate (`#0B0F19`) in Dark Mode / Cool Alabaster (`#F1F5F9`) in Daylight Mode — low-saturation blue-slate minimizes eye fatigue during 3–5 hour programming contests.
- **30% Secondary Elevated Cards:** Layered Slate Navy (`#161F32`) with subtle `1px` borders (`#2A3854`) and rounded `10px` cards to group controls cleanly into four logical stages.
- **10% Semantic & Interactive Accents:**
  - **Primary Interactive Brand:** Royal Indigo (`#4F46E5`) paired with Sky Cyan (`#38BDF8`) for primary calls-to-action, active borders, and section headers.
  - **Safe / Unlocked / Installed Badges:** Emerald Teal (`#34D399` foreground on `#064E3B` pill fill).
  - **Active Lockdown / Live Recording Badges:** Crimson Rose (`#FB7185` foreground on `#4C0519` pill fill) for instant visual recognition.
  - **Warning / Background Installer Badges:** Warm Amber (`#FBBF24` foreground on `#451A03` pill fill).
- Includes a one-click **Dark / Light Theme Toggle** in the top header bar.

---

## Core Features

1. **Dynamic Per-Session Password Protection (`src/state.rs`)**
   - Every time you enable a lockdown, the app prompts you to set and confirm a **Session Password** (with a Show/Hide `👁` toggle and live **Password Strength Meter**).
   - The password is immediately hashed in memory using **SHA-256** and persisted to `session_lock.json` (plaintext is never stored).
   - Includes a **Window Close Guard** that prevents accidentally closing the app while a contest lockdown is active.

2. **Multi-Domain Browser Policy Lockdown & Contest Timer (`src/policy.rs`)**
   - Whitelists your primary contest domain (e.g. `bapsoj.org`) plus optional one-click reference domains (`cppreference.com`, `codeforces.com`, `atcoder.jp`, `vjudge.net`, or custom domains).
   - **Windows:** Configures `URLBlocklist = ["*"]` and `URLAllowlist` across Chrome, Edge, Brave, and Vivaldi (`HKLM\SOFTWARE\Policies\...`), configures Mozilla Firefox `WebsiteFilter`, sets `DownloadRestrictions`, and blocks portable browsers via Windows Explorer `DisallowRun`.
   - **macOS:** Writes managed preference `.plist` rules under `/Library/Preferences` for Chrome, Brave, Edge, and Vivaldi, plus Firefox `distribution/policies.json`, via native macOS Administrator elevation (`osascript`).
   - **Linux:** Writes managed policy JSON files for Chrome, Chromium, Brave, Edge, and Firefox (`/etc/firefox/policies/policies.json`).
   - **Optional Auto-Unlock Contest Timer:** Choose Manual Only, 1 Hour, 2 Hours, 3 Hours, or 5 Hours with a live countdown badge.

3. **IDE Selector, Live Background Installer & Launcher (`src/ide.rs`)**
   - Supports **Visual Studio Code**, **Code::Blocks**, **CP Editor**, **Sublime Text**, and **JetBrains CLion**.
   - Detects installations across PATH, `/Applications` & `~/Applications` (macOS), and `Program Files` & `LocalAppData` (Windows).
   - Tracks background installations asynchronously (`IdeInstallTracker`) with a live spinner and elapsed timer, and automatically launches the IDE once installation completes.

4. **Automated Desktop Video Recording (`src/recorder.rs`)**
   - Automatically records the full desktop into timestamped `.mp4` files inside `recordings/` using FFmpeg (`gdigrab` on Windows, dynamically detected `avfoundation` screen index on macOS, `x11grab` on Linux).
   - Displays a live `⏺ REC HH:MM:SS` badge in the top bar, provides a one-click **Open Folder** button, and implements `Drop` cleanup so MP4 files finalize cleanly (`q\n` over `stdin`) even on unexpected exit.

---

## Project Structure

```text
Block-Website-for-contest/
├── Cargo.toml                        # Rust package & dependencies (eframe, egui, sha2, serde, chrono)
├── src/
│   ├── main.rs                       # Cross-platform Desktop GUI & session workflow
│   ├── theme.rs                      # 60-30-10 Color-Theory Dark & Light palettes and custom widgets
│   ├── state.rs                      # Per-session SHA-256 password hashing, timer & state persistence
│   ├── policy.rs                     # Windows Registry, macOS plist, and Linux JSON browser policy engine
│   ├── ide.rs                        # Cross-platform IDE detector, background installer tracker & launcher
│   └── recorder.rs                   # Background FFmpeg screen recorder, device detector & Drop cleanup
├── .github/workflows/build.yml       # GitHub Actions CI building Windows .exe, macOS, and Linux binaries
└── legacy_scripts/                   # Original standalone C++, PowerShell, macOS & Linux scripts
```

---

## Building & Running Locally

```bash
# Run in development mode
cargo run

# Build optimized release executable
cargo build --release
```
