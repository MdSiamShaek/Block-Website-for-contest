#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ide;
mod policy;
mod recorder;
mod state;
mod theme;

use eframe::egui::{self, Color32, Margin, RichText, Rounding, Stroke};
use ide::{IdeChoice, IdeInstallTracker};
use policy::{disable_browser_lockdown, enable_browser_lockdown, open_allowed_website};
use recorder::ScreenRecorder;
use state::{
    evaluate_password_strength, normalize_domain, verify_password, LockdownSession,
};
use std::time::Duration;
use theme::AppPalette;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 760.0])
            .with_min_inner_size([640.0, 620.0])
            .with_title("Contest Lockdown & Proctoring Suite"),
        ..Default::default()
    };

    eframe::run_native(
        "Contest Lockdown & Proctoring Suite",
        options,
        Box::new(|cc| Ok(Box::new(ContestLockdownApp::new(cc)))),
    )
}

struct ContestLockdownApp {
    session: LockdownSession,
    palette: AppPalette,
    // Website Whitelist Inputs
    domain_input: String,
    extra_domains: Vec<String>,
    new_extra_domain_input: String,
    block_all_downloads: bool,
    duration_minutes: u32,
    // IDE Setup & Tracker
    selected_ide: IdeChoice,
    auto_install_ide: bool,
    auto_launch_ide: bool,
    ide_installer: IdeInstallTracker,
    // Video Recorder
    auto_start_recording: bool,
    recordings_dir: String,
    recorder: ScreenRecorder,
    // Session Passwords & Visibility Toggles
    lock_password_input: String,
    lock_password_confirm: String,
    show_lock_password: bool,
    unlock_password_input: String,
    show_unlock_password: bool,
    // Activity Log
    status_log: Vec<(String, bool)>,
}

impl ContestLockdownApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let session = LockdownSession::load();
        let palette = if session.dark_mode {
            AppPalette::dark()
        } else {
            AppPalette::light()
        };
        palette.apply(&cc.egui_ctx);

        let domain_input = session.allowed_domain.clone();
        let extra_domains = session.extra_domains.clone();
        let selected_ide = IdeChoice::from_name(&session.selected_ide);
        let block_all_downloads = session.block_all_downloads;
        let duration_minutes = session.duration_minutes;

        let mut status_log = vec![(
            "Ready. Configure your contest domain, IDE, and session password to start.".to_string(),
            false,
        )];

        if session.is_locked {
            status_log.push((
                format!(
                    "Restored active lockdown session for https://{} (started at {}). Enter the session password to unlock.",
                    session.allowed_domain, session.started_at
                ),
                false,
            ));
        }

        Self {
            session,
            palette,
            domain_input,
            extra_domains,
            new_extra_domain_input: String::new(),
            block_all_downloads,
            duration_minutes,
            selected_ide,
            auto_install_ide: false,
            auto_launch_ide: true,
            ide_installer: IdeInstallTracker::new(),
            auto_start_recording: true,
            recordings_dir: "recordings".to_string(),
            recorder: ScreenRecorder::new(),
            lock_password_input: String::new(),
            lock_password_confirm: String::new(),
            show_lock_password: false,
            unlock_password_input: String::new(),
            show_unlock_password: false,
            status_log,
        }
    }

    fn push_log(&mut self, msg: impl Into<String>, is_error: bool) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.status_log
            .push((format!("[{}] {}", timestamp, msg.into()), is_error));
        if self.status_log.len() > 12 {
            self.status_log.remove(0);
        }
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.session.dark_mode = !self.session.dark_mode;
        self.palette = if self.session.dark_mode {
            AppPalette::dark()
        } else {
            AppPalette::light()
        };
        self.palette.apply(ctx);
        if self.session.is_locked {
            let _ = self.session.save();
        }
    }

    fn add_extra_domain(&mut self, domain_raw: &str) {
        let clean = normalize_domain(domain_raw);
        if clean.is_empty() {
            return;
        }
        if clean == normalize_domain(&self.domain_input) || self.extra_domains.contains(&clean) {
            return;
        }
        self.extra_domains.push(clean);
    }

    fn handle_start_lockdown(&mut self) {
        let clean_primary = normalize_domain(&self.domain_input);
        if clean_primary.is_empty() {
            self.push_log(
                "Please enter a valid primary contest domain (e.g. bapsoj.org).",
                true,
            );
            return;
        }
        if self.lock_password_input.is_empty() {
            self.push_log(
                "Please set a Session Password before starting the lockdown.",
                true,
            );
            return;
        }
        if self.lock_password_input != self.lock_password_confirm {
            self.push_log("Session Password and Confirm Password do not match.", true);
            return;
        }

        let mut all_domains = vec![clean_primary.clone()];
        for d in &self.extra_domains {
            let c = normalize_domain(d);
            if !c.is_empty() && !all_domains.contains(&c) {
                all_domains.push(c);
            }
        }

        // 1. Apply OS Browser Policy Lockdown
        match enable_browser_lockdown(&all_domains, self.block_all_downloads) {
            Ok(msg) => self.push_log(msg, false),
            Err(err) => {
                self.push_log(
                    format!(
                        "Could not apply browser policy ({}). Ensure Administrator privileges are granted.",
                        err
                    ),
                    true,
                );
                return;
            }
        }

        // 2. Install or Launch Chosen IDE
        let ide_installed = self.selected_ide.is_installed();
        if self.auto_install_ide && !ide_installed {
            match self
                .ide_installer
                .start_install(self.selected_ide, self.auto_launch_ide)
            {
                Ok(msg) => self.push_log(msg, false),
                Err(err) => self.push_log(err, true),
            }
        } else if self.auto_launch_ide && ide_installed {
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
                    self.push_log(format!("Screen recording started: {}", path), false);
                    recorded_path = Some(path);
                }
                Err(err) => {
                    self.push_log(format!("Screen recorder note: {}", err), true);
                }
            }
        }

        // 4. Persist Session State with SHA-256 hash of the entered password
        self.session = LockdownSession::new_locked(
            &clean_primary,
            self.extra_domains.clone(),
            &self.lock_password_input,
            self.selected_ide.display_name(),
            self.duration_minutes,
            self.block_all_downloads,
            recorded_path,
            self.palette.is_dark,
        );
        if let Err(e) = self.session.save() {
            self.push_log(format!("Warning: could not write session_lock.json: {}", e), true);
        }

        self.lock_password_input.clear();
        self.lock_password_confirm.clear();
        self.unlock_password_input.clear();
        self.show_lock_password = false;
        self.show_unlock_password = false;

        self.push_log(
            format!(
                "LOCKDOWN ACTIVE for https://{}. Use your session password to unlock.",
                clean_primary
            ),
            false,
        );
    }

    fn perform_unlock(&mut self, reason_msg: &str) {
        // 1. Stop Video Recording if active
        if self.recorder.is_recording() {
            if let Ok(Some(file)) = self.recorder.stop() {
                self.push_log(format!("Screen recording finalized & saved: {}", file), false);
            }
        }

        // 2. Remove Browser Policies
        match disable_browser_lockdown() {
            Ok(msg) => self.push_log(msg, false),
            Err(err) => {
                self.push_log(
                    format!("Failed to clear OS browser policies: {}", err),
                    true,
                );
                return;
            }
        }

        // 3. Clear Session State
        self.session.is_locked = false;
        let _ = LockdownSession::clear();
        self.unlock_password_input.clear();
        self.show_unlock_password = false;
        self.push_log(reason_msg, false);
    }

    fn handle_unlock_session(&mut self) {
        if !verify_password(&self.unlock_password_input, &self.session.password_sha256) {
            self.push_log(
                "ACCESS DENIED: Incorrect session password! Lockdown remains active.",
                true,
            );
            return;
        }
        self.perform_unlock("Password verified! Session unlocked and normal browsing restored.");
    }
}

impl eframe::App for ContestLockdownApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let p = self.palette;

        // Prevent accidental window close while lockdown is active
        if ctx.input(|i| i.viewport().close_requested()) && self.session.is_locked {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.push_log(
                "Cannot close window while Contest Lockdown is active! Enter the session password to unlock first.",
                true,
            );
        }

        // Poll background IDE installation status
        if let Some((result, auto_launch)) = self.ide_installer.poll_completion() {
            match result {
                Ok(msg) => {
                    self.push_log(msg, false);
                    if let Some(ide) = auto_launch {
                        if let Ok(l_msg) = ide.launch() {
                            self.push_log(l_msg, false);
                        }
                    }
                }
                Err(err) => self.push_log(err, true),
            }
        }

        // Poll background FFmpeg installation status
        if let Some(result) = self.recorder.poll_ffmpeg_install() {
            match result {
                Ok(msg) => self.push_log(msg, false),
                Err(err) => self.push_log(err, true),
            }
        }

        // Check if optional session countdown timer expired
        if let Some(rem_secs) = self.session.remaining_timer_secs() {
            if rem_secs == 0 {
                self.perform_unlock(
                    "Contest timer completed! Automatically unlocked browser policies and saved recording.",
                );
            }
        }

        let is_rec = self.recorder.is_recording();
        let is_installing_ide = self.ide_installer.is_installing();
        let is_installing_ffmpeg = self.recorder.is_installing_ffmpeg();
        let has_timer = self.session.remaining_timer_secs().is_some();

        if is_rec || is_installing_ide || is_installing_ffmpeg || has_timer {
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        // ====================================================================
        // TOP HEADER BAR (60-30-10 Secondary Surface + Semantic Badges)
        // ====================================================================
        egui::TopBottomPanel::top("header_panel")
            .frame(
                egui::Frame::none()
                    .fill(p.bg_header)
                    .stroke(Stroke::new(1.0_f32, p.border_subtle))
                    .inner_margin(Margin::symmetric(20.0, 14.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("🛡 Contest Lockdown & Proctoring Suite")
                                .size(20.0)
                                .strong()
                                .color(p.text_primary),
                        );
                        ui.label(
                            RichText::new(
                                "Cross-Platform Browser Policy Enforcer • IDE Setup • Automated Screen Recorder",
                            )
                            .size(12.5)
                            .color(p.text_secondary),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let theme_label = if p.is_dark { "☀ Light" } else { "🌙 Dark" };
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(theme_label).size(12.0).color(p.text_primary),
                                )
                                .fill(p.bg_card_subtle),
                            )
                            .on_hover_text("Switch between Dark Slate and Daylight color themes")
                            .clicked()
                        {
                            self.toggle_theme(ctx);
                        }

                        if is_rec {
                            p.badge(
                                ui,
                                &format!("⏺ REC {}", self.recorder.elapsed_formatted()),
                                p.danger_fg,
                                p.danger_bg,
                            );
                        } else {
                            p.badge(ui, "⏹ Recorder Idle", p.text_secondary, p.bg_card_subtle);
                        }

                        if self.session.is_locked {
                            p.badge(ui, "🔒 LOCKDOWN ACTIVE", p.danger_fg, p.danger_bg);
                        } else {
                            p.badge(ui, "🔓 UNLOCKED", p.success_fg, p.success_bg);
                        }
                    });
                });
            });

        // ====================================================================
        // BOTTOM ACTIVITY LOG PANEL
        // ====================================================================
        egui::TopBottomPanel::bottom("footer_log_panel")
            .frame(
                egui::Frame::none()
                    .fill(p.bg_header)
                    .stroke(Stroke::new(1.0_f32, p.border_subtle))
                    .inner_margin(Margin::symmetric(20.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Activity & Audit Log")
                            .size(12.0)
                            .strong()
                            .color(p.text_secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("Clear Log")
                            .on_hover_text("Clear status history")
                            .clicked()
                        {
                            self.status_log.clear();
                            self.push_log("Log cleared.", false);
                        }
                    });
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(78.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for (line, is_err) in &self.status_log {
                            let color = if *is_err {
                                p.danger_fg
                            } else {
                                p.text_secondary
                            };
                            ui.label(RichText::new(line).size(12.0).color(color).monospace());
                        }
                    });
            });

        // ====================================================================
        // MAIN CENTRAL CONTENT (60% Base Canvas + 30% Elevated Cards)
        // ====================================================================
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(p.bg_canvas)
                    .inner_margin(Margin::symmetric(20.0, 14.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if !self.session.is_locked {
                        self.render_unlocked_configuration(ui, p, is_installing_ide, is_installing_ffmpeg);
                    } else {
                        self.render_locked_session(ui, p);
                    }
                });
            });
    }
}

impl ContestLockdownApp {
    fn render_unlocked_configuration(
        &mut self,
        ui: &mut egui::Ui,
        p: AppPalette,
        is_installing_ide: bool,
        is_installing_ffmpeg: bool,
    ) {
        // --------------------------------------------------------------------
        // CARD 1: WEBSITE WHITELIST & POLICY CONFIGURATION
        // --------------------------------------------------------------------
        p.card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("1. Website Whitelist & Browser Policy")
                        .size(15.5)
                        .strong()
                        .color(p.accent_cyan),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Chrome • Edge • Brave • Vivaldi • Firefox")
                            .size(11.5)
                            .color(p.text_muted),
                    );
                });
            });
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Primary Contest Domain:").color(p.text_primary));
                ui.add(
                    egui::TextEdit::singleline(&mut self.domain_input)
                        .hint_text("bapsoj.org")
                        .desired_width(240.0),
                );
                if ui
                    .button("🌐 Preview Site")
                    .on_hover_text("Open this website in your browser now")
                    .clicked()
                {
                    let clean = normalize_domain(&self.domain_input);
                    if !clean.is_empty() {
                        let _ = open_allowed_website(&clean);
                    }
                }
            });

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Extra Allowed Reference Domains:")
                        .size(12.5)
                        .color(p.text_secondary),
                );
                for preset in ["cppreference.com", "codeforces.com", "atcoder.jp", "vjudge.net"] {
                    let already_added = self.extra_domains.iter().any(|d| d == preset);
                    if !already_added
                        && ui
                            .small_button(format!("+ {}", preset))
                            .on_hover_text("Whitelist this reference domain alongside the primary site")
                            .clicked()
                    {
                        self.add_extra_domain(preset);
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_extra_domain_input)
                        .hint_text("Add custom domain (e.g. docs.python.org)")
                        .desired_width(240.0),
                );
                if ui.button("+ Add Domain").clicked() && !self.new_extra_domain_input.is_empty() {
                    let to_add = self.new_extra_domain_input.clone();
                    self.add_extra_domain(&to_add);
                    self.new_extra_domain_input.clear();
                }
            });

            if !self.extra_domains.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    let mut remove_idx = None;
                    for (idx, dom) in self.extra_domains.iter().enumerate() {
                        egui::Frame::none()
                            .fill(p.bg_card_subtle)
                            .stroke(Stroke::new(1.0_f32, p.accent_cyan.gamma_multiply(0.5)))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(Margin::symmetric(8.0, 3.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(dom).size(12.0).color(p.accent_cyan));
                                    if ui.small_button("✕").clicked() {
                                        remove_idx = Some(idx);
                                    }
                                });
                            });
                    }
                    if let Some(i) = remove_idx {
                        self.extra_domains.remove(i);
                    }
                });
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.block_all_downloads,
                    "Block all downloads completely (uncheck to allow safe .txt/.zip problem statements)",
                );
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Auto-Unlock Contest Timer:").color(p.text_secondary));
                for (mins, label) in [
                    (0, "Manual Only"),
                    (60, "1 Hour"),
                    (120, "2 Hours"),
                    (180, "3 Hours"),
                    (300, "5 Hours"),
                ] {
                    ui.selectable_value(&mut self.duration_minutes, mins, label);
                }
            });
        });

        ui.add_space(10.0);

        // --------------------------------------------------------------------
        // CARD 2: IDE SELECTION, SILENT INSTALLER & LAUNCHER
        // --------------------------------------------------------------------
        p.card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("2. Contest IDE Installer & Launcher")
                        .size(15.5)
                        .strong()
                        .color(p.accent_cyan),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let installed = self.selected_ide.is_installed();
                    if is_installing_ide {
                        p.badge(
                            ui,
                            &format!("⏳ Installing ({}s)...", self.ide_installer.elapsed_secs()),
                            p.warning_fg,
                            p.warning_bg,
                        );
                    } else if installed {
                        p.badge(ui, "✔ Installed & Ready", p.success_fg, p.success_bg);
                    } else {
                        p.badge(ui, "⚠ Not Installed", p.warning_fg, p.warning_bg);
                    }
                });
            });
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Chosen IDE:").color(p.text_primary));
                egui::ComboBox::from_id_salt("ide_selector")
                    .width(210.0)
                    .selected_text(self.selected_ide.display_name())
                    .show_ui(ui, |ui| {
                        for ide in IdeChoice::ALL {
                            ui.selectable_value(&mut self.selected_ide, ide, ide.display_name());
                        }
                    });

                let installed = self.selected_ide.is_installed();
                if installed {
                    if ui.button("🚀 Launch IDE").clicked() {
                        match self.selected_ide.launch() {
                            Ok(m) => self.push_log(m, false),
                            Err(e) => self.push_log(e, true),
                        }
                    }
                } else if !is_installing_ide {
                    if ui.button("⬇ Install Selected IDE Now").clicked() {
                        match self.ide_installer.start_install(self.selected_ide, false) {
                            Ok(m) => self.push_log(m, false),
                            Err(e) => self.push_log(e, true),
                        }
                    }
                } else {
                    ui.spinner();
                }
            });

            ui.label(
                RichText::new(self.selected_ide.short_description())
                    .size(12.0)
                    .color(p.text_muted),
            );
            ui.add_space(4.0);
            ui.checkbox(
                &mut self.auto_install_ide,
                "Automatically install this IDE in the background if missing when Lockdown starts",
            );
            ui.checkbox(
                &mut self.auto_launch_ide,
                "Automatically launch this IDE when Lockdown starts",
            );
        });

        ui.add_space(10.0);

        // --------------------------------------------------------------------
        // CARD 3: AUTOMATIC SCREEN RECORDING ENGINE
        // --------------------------------------------------------------------
        p.card_frame().show(ui, |ui| {
            let ffmpeg_ok = ScreenRecorder::is_ffmpeg_available();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("3. Automated Desktop Video Recording")
                        .size(15.5)
                        .strong()
                        .color(p.accent_cyan),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if is_installing_ffmpeg {
                        p.badge(ui, "⏳ Installing FFmpeg...", p.warning_fg, p.warning_bg);
                    } else if ffmpeg_ok {
                        p.badge(ui, "✔ FFmpeg Engine Ready", p.success_fg, p.success_bg);
                    } else {
                        p.badge(ui, "⚠ FFmpeg Required", p.warning_fg, p.warning_bg);
                    }
                });
            });
            ui.add_space(6.0);

            ui.checkbox(
                &mut self.auto_start_recording,
                "Start full-screen MP4 desktop recording automatically when Lockdown begins",
            );

            ui.horizontal(|ui| {
                ui.label(RichText::new("Recordings Directory:").color(p.text_primary));
                ui.add(
                    egui::TextEdit::singleline(&mut self.recordings_dir).desired_width(200.0),
                );
                if ui.button("📂 Open Folder").clicked() {
                    if let Err(e) = ScreenRecorder::open_recordings_folder(&self.recordings_dir) {
                        self.push_log(format!("Could not open folder: {}", e), true);
                    }
                }
                if !ffmpeg_ok && !is_installing_ffmpeg && ui.button("⬇ Install FFmpeg").clicked()
                {
                    match self.recorder.start_ffmpeg_install() {
                        Ok(m) => self.push_log(m, false),
                        Err(e) => self.push_log(e, true),
                    }
                }
            });
        });

        ui.add_space(10.0);

        // --------------------------------------------------------------------
        // CARD 4: PER-SESSION PASSWORD SETUP & LOCK TRIGGER (10% Accent CTA)
        // --------------------------------------------------------------------
        p.card_frame().show(ui, |ui| {
            ui.label(
                RichText::new("4. Set Session Password & Activate Lockdown")
                    .size(15.5)
                    .strong()
                    .color(p.accent_cyan),
            );
            ui.label(
                RichText::new(
                    "Set a password for this lockdown session. It is hashed with SHA-256 and required to unlock.",
                )
                .size(12.5)
                .color(p.text_secondary),
            );
            ui.add_space(6.0);

            let (strength_score, strength_label) =
                evaluate_password_strength(&self.lock_password_input);
            let strength_color = match strength_score {
                0 => p.text_muted,
                1 => p.danger_fg,
                2 => p.warning_fg,
                3 => p.accent_cyan,
                _ => p.success_fg,
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new("New Session Password:").color(p.text_primary));
                ui.add(
                    egui::TextEdit::singleline(&mut self.lock_password_input)
                        .password(!self.show_lock_password)
                        .hint_text("Enter password to lock")
                        .desired_width(210.0),
                );
                if ui
                    .button(if self.show_lock_password { "🙈 Hide" } else { "👁 Show" })
                    .clicked()
                {
                    self.show_lock_password = !self.show_lock_password;
                }
                if !self.lock_password_input.is_empty() {
                    ui.label(
                        RichText::new(format!("Strength: {}", strength_label))
                            .size(12.0)
                            .strong()
                            .color(strength_color),
                    );
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Confirm Password:       ").color(p.text_primary));
                ui.add(
                    egui::TextEdit::singleline(&mut self.lock_password_confirm)
                        .password(!self.show_lock_password)
                        .hint_text("Re-enter password")
                        .desired_width(210.0),
                );
                if !self.lock_password_confirm.is_empty() {
                    if self.lock_password_input == self.lock_password_confirm {
                        ui.label(
                            RichText::new("✔ Matches")
                                .size(12.0)
                                .strong()
                                .color(p.success_fg),
                        );
                    } else {
                        ui.label(
                            RichText::new("✕ Does not match")
                                .size(12.0)
                                .strong()
                                .color(p.danger_fg),
                        );
                    }
                }
            });

            ui.add_space(10.0);

            ui.vertical_centered(|ui| {
                let cta_button = egui::Button::new(
                    RichText::new("🔒 Set Session Password & Start Contest Lockdown")
                        .size(15.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(p.accent_primary)
                .stroke(Stroke::new(1.2_f32, p.accent_cyan))
                .rounding(Rounding::same(8.0))
                .min_size(egui::vec2(420.0, 42.0));

                if ui.add(cta_button).clicked() {
                    self.handle_start_lockdown();
                }
            });
        });
    }

    fn render_locked_session(&mut self, ui: &mut egui::Ui, p: AppPalette) {
        // --------------------------------------------------------------------
        // SEMANTIC CRIMSON LOCKDOWN BANNER
        // --------------------------------------------------------------------
        p.banner_frame(p.danger_bg, p.danger_fg).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("🔒 CONTEST ENVIRONMENT LOCKED DOWN")
                            .size(17.0)
                            .strong()
                            .color(p.danger_fg),
                    );
                    ui.label(
                        RichText::new(format!(
                            "All websites are blocked except https://{} (and approved reference domains).",
                            self.session.allowed_domain
                        ))
                        .size(13.0)
                        .color(p.text_primary),
                    );
                });

                if let Some(rem_secs) = self.session.remaining_timer_secs() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let hrs = rem_secs / 3600;
                        let mins = (rem_secs % 3600) / 60;
                        let secs = rem_secs % 60;
                        p.badge(
                            ui,
                            &format!("⏱ Remaining: {:02}:{:02}:{:02}", hrs, mins, secs),
                            p.warning_fg,
                            p.warning_bg,
                        );
                    });
                }
            });
        });

        ui.add_space(12.0);

        // --------------------------------------------------------------------
        // ACTIVE SESSION DETAILS & QUICK LAUNCH ACTIONS
        // --------------------------------------------------------------------
        p.card_frame().show(ui, |ui| {
            ui.label(
                RichText::new("Active Session Parameters & Quick Actions")
                    .size(15.5)
                    .strong()
                    .color(p.accent_cyan),
            );
            ui.add_space(6.0);

            let domains_str = self
                .session
                .all_allowed_domains()
                .into_iter()
                .map(|d| format!("https://{}", d))
                .collect::<Vec<_>>()
                .join("  •  ");

            ui.label(RichText::new(format!("• Whitelisted Domains: {}", domains_str)).color(p.text_primary));
            ui.label(
                RichText::new(format!(
                    "• Contest IDE: {}",
                    self.session.selected_ide
                ))
                .color(p.text_primary),
            );
            ui.label(
                RichText::new(format!(
                    "• Lockdown Started: {}",
                    self.session.started_at
                ))
                .color(p.text_secondary),
            );

            if let Some(path) = self
                .recorder
                .current_file()
                .or_else(|| self.session.recording_path.clone())
            {
                ui.label(
                    RichText::new(format!("• Active Video Recording: {}", path))
                        .color(p.success_fg),
                );
            }

            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for dom in self.session.all_allowed_domains() {
                    if ui
                        .button(format!("🌐 Open {}", dom))
                        .clicked()
                    {
                        let _ = open_allowed_website(&dom);
                    }
                }

                if ui
                    .button(format!("💻 Launch {}", self.session.selected_ide))
                    .clicked()
                {
                    match IdeChoice::from_name(&self.session.selected_ide).launch() {
                        Ok(m) => self.push_log(m, false),
                        Err(e) => self.push_log(e, true),
                    }
                }

                if ui.button("📂 Recordings Folder").clicked() {
                    let _ = ScreenRecorder::open_recordings_folder(&self.recordings_dir);
                }
            });
        });

        ui.add_space(14.0);

        // --------------------------------------------------------------------
        // PASSWORD VERIFICATION & UNLOCK CARD
        // --------------------------------------------------------------------
        p.card_frame().show(ui, |ui| {
            ui.label(
                RichText::new("Unlock Contest Environment")
                    .size(15.5)
                    .strong()
                    .color(p.accent_cyan),
            );
            ui.label(
                RichText::new(
                    "Enter the session password that was set when this lockdown started to stop recording and restore all websites:",
                )
                .size(12.5)
                .color(p.text_secondary),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Session Password:").color(p.text_primary));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.unlock_password_input)
                        .password(!self.show_unlock_password)
                        .hint_text("Enter session password")
                        .desired_width(240.0),
                );
                if ui
                    .button(if self.show_unlock_password {
                        "🙈 Hide"
                    } else {
                        "👁 Show"
                    })
                    .clicked()
                {
                    self.show_unlock_password = !self.show_unlock_password;
                }
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.handle_unlock_session();
                }
            });

            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                let unlock_btn = egui::Button::new(
                    RichText::new("🔓 Verify Password, Stop Recording & Restore Normal Access")
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(p.accent_primary)
                .stroke(Stroke::new(1.2_f32, p.accent_cyan))
                .rounding(Rounding::same(8.0))
                .min_size(egui::vec2(440.0, 42.0));

                if ui.add(unlock_btn).clicked() {
                    self.handle_unlock_session();
                }
            });
        });
    }
}
