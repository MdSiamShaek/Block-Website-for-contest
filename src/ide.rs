use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdeChoice {
    VsCode,
    CodeBlocks,
    CpEditor,
    SublimeText,
    CLion,
}

impl IdeChoice {
    pub const ALL: [IdeChoice; 5] = [
        IdeChoice::VsCode,
        IdeChoice::CodeBlocks,
        IdeChoice::CpEditor,
        IdeChoice::SublimeText,
        IdeChoice::CLion,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            IdeChoice::VsCode => "Visual Studio Code",
            IdeChoice::CodeBlocks => "Code::Blocks",
            IdeChoice::CpEditor => "CP Editor",
            IdeChoice::SublimeText => "Sublime Text",
            IdeChoice::CLion => "JetBrains CLion",
        }
    }

    pub fn short_description(&self) -> &'static str {
        match self {
            IdeChoice::VsCode => "Lightweight editor with C++/Python/Java extensions",
            IdeChoice::CodeBlocks => "Classic C/C++ IDE with bundled MinGW compiler support",
            IdeChoice::CpEditor => "Dedicated competitive programming IDE with test-case runner",
            IdeChoice::SublimeText => "Ultra-fast text editor with C++ build systems",
            IdeChoice::CLion => "Full-featured JetBrains C/C++ & Rust IDE",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "Code::Blocks" => IdeChoice::CodeBlocks,
            "CP Editor" => IdeChoice::CpEditor,
            "Sublime Text" => IdeChoice::SublimeText,
            "JetBrains CLion" => IdeChoice::CLion,
            _ => IdeChoice::VsCode,
        }
    }

    /// Checks whether the selected IDE is currently installed on the system,
    /// searching both system PATH and standard OS application directories.
    pub fn is_installed(&self) -> bool {
        if cfg!(target_os = "windows") {
            let bin = match self {
                IdeChoice::VsCode => "code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "subl",
                IdeChoice::CLion => "clion64",
            };
            if Command::new("where")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return true;
            }
            self.windows_known_paths().iter().any(|p| p.exists())
        } else if cfg!(target_os = "macos") {
            let app_bundle = match self {
                IdeChoice::VsCode => "Visual Studio Code.app",
                IdeChoice::CodeBlocks => "CodeBlocks.app",
                IdeChoice::CpEditor => "CP Editor.app",
                IdeChoice::SublimeText => "Sublime Text.app",
                IdeChoice::CLion => "CLion.app",
            };
            let sys_path = Path::new("/Applications").join(app_bundle);
            if sys_path.exists() {
                return true;
            }
            if let Ok(home) = std::env::var("HOME") {
                if Path::new(&home)
                    .join("Applications")
                    .join(app_bundle)
                    .exists()
                {
                    return true;
                }
            }
            false
        } else {
            let bin = match self {
                IdeChoice::VsCode => "code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "subl",
                IdeChoice::CLion => "clion",
            };
            Command::new("which")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    fn windows_known_paths(&self) -> Vec<PathBuf> {
        let local_app_data =
            std::env::var("LOCALAPPDATA").unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
        let program_files =
            std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
        let program_files_x86 =
            std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".to_string());

        match self {
            IdeChoice::VsCode => vec![
                Path::new(&local_app_data).join(r"Programs\Microsoft VS Code\Code.exe"),
                Path::new(&program_files).join(r"Microsoft VS Code\Code.exe"),
            ],
            IdeChoice::CodeBlocks => vec![
                Path::new(&program_files).join(r"CodeBlocks\codeblocks.exe"),
                Path::new(&program_files_x86).join(r"CodeBlocks\codeblocks.exe"),
            ],
            IdeChoice::CpEditor => vec![
                Path::new(&local_app_data).join(r"Programs\CP Editor\cpeditor.exe"),
                Path::new(&program_files).join(r"CP Editor\cpeditor.exe"),
            ],
            IdeChoice::SublimeText => vec![
                Path::new(&program_files).join(r"Sublime Text\sublime_text.exe"),
                Path::new(&program_files).join(r"Sublime Text 3\sublime_text.exe"),
            ],
            IdeChoice::CLion => vec![
                Path::new(&program_files).join(r"JetBrains\CLion\bin\clion64.exe"),
            ],
        }
    }

    /// Launches the selected IDE if installed.
    pub fn launch(&self) -> Result<String, String> {
        if cfg!(target_os = "windows") {
            if let Some(exe_path) = self.windows_known_paths().into_iter().find(|p| p.exists()) {
                Command::new(exe_path)
                    .spawn()
                    .map_err(|e| format!("Failed to launch {}: {}", self.display_name(), e))?;
                return Ok(format!("Launched {}", self.display_name()));
            }
            let cmd = match self {
                IdeChoice::VsCode => "code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "subl",
                IdeChoice::CLion => "clion64",
            };
            Command::new("cmd")
                .args(["/C", "start", "", cmd])
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", self.display_name(), e))?;
        } else if cfg!(target_os = "macos") {
            let app_name = match self {
                IdeChoice::VsCode => "Visual Studio Code",
                IdeChoice::CodeBlocks => "CodeBlocks",
                IdeChoice::CpEditor => "CP Editor",
                IdeChoice::SublimeText => "Sublime Text",
                IdeChoice::CLion => "CLion",
            };
            Command::new("open")
                .args(["-a", app_name])
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", self.display_name(), e))?;
        } else {
            let cmd = match self {
                IdeChoice::VsCode => "code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "subl",
                IdeChoice::CLion => "clion",
            };
            Command::new(cmd)
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", self.display_name(), e))?;
        }

        Ok(format!("Launched {}", self.display_name()))
    }

    /// Spawns a tracked background installation child process for the selected IDE.
    pub fn spawn_installer(&self) -> Result<Child, String> {
        if cfg!(target_os = "windows") {
            let winget_id = match self {
                IdeChoice::VsCode => "Microsoft.VisualStudioCode",
                IdeChoice::CodeBlocks => "CodeBlocks.CodeBlocks",
                IdeChoice::CpEditor => "cpeditor.cpeditor",
                IdeChoice::SublimeText => "SublimeHQ.SublimeText.4",
                IdeChoice::CLion => "JetBrains.CLion",
            };
            Command::new("winget")
                .args([
                    "install",
                    "--id",
                    winget_id,
                    "-e",
                    "--silent",
                    "--accept-package-agreements",
                    "--accept-source-agreements",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to start winget installer: {}", e))
        } else if cfg!(target_os = "macos") {
            let cask = match self {
                IdeChoice::VsCode => "visual-studio-code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "sublime-text",
                IdeChoice::CLion => "clion",
            };
            let brew = if Path::new("/opt/homebrew/bin/brew").exists() {
                "/opt/homebrew/bin/brew"
            } else if Path::new("/usr/local/bin/brew").exists() {
                "/usr/local/bin/brew"
            } else {
                "brew"
            };
            Command::new(brew)
                .args(["install", "--cask", cask])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to start Homebrew cask installer: {}", e))
        } else {
            let pkg = match self {
                IdeChoice::VsCode => "code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "sublime-text",
                IdeChoice::CLion => "clion",
            };
            Command::new("sh")
                .args([
                    "-c",
                    &format!(
                        "pkexec apt-get install -y {0} || sudo snap install --classic {0}",
                        pkg
                    ),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to start Linux package installer: {}", e))
        }
    }
}

/// Tracks an asynchronous background IDE installation so the GUI can display a live progress indicator
/// and auto-launch the IDE upon completion.
pub struct IdeInstallTracker {
    child: Option<Child>,
    target_ide: Option<IdeChoice>,
    started_at: Option<Instant>,
    pub auto_launch_when_done: bool,
}

impl Default for IdeInstallTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl IdeInstallTracker {
    pub fn new() -> Self {
        Self {
            child: None,
            target_ide: None,
            started_at: None,
            auto_launch_when_done: false,
        }
    }

    pub fn start_install(&mut self, ide: IdeChoice, auto_launch: bool) -> Result<String, String> {
        if self.is_installing() {
            return Err("An IDE installation is already in progress.".to_string());
        }
        let child = ide.spawn_installer()?;
        self.child = Some(child);
        self.target_ide = Some(ide);
        self.started_at = Some(Instant::now());
        self.auto_launch_when_done = auto_launch;
        Ok(format!(
            "Installing {} in the background...",
            ide.display_name()
        ))
    }

    pub fn is_installing(&mut self) -> bool {
        if let Some(child) = self.child.as_mut() {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.started_at.map(|s| s.elapsed().as_secs()).unwrap_or(0)
    }

    /// Polls if the background installer just finished and returns a status message if completed.
    pub fn poll_completion(&mut self) -> Option<(Result<String, String>, Option<IdeChoice>)> {
        let child = self.child.as_mut()?;
        match child.try_wait() {
            Ok(Some(status)) => {
                self.child = None;
                self.started_at = None;
                let ide = self.target_ide.take();
                let name = ide.map(|i| i.display_name()).unwrap_or("IDE");
                if status.success() {
                    let launch_target = if self.auto_launch_when_done { ide } else { None };
                    Some((
                        Ok(format!("Successfully installed {}!", name)),
                        launch_target,
                    ))
                } else {
                    Some((
                        Err(format!(
                            "Installer for {} exited with code {:?}.",
                            name,
                            status.code()
                        )),
                        None,
                    ))
                }
            }
            Ok(None) => None,
            Err(e) => {
                self.child = None;
                self.started_at = None;
                let _ = self.target_ide.take();
                Some((Err(format!("Installer error: {}", e)), None))
            }
        }
    }
}
