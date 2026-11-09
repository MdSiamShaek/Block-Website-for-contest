# Block-Website-for-contest

A multi-platform contest environment lockdown toolkit designed to restrict computer web browsing to a single allowed website (configured by default for [BAPS Online Judge](https://bapsoj.org)), prevent bypasses via portable/alternative browsers, and password-protect the unlock process.

---

## Features

- **Single-Domain Whitelisting:** Blocks all websites (`*`) while exclusively allowing `bapsoj.org` and its subdomains (`*.bapsoj.org`).
- **Contest File Downloads:** Allows safe contest files (`.txt`, `.zip`, `.cpp`, `.py`) while blocking dangerous executables (`.exe`, `.msi`) using browser `DownloadRestrictions = 1`.
- **Alternative Browser Execution Block:** Restricts execution of non-whitelisted and portable browsers (`opera.exe`, `tor.exe`, `vivaldi.exe`, etc.) via Windows Explorer `DisallowRun`.
- **SHA-256 Password Protection:** Unlocking requires an administrator password verified against an in-memory SHA-256 hash (default password: `MySecretPass123!`).
- **Automatic Administrator Elevation:** Windows C++ binary triggers native UAC prompts on launch automatically.

---

## Included Files

| File | Platform | Description |
| :--- | :--- | :--- |
| `bapsoj_lockdown.cpp` | **Windows** | Native C++ executable source with CNG SHA-256 hashing, UAC self-elevation, and browser policy registry management. |
| `BAPSOJ_Lockdown.ps1` | **Windows** | PowerShell implementation with interactive menu and masked password input. Compilable to `.exe` via `ps2exe`. |
| `mac_lockdown.sh` | **macOS** | Shell script managing `/Library/Preferences` managed plists for Chrome, Brave, and Edge. |
| `linux_lockdown.sh` | **Linux** | Shell script managing `/etc/opt/chrome/policies/managed/whitelist.json`. |

---

## How to Build & Run

### 1. Windows (C++ Executable)

Compile with MinGW `g++` (or Visual Studio MSVC):

```bash
g++ -O2 -s bapsoj_lockdown.cpp -o BAPSOJ_Lockdown.exe -lbcrypt -lshell32 -ladvapi32
```

Double-click `BAPSOJ_Lockdown.exe` to run. Select:
- **`1`** to enable contest lockdown.
- **`2`** to unlock (prompts for the administrator password).

---

### 2. Windows (PowerShell / PS2EXE)

To compile the PowerShell script into an `.exe`:

```powershell
Install-Module -Name ps2exe -Scope CurrentUser -Force
Invoke-ps2exe -InputFile ".\BAPSOJ_Lockdown.ps1" -OutputFile ".\BAPSOJ_Lockdown.exe" -requireAdmin
```

---

### 3. macOS

Run with root privileges:

```bash
sudo bash mac_lockdown.sh
```

Select `1` to apply the policy to Chrome, Brave, and Edge. Select `2` to restore normal access.

> **Note for macOS:** Apple Safari does not read enterprise policy plists. To lock down Safari, use **System Settings > Screen Time > Content Restrictions > Allowed Websites Only**.

---

### 4. Linux

Run with root privileges:

```bash
sudo bash linux_lockdown.sh
```

---

## Changing the Target Domain or Password

1. **Target Domain:** Change `bapsoj.org` to your preferred contest domain in `bapsoj_lockdown.cpp`, `BAPSOJ_Lockdown.ps1`, `mac_lockdown.sh`, or `linux_lockdown.sh`.
2. **Password:** Generate a SHA-256 hash for your secret password in PowerShell:
   ```powershell
   $Pass = "YourCustomPassword!"
   [System.BitConverter]::ToString([System.Security.Cryptography.SHA256]::Create().ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Pass))).Replace("-","")
   ```
   Replace the `STORED_HASH` string in `bapsoj_lockdown.cpp` and `BAPSOJ_Lockdown.ps1`.
