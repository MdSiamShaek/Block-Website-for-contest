use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

/// Manages background desktop video recording via FFmpeg across Windows, macOS, and Linux.
pub struct ScreenRecorder {
    child: Option<Child>,
    started_at: Option<Instant>,
    output_file: Option<PathBuf>,
}

impl Default for ScreenRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenRecorder {
    pub fn new() -> Self {
        Self {
            child: None,
            started_at: None,
            output_file: None,
        }
    }

    /// Checks if `ffmpeg` is available on the system PATH or in common install locations.
    pub fn find_ffmpeg() -> Option<String> {
        let candidates = if cfg!(target_os = "windows") {
            vec!["ffmpeg.exe", r"C:\ffmpeg\bin\ffmpeg.exe"]
        } else if cfg!(target_os = "macos") {
            vec![
                "ffmpeg",
                "/opt/homebrew/bin/ffmpeg",
                "/usr/local/bin/ffmpeg",
            ]
        } else {
            vec!["ffmpeg", "/usr/bin/ffmpeg", "/usr/local/bin/ffmpeg"]
        };

        for bin in candidates {
            if let Ok(output) = Command::new(bin).arg("-version").output() {
                if output.status.success() {
                    return Some(bin.to_string());
                }
            }
        }
        None
    }

    pub fn is_ffmpeg_available() -> bool {
        Self::find_ffmpeg().is_some()
    }

    /// Installs FFmpeg automatically using the OS package manager (`winget`, `brew`, or `apt`).
    pub fn install_ffmpeg() -> Result<String, String> {
        if cfg!(target_os = "windows") {
            Command::new("winget")
                .args([
                    "install",
                    "--id",
                    "Gyan.FFmpeg",
                    "-e",
                    "--silent",
                    "--accept-package-agreements",
                    "--accept-source-agreements",
                ])
                .spawn()
                .map_err(|e| format!("Failed to launch winget for FFmpeg: {}", e))?;
            Ok("Installing FFmpeg via winget (Gyan.FFmpeg)...".to_string())
        } else if cfg!(target_os = "macos") {
            let brew = if Path::new("/opt/homebrew/bin/brew").exists() {
                "/opt/homebrew/bin/brew"
            } else {
                "brew"
            };
            Command::new(brew)
                .args(["install", "ffmpeg"])
                .spawn()
                .map_err(|e| format!("Failed to launch Homebrew for FFmpeg: {}", e))?;
            Ok("Installing FFmpeg via Homebrew (brew install ffmpeg)...".to_string())
        } else {
            Command::new("sh")
                .args(["-c", "pkexec apt-get install -y ffmpeg || sudo apt-get install -y ffmpeg"])
                .spawn()
                .map_err(|e| format!("Failed to launch apt for FFmpeg: {}", e))?;
            Ok("Installing FFmpeg via system package manager...".to_string())
        }
    }

    /// Starts recording the entire desktop into the `recordings/` directory.
    pub fn start(&mut self, output_dir: &str) -> Result<String, String> {
        if self.is_recording() {
            return Ok(self
                .output_file
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default());
        }

        let ffmpeg_bin = Self::find_ffmpeg().ok_or_else(|| {
            "FFmpeg is not installed. Click 'Install FFmpeg' in the GUI or disable video recording.".to_string()
        })?;

        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Could not create recordings directory: {}", e))?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let file_path = Path::new(output_dir).join(format!("contest_session_{}.mp4", timestamp));
        let file_str = file_path.to_string_lossy().to_string();

        let mut cmd = Command::new(ffmpeg_bin);
        cmd.arg("-y"); // Overwrite if needed

        if cfg!(target_os = "windows") {
            cmd.args(["-f", "gdigrab", "-framerate", "15", "-i", "desktop"]);
        } else if cfg!(target_os = "macos") {
            cmd.args([
                "-f",
                "avfoundation",
                "-capture_cursor",
                "1",
                "-framerate",
                "15",
                "-i",
                "1:none",
            ]);
        } else {
            let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0.0".to_string());
            cmd.args(["-f", "x11grab", "-framerate", "15", "-i", &display]);
        }

        cmd.args([
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "28",
            "-pix_fmt",
            "yuv420p",
            &file_str,
        ]);

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start FFmpeg screen recording: {}", e))?;

        self.child = Some(child);
        self.started_at = Some(Instant::now());
        self.output_file = Some(file_path);

        Ok(file_str)
    }

    /// Gracefully stops the active FFmpeg recording by writing `q\n` to stdin so the MP4 trailer is finalized.
    pub fn stop(&mut self) -> Result<Option<String>, String> {
        if let Some(mut child) = self.child.take() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"q\n");
                let _ = stdin.flush();
            }
            let _ = child.wait();
            self.started_at = None;
            let saved = self.output_file.take().map(|p| p.display().to_string());
            Ok(saved)
        } else {
            Ok(None)
        }
    }

    /// Returns true if the recording child process is currently running.
    pub fn is_recording(&mut self) -> bool {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    self.child = None;
                    self.started_at = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Formats the elapsed recording duration as `HH:MM:SS`.
    pub fn elapsed_formatted(&self) -> String {
        if let Some(start) = self.started_at {
            let secs = start.elapsed().as_secs();
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let rem_secs = secs % 60;
            format!("{:02}:{:02}:{:02}", hours, mins, rem_secs)
        } else {
            "00:00:00".to_string()
        }
    }

    pub fn current_file(&self) -> Option<String> {
        self.output_file.as_ref().map(|p| p.display().to_string())
    }
}
