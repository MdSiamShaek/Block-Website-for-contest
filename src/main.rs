#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ide;
mod policy;
mod recorder;
mod state;

use eframe::egui;
use ide::IdeChoice;
use policy::{disable_browser_lockdown, enable_browser_lockdown, open_allowed_website};
use recorder::ScreenRecorder;
use state::{normalize_domain, verify_password, LockdownSession};
use std::time::Duration;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([680.0, 650.0])
            .with_min_inner_size([580.0, 560.0])
            .with_title("Contest Lockdown & Proctoring Suite"),
        ..Default::default()
    };

    eframe::run_native(
        "Contest Lockdown & Proctoring Suite",
        options,
        Box::new(|_cc| Ok(Box::new(ContestLockdownApp::new()))),
    )
}

struct ContestLockdownApp {
    session: LockdownSession,
    domain_input: String,
    selected_ide: IdeChoice,
    auto_install_ide: bool,
    auto_launch_ide: bool,
    auto_start_recording: bool,
    block_all_downloads: bool,
    recordings_dir: String,
    lock_password_input: String,
    lock_password_confirm: String,
    unlock_password_input: String,
    recorder: ScreenRecorder,
    status_log: Vec<String>,
    last_message_error: bool,
}

impl ContestLockdownApp {
    fn new() -> Self {
        let session = LockdownSession::load();
        let domain_input = session.allowed_domain.clone();
        let selected_ide = IdeChoice::from_name(&session.selected_ide);

        let mut log = vec![
            "Ready. Enter your target website, choose an IDE, and set a session password to start."
                .to_string(),
        ];
        if session.is_locked {
            log.push(format!(
                "Restored active lockdown session for https://{} (started at {}). Enter the session password to unlock.",
                session.allowed_domain, session.started_at
            ));
        }

        Self {
            session,
            domain_input,
            selected_ide,
            auto_install_ide: false,
            auto_launch_ide: true,
            auto_start_recording: true,
            block_all_downloads: false,
            recordings_dir: "recordings".to_string(),
            lock_password_input: String::new(),
            lock_password_confirm: String::new(),
            unlock_password_input: String::new(),
            recorder: ScreenRecorder::new(),
            status_log: log,
            last_message_error: false,
        }
    }

    fn push_log(&mut self, msg: impl Into<String>, is_error: bool) {
        self.last_message_error = is_error;
        self.status_log.push(msg.into());
        if self.status_log.len() > 8 {
            self.status_log.remove(0);
        }
    }

    fn handle_start_lockdown(&mut self) {
        let clean_domain = normalize_domain(&self.domain_input);
        if clean_domain.is_empty() {
            self.push_log("Please enter a valid website domain (e.g. bapsoj.org).", true);
            return;
        }
        if self.lock_password_input.is_empty() {
            self.push_log(
                "You must set a password for this lockdown session before locking!",
                true,
            );
            return;
        }
        if self.lock_password_input != self.lock_password_confirm {
            self.push_log("Password and Confirm Password do not match!", true);
            return;
        }

        // 1. Apply Browser Policy Lockdown
        match enable_browser_lockdown(&clean_domain, self.block_all_downloads) {
            Ok(msg) => self.push_log(msg, false),
            Err(err) => {
                self.push_log(
                    format!(
                        "Could not apply OS browser policy ({}). Make sure you run as Administrator / approve privilege prompt.",
                        err
                    ),
                    true,
                );
                return;
            }
        }

        // 2. Install or Launch Chosen IDE
        if self.auto_install_ide && !self.selected_ide.is_installed() {
            match self.selected_ide.install() {
                Ok(msg) => self.push_log(msg, false),
                Err(err) => self.push_log(err, true),
            }
        }
        if self.auto_launch_ide && self.selected_ide.is_installed() {
            match self.selected_ide.launch() {
                Ok(msg) => self.push_log(msg, false),
                Err(err) => self.push_log(err, true),
            }
        }

        // 3. Start Video Recording Automatically
        let mut recorded_path = None;
        if self.auto_start_recording {
            match self.recorder.start(&self.recordings_dir) {
                Ok(path) => {
                    self.push_log(format!("Video recording started: {}", path), false);
                    recorded_path = Some(path);
                }
                Err(err) => {
                    self.push_log(format!("Recording warning: {}", err), true);
                }
            }
        }

        // 4. Save Session Lock State with SHA-256 hash of the entered password
        self.session = LockdownSession::new_locked(
            &clean_domain,
            &self.lock_password_input,
            self.selected_ide.display_name(),
            recorded_path,
        );
        if let Err(e) = self.session.save() {
            self.push_log(format!("Warning: could not save session file: {}", e), true);
        }

        // Clear plaintext passwords from memory immediately
        self.lock_password_input.clear();
        self.lock_password_confirm.clear();
        self.unlock_password_input.clear();
        self.push_log(
            format!(
                "LOCKDOWN ENABLED for https://{}. Use the password you just set to unlock.",
                clean_domain
            ),
            false,
        );
    }

    fn handle_unlock_session(&mut self) {
        if !verify_password(&self.unlock_password_input, &self.session.password_sha256) {
            self.push_log(
                "ACCESS DENIED: Incorrect password! This session remains locked.",
                true,
            );
            return;
        }

        // 1. Stop Video Recording if active
        if self.recorder.is_recording() {
            if let Ok(Some(file)) = self.recorder.stop() {
                self.push_log(format!("Video recording saved to: {}", file), false);
            }
        }

        // 2. Remove Browser Policies
        match disable_browser_lockdown() {
            Ok(msg) => self.push_log(msg, false),
            Err(err) => {
                self.push_log(
                    format!("Password verified, but failed to clear OS policies: {}", err),
                    true,
                );
                return;
            }
        }

        // 3. Clear Session State
        self.session.is_locked = false;
        let _ = LockdownSession::clear();
        self.unlock_password_input.clear();
        self.push_log(
            "Session unlocked! All websites and browser settings restored.",
            false,
        );
    }
}

impl eframe::App for ContestLockdownApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.recorder.is_recording() {
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Contest Lockdown & Proctoring Suite");
                ui.label("Cross-Platform Website Whitelister • IDE Installer • Auto Video Recorder");
            });
            ui.add_space(6.0);
            ui.separator();

            // Status Banner
            let is_rec = self.recorder.is_recording();
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.session.is_locked {
                        ui.colored_label(
                            egui::Color32::from_rgb(230, 60, 60),
                            format!(
                                "● LOCKDOWN ACTIVE — Allowed Site: https://{}",
                                self.session.allowed_domain
                            ),
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(60, 190, 95),
                            "● STATUS: UNLOCKED (Normal Internet Access)",
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_rec {
                            ui.colored_label(
                                egui::Color32::from_rgb(240, 80, 80),
                                format!("⏺ REC {}", self.recorder.elapsed_formatted()),
                            );
                        } else {
                            ui.label("⏹ Recorder Idle");
                        }
                    });
                });
            });

            ui.add_space(8.0);

            if !self.session.is_locked {
                // ============================================================
                // UNLOCKED VIEW: CONFIGURE & START LOCKDOWN
                // ============================================================
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.heading("1. Website Lockdown Configuration");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Allowed Website Domain:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.domain_input)
                                .hint_text("bapsoj.org")
                                .desired_width(280.0),
                        );
                    });
                    ui.checkbox(
                        &mut self.block_all_downloads,
                        "Block all file downloads completely (uncheck to allow safe .txt/.zip contest files)",
                    );
                });

                ui.add_space(8.0);

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.heading("2. IDE Selection & Setup");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Select Contest IDE:");
                        egui::ComboBox::from_id_salt("ide_selector")
                            .selected_text(self.selected_ide.display_name())
                            .show_ui(ui, |ui| {
                                for ide in IdeChoice::ALL {
                                    ui.selectable_value(
                                        &mut self.selected_ide,
                                        ide,
                                        ide.display_name(),
                                    );
                                }
                            });

                        let installed = self.selected_ide.is_installed();
                        if installed {
                            ui.colored_label(egui::Color32::from_rgb(60, 190, 95), "✔ Installed");
                            if ui.button("Launch IDE Now").clicked() {
                                match self.selected_ide.launch() {
                                    Ok(m) => self.push_log(m, false),
                                    Err(e) => self.push_log(e, true),
                                }
                            }
                        } else {
                            ui.colored_label(
                                egui::Color32::from_rgb(230, 160, 40),
                                "⚠ Not Detected",
                            );
                            if ui.button("Install Selected IDE Now").clicked() {
                                match self.selected_ide.install() {
                                    Ok(m) => self.push_log(m, false),
                                    Err(e) => self.push_log(e, true),
                                }
                            }
                        }
                    });

                    ui.checkbox(
                        &mut self.auto_install_ide,
                        "Automatically install selected IDE if missing when Lockdown starts",
                    );
                    ui.checkbox(
                        &mut self.auto_launch_ide,
                        "Automatically launch selected IDE when Lockdown starts",
                    );
                });

                ui.add_space(8.0);

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.heading("3. Automatic Video Recording");
                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut self.auto_start_recording,
                        "Start full-screen video recording automatically when Lockdown begins",
                    );
                    ui.horizontal(|ui| {
                        ui.label("Output Folder:");
                        ui.text_edit_singleline(&mut self.recordings_dir);
                        if ScreenRecorder::is_ffmpeg_available() {
                            ui.colored_label(egui::Color32::from_rgb(60, 190, 95), "✔ FFmpeg Ready");
                        } else {
                            ui.colored_label(
                                egui::Color32::from_rgb(230, 160, 40),
                                "⚠ FFmpeg Missing",
                            );
                            if ui.button("Install FFmpeg").clicked() {
                                match ScreenRecorder::install_ffmpeg() {
                                    Ok(m) => self.push_log(m, false),
                                    Err(e) => self.push_log(e, true),
                                }
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.heading("4. Set Session Password & Lock");
                    ui.label("Enter a password for this lockdown session. You will need this exact password to unlock.");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Session Password:        ");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.lock_password_input)
                                .password(true)
                                .desired_width(220.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Confirm Password:        ");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.lock_password_confirm)
                                .password(true)
                                .desired_width(220.0),
                        );
                    });
                    ui.add_space(8.0);

                    let start_btn = egui::Button::new(
                        egui::RichText::new("🔒 Set Password & Start Contest Lockdown")
                            .size(16.0)
                            .strong(),
                    )
                    .min_size(egui::vec2(340.0, 38.0));

                    if ui.add(start_btn).clicked() {
                        self.handle_start_lockdown();
                    }
                });
            } else {
                // ============================================================
                // LOCKED VIEW: SESSION INFO & PASSWORD UNLOCK
                // ============================================================
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.heading("Active Contest Session Details");
                    ui.add_space(4.0);
                    ui.label(format!(
                        "• Whitelisted Website: https://{} (all other websites are blocked)",
                        self.session.allowed_domain
                    ));
                    ui.label(format!("• Selected Contest IDE: {}", self.session.selected_ide));
                    ui.label(format!("• Session Started At: {}", self.session.started_at));
                    if let Some(path) = self.recorder.current_file().or(self.session.recording_path.clone()) {
                        ui.label(format!("• Screen Recording File: {}", path));
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("🌐 Open Allowed Website").clicked() {
                            let _ = open_allowed_website(&self.session.allowed_domain);
                        }
                        if ui.button("💻 Launch Contest IDE").clicked() {
                            match IdeChoice::from_name(&self.session.selected_ide).launch() {
                                Ok(m) => self.push_log(m, false),
                                Err(e) => self.push_log(e, true),
                            }
                        }
                    });
                });

                ui.add_space(12.0);

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.heading("Unlock Contest Lockdown");
                    ui.label("Enter the session password that was set when this lockdown started:");
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Unlock Password:");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.unlock_password_input)
                                .password(true)
                                .desired_width(240.0),
                        );
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.handle_unlock_session();
                        }
                    });
                    ui.add_space(8.0);

                    let unlock_btn = egui::Button::new(
                        egui::RichText::new("🔓 Verify Password, Stop Recording & Unlock")
                            .size(16.0)
                            .strong(),
                    )
                    .min_size(egui::vec2(340.0, 38.0));

                    if ui.add(unlock_btn).clicked() {
                        self.handle_unlock_session();
                    }
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.label("Activity Log:");
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    for (idx, line) in self.status_log.iter().enumerate() {
                        let is_last = idx + 1 == self.status_log.len();
                        let color = if is_last && self.last_message_error {
                            egui::Color32::from_rgb(230, 80, 80)
                        } else {
                            ui.visuals().text_color()
                        };
                        ui.colored_label(color, format!("> {}", line));
                    }
                });
        });
    }
}
