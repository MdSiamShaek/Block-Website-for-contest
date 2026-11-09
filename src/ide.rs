use std::path::Path;
use std::process::Command;

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

    pub fn from_name(name: &str) -> Self {
        match name {
            "Code::Blocks" => IdeChoice::CodeBlocks,
            "CP Editor" => IdeChoice::CpEditor,
            "Sublime Text" => IdeChoice::SublimeText,
            "JetBrains CLion" => IdeChoice::CLion,
            _ => IdeChoice::VsCode,
        }
    }

    /// Checks whether the selected IDE is currently installed on the system.
    pub fn is_installed(&self) -> bool {
        if cfg!(target_os = "windows") {
            let bin = match self {
                IdeChoice::VsCode => "code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "subl",
                IdeChoice::CLion => "clion64",
            };
            Command::new("where")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        } else if cfg!(target_os = "macos") {
            let app_bundle = match self {
                IdeChoice::VsCode => "/Applications/Visual Studio Code.app",
                IdeChoice::CodeBlocks => "/Applications/CodeBlocks.app",
                IdeChoice::CpEditor => "/Applications/CP Editor.app",
                IdeChoice::SublimeText => "/Applications/Sublime Text.app",
                IdeChoice::CLion => "/Applications/CLion.app",
            };
            Path::new(app_bundle).exists()
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

    /// Launches the selected IDE if installed.
    pub fn launch(&self) -> Result<String, String> {
        if cfg!(target_os = "windows") {
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

    /// Installs the selected IDE silently in the background using the platform's native package manager.
    pub fn install(&self) -> Result<String, String> {
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
                .spawn()
                .map_err(|e| format!("Failed to start winget installer: {}", e))?;
            Ok(format!(
                "Installing {} via Windows Package Manager (winget)...",
                self.display_name()
            ))
        } else if cfg!(target_os = "macos") {
            let cask = match self {
                IdeChoice::VsCode => "visual-studio-code",
                IdeChoice::CodeBlocks => "codeblocks",
                IdeChoice::CpEditor => "cpeditor",
                IdeChoice::SublimeText => "sublime-text",
                IdeChoice::CLion => "clion",
            };
            Command::new("brew")
                .args(["install", "--cask", cask])
                .spawn()
                .map_err(|e| format!("Failed to start Homebrew cask installer: {}", e))?;
            Ok(format!(
                "Installing {} via Homebrew (brew install --cask {})...",
                self.display_name(),
                cask
            ))
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
                .spawn()
                .map_err(|e| format!("Failed to start Linux package installer: {}", e))?;
            Ok(format!(
                "Installing {} via system package manager...",
                self.display_name()
            ))
        }
    }
}
