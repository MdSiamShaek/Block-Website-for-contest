# Block-Website-for-contest — Cross-Platform Rust Contest Lockdown & Proctoring Suite

A cross-platform **Rust Desktop GUI application** (`eframe` / `egui`) for programming contests and lab environments. It locks down web browsers to a single chosen domain via OS-level Browser Enterprise Policies, installs and launches a chosen IDE, starts full-screen desktop video recording automatically, and requires a **dynamic per-session password** (hashed with SHA-256) set at lock time to unlock.

---

## Core Features

1. **Dynamic Per-Session Password Protection (`src/state.rs`)**
   - Every time you enable a lockdown, the app asks you to set and confirm a **Session Password**.
   - The password is immediately hashed in memory using **SHA-256** and persisted to `session_lock.json` (plaintext is never stored).
   - Unlocking the browser policies and stopping the recording requires entering that exact password—even if the app window is closed and reopened during the contest.

2. **Cross-Platform Browser Policy Lockdown (`src/policy.rs`)**
   - **Windows:** Configures `URLBlocklist = ["*"]` and `URLAllowlist = ["<domain>", "*.<domain>"]` across Chrome, Edge, and Brave (`HKLM\SOFTWARE\Policies\...`), configures Mozilla Firefox `WebsiteFilter`, sets `DownloadRestrictions` (allowing safe `.txt`/`.zip` contest problem files while blocking `.exe`/`.msi` installers, or blocking all downloads), and blocks portable browsers via Windows Explorer `DisallowRun`.
   - **macOS:** Writes managed preference `.plist` rules under `/Library/Preferences` for Chrome, Brave, and Edge via native macOS Administrator elevation (`osascript`), and restarts running browsers.
   - **Linux:** Writes managed policy JSON files to `/etc/opt/chrome/policies/managed/whitelist.json`, `/etc/chromium/policies/managed/whitelist.json`, and `/etc/brave/policies/managed/whitelist.json`.
   - Automatically opens the whitelisted contest website in the browser when lockdown begins.

3. **Supported IDE Selector, Silent Installer & Launcher (`src/ide.rs`)**
   - Supports **Visual Studio Code**, **Code::Blocks**, **CP Editor**, **Sublime Text**, and **JetBrains CLion**.
   - Automatically detects if the chosen IDE is installed on Windows (`where`), macOS (`/Applications/*.app`), or Linux (`which`).
   - One-click or automatic silent installation via `winget` (Windows), `brew install --cask` (macOS), or `apt`/`snap` (Linux), plus automatic IDE launch on contest start.

4. **Automatic Background Screen Recording (`src/recorder.rs`)**
   - Automatically records the full desktop screen into timestamped `.mp4` files inside `recordings/` using FFmpeg (`gdigrab` on Windows, `avfoundation` on macOS, `x11grab` on Linux).
   - Displays a live `REC HH:MM:SS` timer in the GUI and gracefully finalizes the MP4 trailer (`q\n` over `stdin`) when the session is unlocked.
   - Includes a one-click **Install FFmpeg** button if FFmpeg is not yet installed on the machine.

---

## Project Structure

```text
Block-Website-for-contest/
├── Cargo.toml                        # Rust package & dependencies (eframe, egui, sha2, serde, chrono)
├── src/
│   ├── main.rs                       # Cross-platform Desktop GUI (eframe/egui)
│   ├── state.rs                      # Per-session SHA-256 password hashing & session_lock.json persistence
│   ├── policy.rs                     # Windows Registry, macOS plist, and Linux JSON browser policy engine
│   ├── ide.rs                        # Cross-platform IDE detection, silent installer & launcher
│   └── recorder.rs                   # Background FFmpeg screen recorder & live timer
├── .github/workflows/build.yml       # GitHub Actions CI building Windows .exe, macOS, and Linux binaries
└── legacy_scripts/                   # Original standalone C++, PowerShell, macOS & Linux scripts
```

---

## Building & Running Locally

### Prerequisites
- [Rust Toolchain (`rustup` / `cargo`)](https://rustup.rs/)
- Optional: [FFmpeg](https://ffmpeg.org/) (can also be installed directly from the app GUI)

### Compile and Run
```bash
# Run in development mode
cargo run

# Build optimized release executable
cargo build --release
```

- **Windows Release Binary:** `target\release\contest_lockdown.exe` (Run as Administrator so Registry policy edits succeed)
- **macOS Release Binary:** `target/release/contest_lockdown`
- **Linux Release Binary:** `target/release/contest_lockdown`

---

## Downloading Prebuilt Binaries (GitHub Actions)

Every push to `main` triggers the **Build Cross-Platform Binaries** GitHub Actions workflow (`.github/workflows/build.yml`), which automatically compiles:
- `contest_lockdown-windows-x86_64.exe` (Windows)
- `contest_lockdown-macos-arm64` (macOS)
- `contest_lockdown-linux-x86_64` (Linux)
