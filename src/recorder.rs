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
    installer_child: Option<Child>,
    pub capture_audio: bool,
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
            installer_child: None,
            capture_audio: false,
        }
    }

    /// Checks if `ffmpeg` is available on the system PATH or in common install locations.
    pub fn find_ffmpeg() -> Option<String> {
        let candidates = if cfg!(target_os = "windows") {
            vec![
                "ffmpeg.exe".to_string(),
                r"C:\ffmpeg\bin\ffmpeg.exe".to_string(),
                format!(
                    r"{}\Microsoft\WinGet\Links\ffmpeg.exe",
                    std::env::var("LOCALAPPDATA").unwrap_or_default()
                ),
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "ffmpeg".to_string(),
                "/opt/homebrew/bin/ffmpeg".to_string(),
                "/usr/local/bin/ffmpeg".to_string(),
            ]
        } else {
            vec![
                "ffmpeg".to_string(),
                "/usr/bin/ffmpeg".to_string(),
                "/usr/local/bin/ffmpeg".to_string(),
                "/snap/bin/ffmpeg".to_string(),
            ]
        };

        for bin in candidates {
            if bin.is_empty() {
                continue;
            }
            if let Ok(output) = Command::new(&bin).arg("-version").output() {
                if output.status.success() {
                    return Some(bin);
                }
            }
        }
        None
    }

    pub fn is_ffmpeg_available() -> bool {
        Self::find_ffmpeg().is_some()
    }

    /// Installs FFmpeg automatically using the OS package manager (`winget`, `brew`, or `apt`) and tracks the process.
    pub fn start_ffmpeg_install(&mut self) -> Result<String, String> {
        if self.is_installing_ffmpeg() {
            return Err("FFmpeg installation is already in progress.".to_string());
        }

        let child = if cfg!(target_os = "windows") {
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
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to launch winget for FFmpeg: {}", e))?
        } else if cfg!(target_os = "macos") {
            let brew = if Path::new("/opt/homebrew/bin/brew").exists() {
                "/opt/homebrew/bin/brew"
            } else if Path::new("/usr/local/bin/brew").exists() {
                "/usr/local/bin/brew"
            } else {
                "brew"
            };
            Command::new(brew)
                .args(["install", "ffmpeg"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to launch Homebrew for FFmpeg: {}", e))?
        } else {
            Command::new("sh")
                .args([
                    "-c",
                    "pkexec apt-get install -y ffmpeg || sudo apt-get install -y ffmpeg",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to launch apt for FFmpeg: {}", e))?
        };

        self.installer_child = Some(child);
        Ok("Installing FFmpeg in the background...".to_string())
    }

    pub fn is_installing_ffmpeg(&mut self) -> bool {
        if let Some(child) = self.installer_child.as_mut() {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    }

    pub fn poll_ffmpeg_install(&mut self) -> Option<Result<String, String>> {
        let child = self.installer_child.as_mut()?;
        match child.try_wait() {
            Ok(Some(status)) => {
                self.installer_child = None;
                if status.success() || Self::is_ffmpeg_available() {
                    Some(Ok("FFmpeg installed and ready for screen recording!".to_string()))
                } else {
                    Some(Err("FFmpeg installer finished with an error.".to_string()))
                }
            }
            Ok(None) => None,
            Err(e) => {
                self.installer_child = None;
                Some(Err(format!("FFmpeg installer error: {}", e)))
            }
        }
    }

    /// Dynamically detects the macOS AVFoundation screen device index (e.g. "Capture screen 0").
    fn detect_macos_screen_input(ffmpeg_bin: &str, capture_audio: bool) -> String {
        let audio_part = if capture_audio { "0" } else { "none" };
        if let Ok(output) = Command::new(ffmpeg_bin)
            .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()
        {
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines() {
                if line.contains("Capture screen") {
                    if let Some(start) = line.rfind('[') {
                        if let Some(end) = line[start..].find(']') {
                            let idx = line[start + 1..start + end].trim();
                            if idx.chars().all(|c| c.is_ascii_digit()) && !idx.is_empty() {
                                return format!("{}:{}", idx, audio_part);
                            }
                        }
                    }
                }
            }
        }
        format!("1:{}", audio_part)
    }

    /// Starts recording the entire desktop into the `output_dir` directory.
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

        let mut cmd = Command::new(&ffmpeg_bin);
        cmd.arg("-y");

        if cfg!(target_os = "windows") {
            cmd.args(["-f", "gdigrab", "-framerate", "15", "-i", "desktop"]);
        } else if cfg!(target_os = "macos") {
            let input_spec = Self::detect_macos_screen_input(&ffmpeg_bin, self.capture_audio);
            cmd.args([
                "-f",
                "avfoundation",
                "-capture_cursor",
                "1",
                "-framerate",
                "15",
                "-i",
                &input_spec,
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

    /// Opens the recordings folder in the system file explorer (Finder / Explorer / Files).
    pub fn open_recordings_folder(output_dir: &str) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
        if cfg!(target_os = "windows") {
            Command::new("explorer")
                .arg(output_dir)
                .spawn()
                .map_err(|e| e.to_string())?;
        } else if cfg!(target_os = "macos") {
            Command::new("open")
                .arg(output_dir)
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("xdg-open")
                .arg(output_dir)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
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

impl Drop for ScreenRecorder {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
