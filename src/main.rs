#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ide;
mod policy;
mod recorder;
mod state;
mod theme;

use eframe::egui::{self, Color32, FontId, Margin, RichText, Rounding, Stroke};
use ide::{IdeChoice, IdeInstallTracker};
use policy::{disable_browser_lockdown, enable_browser_lockdown, open_allowed_website};
use recorder::ScreenRecorder;
use state::{
    evaluate_password_strength, normalize_domain, verify_password, LockdownSession,
};
use std::time::Duration;
use theme::{build_window_icon, AppPalette};

/// The 4 sequential pages:
/// 1. Contest IDE Installer & Launcher
/// 2. Automated Desktop Video Recording
/// 3. Website Whitelist & Browser Policy
/// 4. Set Session Password & Activate Lockdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardPage {
    IdeSetup = 0,
    VideoRecording = 1,
    WebsitePolicy = 2,
    ActivateLockdown = 3,
}

impl WizardPage {
    pub const ALL: [WizardPage; 4] = [
        WizardPage::IdeSetup,
        WizardPage::VideoRecording,
        WizardPage::WebsitePolicy,
        WizardPage::ActivateLockdown,
    ];

    pub fn step_number(&self) -> usize {
        (*self as usize) + 1
    }

    pub fn title(&self) -> &'static str {
        match self {
            WizardPage::IdeSetup => "Contest IDE Installer & Launcher",
            WizardPage::VideoRecording => "Automated Desktop Video Recording",
            WizardPage::WebsitePolicy => "Website Whitelist & Browser Policy",
            WizardPage::ActivateLockdown => "Set Session Password & Activate Lockdown",
        }
    }

    pub fn short_tab_label(&self) -> &'static str {
        match self {
            WizardPage::IdeSetup => "1. IDE Installer",
            WizardPage::VideoRecording => "2. Video Recording",
            WizardPage::WebsitePolicy => "3. Website Whitelist",
            WizardPage::ActivateLockdown => "4. Activate Lockdown",
        }
    }

    pub fn next(&self) -> Option<WizardPage> {
        match self {
            WizardPage::IdeSetup => Some(WizardPage::VideoRecording),
            WizardPage::VideoRecording => Some(WizardPage::WebsitePolicy),
            WizardPage::WebsitePolicy => Some(WizardPage::ActivateLockdown),
            WizardPage::ActivateLockdown => None,
        }
    }

    pub fn prev(&self) -> Option<WizardPage> {
        match self {
            WizardPage::IdeSetup => None,
            WizardPage::VideoRecording => Some(WizardPage::IdeSetup),
            WizardPage::WebsitePolicy => Some(WizardPage::VideoRecording),
            WizardPage::ActivateLockdown => Some(WizardPage::WebsitePolicy),
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 720.0])
            .with_min_inner_size([740.0, 620.0])
            .with_icon(build_window_icon())
            .with_title("ProctorX — SECURE. MONITOR. CONTROL."),
        ..Default::default()
    };

    eframe::run_native(
        "ProctorX — SECURE. MONITOR. CONTROL.",
        options,
        Box::new(|cc| Ok(Box::new(ContestLockdownApp::new(cc)))),
    )
}

struct ContestLockdownApp {
    session: LockdownSession,
    palette: AppPalette,
    current_page: WizardPage,
    // Page 1: Multi-IDE Selection & Batch Installer Tracker
    selected_ides: Vec<IdeChoice>,
    auto_install_ide: bool,
    auto_launch_ide: bool,
    ide_installer: IdeInstallTracker,
    // Page 2: Video Recorder
    auto_start_recording: bool,
    recordings_dir: String,
    recorder: ScreenRecorder,
    // Page 3: Website Whitelist Inputs
    domain_input: String,
    extra_domains: Vec<String>,
    new_extra_domain_input: String,
    block_all_downloads: bool,
    duration_minutes: u32,
    // Page 4: Session Passwords & Visibility Toggles
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
        let mut selected_ides: Vec<IdeChoice> = session
            .all_selected_ides()
            .iter()
            .map(|n| IdeChoice::from_name(n))
            .collect();
        if selected_ides.is_empty() {
            selected_ides.push(IdeChoice::VsCode);
        }

        let block_all_downloads = session.block_all_downloads;
        let duration_minutes = session.duration_minutes;

        let mut status_log = vec![(
            "ProctorX Ready — Complete Steps 1–4 to lock down the contest environment."
                .to_string(),
            false,
        )];

        if session.is_locked {
            status_log.push((
                format!(
                    "Restored active ProctorX lockdown for https://{} ({}). Enter session password to unlock.",
                    session.allowed_domain, session.started_at
                ),
                false,
            ));
        }

        Self {
            session,
            palette,
            current_page: WizardPage::IdeSetup,
            selected_ides,
            auto_install_ide: false,
            auto_launch_ide: true,
            ide_installer: IdeInstallTracker::new(),
            auto_start_recording: true,
            recordings_dir: "recordings".to_string(),
            recorder: ScreenRecorder::new(),
            domain_input,
            extra_domains,
            new_extra_domain_input: String::new(),
            block_all_downloads,
            duration_minutes,
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

    fn toggle_extra_domain(&mut self, domain_raw: &str) {
        let clean = normalize_domain(domain_raw);
        if clean.is_empty() {
            return;
        }
        if let Some(pos) = self.extra_domains.iter().position(|d| d == &clean) {
            self.extra_domains.remove(pos);
        } else if clean != normalize_domain(&self.domain_input) {
            self.extra_domains.push(clean);
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
                "Please enter a valid primary contest domain on Page 3 (e.g. bapsoj.org).",
                true,
            );
            self.current_page = WizardPage::WebsitePolicy;
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

        // 2. Install or Launch All Selected IDEs
        let mut missing_ides = Vec::new();
        let selected_snapshot = self.selected_ides.clone();
        for ide in selected_snapshot {
            if ide.is_installed() {
                if self.auto_launch_ide {
                    match ide.launch() {
                        Ok(msg) => self.push_log(msg, false),
                        Err(err) => self.push_log(err, true),
                    }
                }
            } else if self.auto_install_ide {
                missing_ides.push(ide);
            }
        }
        if !missing_ides.is_empty() {
            match self
                .ide_installer
                .enqueue_installs(&missing_ides, self.auto_launch_ide)
            {
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
        let ide_names: Vec<String> = self
            .selected_ides
            .iter()
            .map(|i| i.display_name().to_string())
            .collect();

        self.session = LockdownSession::new_locked(
            &clean_primary,
            self.extra_domains.clone(),
            &self.lock_password_input,
            ide_names,
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
        if self.recorder.is_recording() {
            if let Ok(Some(file)) = self.recorder.stop() {
                self.push_log(format!("Screen recording finalized & saved: {}", file), false);
            }
        }

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

        self.session.is_locked = false;
        let _ = LockdownSession::clear();
        self.unlock_password_input.clear();
        self.show_unlock_password = false;
        self.current_page = WizardPage::IdeSetup;
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
                "Cannot close ProctorX while Contest Lockdown is active! Enter the session password to unlock first.",
                true,
            );
        }

        // Poll background IDE batch installation status
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
        // SINGLE-ROW COMPACT PROCTORX TOP HEADER BAR
        // ====================================================================
        egui::TopBottomPanel::top("header_panel")
            .frame(
                egui::Frame::none()
                    .fill(p.bg_header)
                    .stroke(Stroke::new(1.5_f32, p.gold_core))
                    .inner_margin(Margin::symmetric(24.0, 12.0)),
            )
            .show(ctx, |ui| {
                // Everything in ONE horizontal row:
                // Left: [Pixel Logo] ProctorX  •  SECURE. MONITOR. CONTROL.
                // Right: [🔓 UNLOCKED] [⏹ Recorder Idle] [☀ Cream Mode]
                ui.horizontal(|ui| {
                    p.draw_proctorx_pixel_logo(ui, 36.0);
                    ui.add_space(6.0);
                    ui.label(p.proctorx_wordmark_job(22.0));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("SECURE. MONITOR. CONTROL.")
                            .size(11.5)
                            .strong()
                            .color(p.gold_core),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let theme_label = if p.is_dark {
                            "☀ Cream Mode"
                        } else {
                            "🌙 Navy Mode"
                        };
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(theme_label)
                                        .size(12.0)
                                        .strong()
                                        .color(p.text_on_gold),
                                )
                                .fill(p.gold_core)
                                .rounding(Rounding::same(12.0))
                                .min_size(egui::vec2(110.0, 28.0)),
                            )
                            .on_hover_text("Switch between #0D3B66 Navy Mode and #FAF0CA Cream Mode")
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
                            p.badge(ui, "⏹ Recorder Idle", p.cream_core, p.navy_core);
                        }

                        if self.session.is_locked {
                            p.badge(ui, "🔒 LOCKED", p.danger_fg, p.danger_bg);
                        } else {
                            p.badge(ui, "🔓 UNLOCKED", p.text_on_gold, p.gold_core);
                        }
                    });
                });

                // Symmetrical 4-Column Stepper Bar (shown when Unlocked)
                if !self.session.is_locked {
                    ui.add_space(10.0);
                    ui.columns(4, |cols| {
                        for (idx, page) in WizardPage::ALL.iter().enumerate() {
                            let is_active = self.current_page == *page;
                            let fill = if is_active {
                                p.gold_core
                            } else {
                                p.navy_core
                            };
                            let text_col = if is_active {
                                p.text_on_gold
                            } else {
                                p.cream_core
                            };
                            let border_col = if is_active {
                                p.cream_core
                            } else {
                                p.gold_core.gamma_multiply(0.55)
                            };

                            let btn_w = cols[idx].available_width();
                            let btn = egui::Button::new(
                                RichText::new(page.short_tab_label())
                                    .size(13.0)
                                    .strong()
                                    .color(text_col),
                            )
                            .fill(fill)
                            .stroke(Stroke::new(1.2_f32, border_col))
                            .rounding(Rounding::same(8.0))
                            .min_size(egui::vec2(btn_w, 34.0));

                            if cols[idx].add(btn).clicked() {
                                self.current_page = *page;
                            }
                        }
                    });
                }
            });

        // ====================================================================
        // SYMMETRICAL BOTTOM NAVIGATION & ACTIVITY LOG PANEL
        // ====================================================================
        egui::TopBottomPanel::bottom("footer_log_panel")
            .frame(
                egui::Frame::none()
                    .fill(p.bg_header)
                    .stroke(Stroke::new(1.2_f32, p.gold_core.gamma_multiply(0.6)))
                    .inner_margin(Margin::symmetric(24.0, 10.0)),
            )
            .show(ctx, |ui| {
                if !self.session.is_locked {
                    let nav_btn_w = 195.0;
                    let nav_btn_h = 36.0;

                    ui.horizontal(|ui| {
                        // Left Button (Fixed 195x36)
                        if let Some(prev_page) = self.current_page.prev() {
                            let back_btn = egui::Button::new(
                                RichText::new(format!("← Back: {}", prev_page.short_tab_label()))
                                    .size(13.0)
                                    .strong()
                                    .color(p.cream_core),
                            )
                            .fill(p.navy_core)
                            .stroke(Stroke::new(1.2_f32, p.gold_core))
                            .rounding(Rounding::same(8.0))
                            .min_size(egui::vec2(nav_btn_w, nav_btn_h));

                            if ui.add(back_btn).clicked() {
                                self.current_page = prev_page;
                            }
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Button::new(RichText::new("← First Step").size(13.0))
                                    .rounding(Rounding::same(8.0))
                                    .min_size(egui::vec2(nav_btn_w, nav_btn_h)),
                            );
                        }

                        // Right Button placed via right_to_left first so Center gets exact remaining width
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(next_page) = self.current_page.next() {
                                let next_btn = egui::Button::new(
                                    RichText::new(format!(
                                        "Next: {} →",
                                        next_page.short_tab_label()
                                    ))
                                    .size(13.0)
                                    .strong()
                                    .color(p.text_on_gold),
                                )
                                .fill(p.gold_core)
                                .stroke(Stroke::new(1.2_f32, p.cream_core))
                                .rounding(Rounding::same(8.0))
                                .min_size(egui::vec2(nav_btn_w, nav_btn_h));

                                if ui.add(next_btn).clicked() {
                                    self.current_page = next_page;
                                }
                            } else {
                                let lock_now_btn = egui::Button::new(
                                    RichText::new("🔒 Activate Lockdown")
                                        .size(13.0)
                                        .strong()
                                        .color(p.text_on_gold),
                                )
                                .fill(p.gold_core)
                                .stroke(Stroke::new(1.2_f32, p.cream_core))
                                .rounding(Rounding::same(8.0))
                                .min_size(egui::vec2(nav_btn_w, nav_btn_h));

                                if ui.add(lock_now_btn).clicked() {
                                    self.handle_start_lockdown();
                                }
                            }

                            // Center Title centered across the exact remaining middle gap
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), nav_btn_h),
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "Page {} of 4 • {}",
                                            self.current_page.step_number(),
                                            self.current_page.title()
                                        ))
                                        .size(13.0)
                                        .strong()
                                        .color(p.cream_core),
                                    );
                                },
                            );
                        });
                    });
                    ui.add_space(6.0);
                    ui.separator();
                }

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Activity Log")
                            .size(11.5)
                            .strong()
                            .color(p.gold_core),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(RichText::new("Clear").color(p.text_primary))
                            .clicked()
                        {
                            self.status_log.clear();
                            self.push_log("Log cleared.", false);
                        }
                    });
                });
                egui::ScrollArea::vertical()
                    .max_height(46.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (line, is_err) in &self.status_log {
                            let color = if *is_err {
                                Color32::from_rgb(255, 138, 138)
                            } else {
                                p.cream_core
                            };
                            ui.label(RichText::new(line).size(11.5).color(color).monospace());
                        }
                    });
            });

        // ====================================================================
        // MAIN CENTRAL CONTENT (Symmetrical 24px Left/Right Margins)
        // ====================================================================
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(p.bg_canvas)
                    .inner_margin(Margin::symmetric(24.0, 12.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if !self.session.is_locked {
                        match self.current_page {
                            WizardPage::IdeSetup => {
                                self.render_page_1_ide(ui, p, is_installing_ide);
                            }
                            WizardPage::VideoRecording => {
                                self.render_page_2_recording(ui, p, is_installing_ffmpeg);
                            }
                            WizardPage::WebsitePolicy => {
                                self.render_page_3_website(ui, p);
                            }
                            WizardPage::ActivateLockdown => {
                                self.render_page_4_activate(ui, p);
                            }
                        }
                    } else {
                        self.render_locked_session(ui, p);
                    }
                });
            });
    }
}

impl ContestLockdownApp {
    // ========================================================================
    // PAGE 1: CONTEST IDE INSTALLER & LAUNCHER
    // ========================================================================
    fn render_page_1_ide(&mut self, ui: &mut egui::Ui, p: AppPalette, is_installing_ide: bool) {
        let title_col = if p.is_dark { p.gold_core } else { p.navy_core };

        p.card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Contest IDE Installer & Launcher")
                        .size(18.0)
                        .strong()
                        .color(title_col),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Clear").clicked() {
                        self.selected_ides.clear();
                    }
                    if ui.small_button("Select Installed").clicked() {
                        self.selected_ides = IdeChoice::ALL
                            .into_iter()
                            .filter(|i| i.is_installed())
                            .collect();
                    }
                    if ui.small_button("Select All").clicked() {
                        self.selected_ides = IdeChoice::ALL.to_vec();
                    }

                    if is_installing_ide {
                        p.badge(
                            ui,
                            &format!("⏳ {}", self.ide_installer.status_summary()),
                            p.warning_fg,
                            p.warning_bg,
                        );
                    } else {
                        p.badge(
                            ui,
                            &format!("✔ {} Selected", self.selected_ides.len()),
                            p.text_on_gold,
                            p.gold_core,
                        );
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Render uniform, symmetrical IDE rows
            for ide in IdeChoice::ALL {
                let mut is_checked = self.selected_ides.contains(&ide);
                let ide_installed = ide.is_installed();
                let ide_queued = self.ide_installer.is_ide_active_or_queued(ide);

                let card_bg = if is_checked {
                    if p.is_dark {
                        Color32::from_rgb(26, 92, 152)
                    } else {
                        Color32::from_rgb(249, 232, 162)
                    }
                } else {
                    p.bg_input
                };
                let border_col = if is_checked {
                    p.gold_core
                } else {
                    p.border_subtle
                };

                egui::Frame::none()
                    .fill(card_bg)
                    .stroke(Stroke::new(
                        if is_checked { 1.6_f32 } else { 1.0_f32 },
                        border_col,
                    ))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::symmetric(14.0, 8.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            if ui
                                .checkbox(
                                    &mut is_checked,
                                    RichText::new(ide.display_name()).size(14.0).strong(),
                                )
                                .changed()
                            {
                                if is_checked {
                                    if !self.selected_ides.contains(&ide) {
                                        self.selected_ides.push(ide);
                                    }
                                } else {
                                    self.selected_ides.retain(|&x| x != ide);
                                }
                            }

                            ui.label(
                                RichText::new(format!("— {}", ide.short_description()))
                                    .size(12.0)
                                    .color(p.text_secondary),
                            );

                            // Right-aligned uniform 96px action button + status label
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ide_installed {
                                    let btn = egui::Button::new(
                                        RichText::new("🚀 Launch")
                                            .size(12.0)
                                            .strong()
                                            .color(p.text_on_gold),
                                    )
                                    .fill(p.gold_core)
                                    .min_size(egui::vec2(96.0, 26.0));
                                    if ui.add(btn).clicked() {
                                        match ide.launch() {
                                            Ok(m) => self.push_log(m, false),
                                            Err(e) => self.push_log(e, true),
                                        }
                                    }
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("✔ Installed")
                                            .size(12.0)
                                            .strong()
                                            .color(title_col),
                                    );
                                } else if ide_queued {
                                    ui.add_enabled(
                                        false,
                                        egui::Button::new(RichText::new("⏳ Queued").size(12.0))
                                            .min_size(egui::vec2(96.0, 26.0)),
                                    );
                                    ui.add_space(6.0);
                                    ui.spinner();
                                } else {
                                    let btn = egui::Button::new(
                                        RichText::new("⬇ Install")
                                            .size(12.0)
                                            .strong()
                                            .color(p.cream_core),
                                    )
                                    .fill(p.navy_core)
                                    .stroke(Stroke::new(1.0_f32, p.gold_core))
                                    .min_size(egui::vec2(96.0, 26.0));
                                    if ui.add(btn).clicked() {
                                        match self.ide_installer.enqueue_installs(&[ide], false) {
                                            Ok(m) => self.push_log(m, false),
                                            Err(e) => self.push_log(e, true),
                                        }
                                    }
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("Not Installed")
                                            .size(12.0)
                                            .color(p.text_muted),
                                    );
                                }
                            });
                        });
                    });
                ui.add_space(3.0);
            }

            let selected_missing: Vec<IdeChoice> = self
                .selected_ides
                .iter()
                .copied()
                .filter(|i| !i.is_installed())
                .collect();
            let selected_installed: Vec<IdeChoice> = self
                .selected_ides
                .iter()
                .copied()
                .filter(|i| i.is_installed())
                .collect();

            ui.add_space(6.0);

            // Symmetrical 2-Column Batch Action Buttons + Auto-Checkboxes
            ui.columns(2, |cols| {
                let col0_w = cols[0].available_width();
                let install_btn = egui::Button::new(
                    RichText::new(format!(
                        "⬇ Install Selected Missing IDEs ({})",
                        selected_missing.len()
                    ))
                    .size(13.0)
                    .strong()
                    .color(p.text_on_gold),
                )
                .fill(p.gold_core)
                .rounding(Rounding::same(8.0))
                .min_size(egui::vec2(col0_w, 36.0));

                if cols[0]
                    .add_enabled(!selected_missing.is_empty(), install_btn)
                    .clicked()
                {
                    match self
                        .ide_installer
                        .enqueue_installs(&selected_missing, false)
                    {
                        Ok(m) => self.push_log(m, false),
                        Err(e) => self.push_log(e, true),
                    }
                }
                cols[0].add_space(4.0);
                cols[0].checkbox(
                    &mut self.auto_install_ide,
                    "Auto-install missing IDEs on Lockdown",
                );

                let col1_w = cols[1].available_width();
                let launch_btn = egui::Button::new(
                    RichText::new(format!(
                        "🚀 Launch Selected Installed IDEs ({})",
                        selected_installed.len()
                    ))
                    .size(13.0)
                    .strong()
                    .color(p.text_on_gold),
                )
                .fill(p.gold_core)
                .rounding(Rounding::same(8.0))
                .min_size(egui::vec2(col1_w, 36.0));

                if cols[1]
                    .add_enabled(!selected_installed.is_empty(), launch_btn)
                    .clicked()
                {
                    for ide in &selected_installed {
                        match ide.launch() {
                            Ok(m) => self.push_log(m, false),
                            Err(e) => self.push_log(e, true),
                        }
                    }
                }
                cols[1].add_space(4.0);
                cols[1].checkbox(
                    &mut self.auto_launch_ide,
                    "Auto-launch installed IDEs on Lockdown",
                );
            });
        });
    }

    // ========================================================================
    // PAGE 2: AUTOMATED DESKTOP VIDEO RECORDING
    // ========================================================================
    fn render_page_2_recording(
        &mut self,
        ui: &mut egui::Ui,
        p: AppPalette,
        is_installing_ffmpeg: bool,
    ) {
        let title_col = if p.is_dark { p.gold_core } else { p.navy_core };
        let ffmpeg_ok = ScreenRecorder::is_ffmpeg_available();

        p.card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Automated Desktop Video Recording")
                        .size(18.0)
                        .strong()
                        .color(title_col),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if is_installing_ffmpeg {
                        p.badge(ui, "⏳ Installing FFmpeg...", p.warning_fg, p.warning_bg);
                    } else if ffmpeg_ok {
                        p.badge(ui, "✔ FFmpeg Ready", p.success_fg, p.success_bg);
                    } else {
                        p.badge(ui, "⚠ FFmpeg Missing", p.warning_fg, p.warning_bg);
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.columns(2, |cols| {
                    cols[0].checkbox(
                        &mut self.auto_start_recording,
                        RichText::new("Auto-start full-screen MP4 recording on Lockdown")
                            .size(13.5)
                            .strong(),
                    );
                    cols[1].checkbox(
                        &mut self.recorder.capture_audio,
                        RichText::new("Include system/microphone audio track")
                            .size(13.5),
                    );
                });

                ui.add_space(12.0);
                ui.label(
                    RichText::new("Video Output Directory:")
                        .size(13.5)
                        .strong()
                        .color(p.text_primary),
                );
                ui.add_space(4.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let folder_btn = egui::Button::new(
                        RichText::new("📂 Open Recordings Folder")
                            .size(13.5)
                            .strong()
                            .color(p.text_on_gold),
                    )
                    .fill(p.gold_core)
                    .rounding(Rounding::same(8.0))
                    .min_size(egui::vec2(210.0, 36.0));

                    if ui.add(folder_btn).clicked() {
                        if let Err(e) =
                            ScreenRecorder::open_recordings_folder(&self.recordings_dir)
                        {
                            self.push_log(format!("Could not open folder: {}", e), true);
                        }
                    }

                    ui.add(
                        egui::TextEdit::singleline(&mut self.recordings_dir)
                            .hint_text("recordings")
                            .font(FontId::proportional(15.0))
                            .margin(Margin::symmetric(12.0, 8.0))
                            .desired_width(ui.available_width()),
                    );
                });
            });

            ui.add_space(12.0);
            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.columns(3, |cols| {
                    cols[0].vertical_centered(|ui| {
                        ui.label(RichText::new("H.264 MP4 Encoder").strong().color(title_col));
                        ui.label(
                            RichText::new("libx264 ultrafast • 15 FPS")
                                .size(12.0)
                                .color(p.text_secondary),
                        );
                    });
                    cols[1].vertical_centered(|ui| {
                        ui.label(RichText::new("Cross-OS Capture").strong().color(title_col));
                        ui.label(
                            RichText::new("Win GDI • macOS • Linux X11")
                                .size(12.0)
                                .color(p.text_secondary),
                        );
                    });
                    cols[2].vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Crash-Safe Finalize")
                                .strong()
                                .color(title_col),
                        );
                        ui.label(
                            RichText::new("Auto-saves MP4 on exit")
                                .size(12.0)
                                .color(p.text_secondary),
                        );
                    });
                });

                if !ffmpeg_ok && !is_installing_ffmpeg {
                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        let ff_btn = egui::Button::new(
                            RichText::new("⬇ Install FFmpeg Engine Automatically")
                                .size(13.5)
                                .strong()
                                .color(p.text_on_gold),
                        )
                        .fill(p.gold_core)
                        .rounding(Rounding::same(8.0))
                        .min_size(egui::vec2(300.0, 36.0));

                        if ui.add(ff_btn).clicked() {
                            match self.recorder.start_ffmpeg_install() {
                                Ok(m) => self.push_log(m, false),
                                Err(e) => self.push_log(e, true),
                            }
                        }
                    });
                }
            });
        });
    }

    // ========================================================================
    // PAGE 3: WEBSITE WHITELIST & BROWSER POLICY
    // ========================================================================
    fn render_page_3_website(&mut self, ui: &mut egui::Ui, p: AppPalette) {
        let title_col = if p.is_dark { p.gold_core } else { p.navy_core };
        let total_sites = 1 + self.extra_domains.len();

        p.card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Website Whitelist & Browser Policy")
                        .size(18.0)
                        .strong()
                        .color(title_col),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    p.badge(
                        ui,
                        &format!("🌐 {} Allowed Site(s)", total_sites),
                        p.text_on_gold,
                        p.gold_core,
                    );
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // ----------------------------------------------------------------
            // SUB-PANEL 1: PRIMARY CONTEST DOMAIN
            // ----------------------------------------------------------------
            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Primary Contest Platform")
                        .size(14.0)
                        .strong()
                        .color(title_col),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    egui::Frame::none()
                        .fill(p.gold_core)
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("https://")
                                    .size(14.5)
                                    .strong()
                                    .monospace()
                                    .color(p.text_on_gold),
                            );
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let test_btn = egui::Button::new(
                            RichText::new("🌐 Test Link")
                                .size(13.5)
                                .strong()
                                .color(p.text_on_gold),
                        )
                        .fill(p.gold_core)
                        .rounding(Rounding::same(8.0))
                        .min_size(egui::vec2(130.0, 36.0));

                        if ui
                            .add(test_btn)
                            .on_hover_text("Open this URL in your browser to verify")
                            .clicked()
                        {
                            let clean = normalize_domain(&self.domain_input);
                            if !clean.is_empty() {
                                let _ = open_allowed_website(&clean);
                            }
                        }

                        ui.add(
                            egui::TextEdit::singleline(&mut self.domain_input)
                                .hint_text("bapsoj.org")
                                .font(FontId::proportional(15.0))
                                .margin(Margin::symmetric(12.0, 8.0))
                                .desired_width(ui.available_width()),
                        );
                    });
                });
            });

            ui.add_space(8.0);

            // ----------------------------------------------------------------
            // SUB-PANEL 2: REFERENCE & DOCUMENTATION WHITELIST
            // ----------------------------------------------------------------
            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Reference & Judge Whitelist")
                        .size(14.0)
                        .strong()
                        .color(title_col),
                );
                ui.add_space(6.0);

                let presets = [
                    ("cppreference.com", "C/C++ Docs"),
                    ("codeforces.com", "Codeforces"),
                    ("atcoder.jp", "AtCoder"),
                    ("vjudge.net", "VJudge"),
                ];

                ui.columns(4, |cols| {
                    for (idx, (domain, label)) in presets.iter().enumerate() {
                        let is_enabled = self.extra_domains.iter().any(|d| d == *domain);
                        let fill = if is_enabled {
                            p.gold_core
                        } else {
                            p.bg_card
                        };
                        let fg = if is_enabled {
                            p.text_on_gold
                        } else {
                            p.text_primary
                        };
                        let prefix = if is_enabled { "✔" } else { "+" };

                        let pill_btn = egui::Button::new(
                            RichText::new(format!("{} {}\n({})", prefix, label, domain))
                                .size(11.5)
                                .strong()
                                .color(fg),
                        )
                        .fill(fill)
                        .stroke(Stroke::new(
                            1.0_f32,
                            if is_enabled {
                                p.cream_core
                            } else {
                                p.border_subtle
                            },
                        ))
                        .rounding(Rounding::same(8.0))
                        .min_size(egui::vec2(cols[idx].available_width(), 40.0));

                        if cols[idx].add(pill_btn).clicked() {
                            self.toggle_extra_domain(domain);
                        }
                    }
                });

                ui.add_space(8.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_btn = egui::Button::new(
                        RichText::new("+ Add Domain")
                            .size(13.5)
                            .strong()
                            .color(p.text_on_gold),
                    )
                    .fill(p.gold_core)
                    .rounding(Rounding::same(8.0))
                    .min_size(egui::vec2(145.0, 36.0));

                    let add_clicked = ui.add(add_btn).clicked();

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.new_extra_domain_input)
                            .hint_text("Custom domain (e.g. docs.python.org)")
                            .font(FontId::proportional(15.0))
                            .margin(Margin::symmetric(12.0, 8.0))
                            .desired_width(ui.available_width()),
                    );

                    let enter_pressed =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if (add_clicked || enter_pressed) && !self.new_extra_domain_input.is_empty() {
                        let to_add = self.new_extra_domain_input.clone();
                        self.add_extra_domain(&to_add);
                        self.new_extra_domain_input.clear();
                    }
                });

                if !self.extra_domains.is_empty() {
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        let mut remove_idx = None;
                        for (idx, dom) in self.extra_domains.iter().enumerate() {
                            egui::Frame::none()
                                .fill(p.gold_core)
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(8.0, 3.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(dom)
                                                .size(12.0)
                                                .strong()
                                                .color(p.text_on_gold),
                                        );
                                        if ui
                                            .small_button(
                                                RichText::new("✕").color(p.text_on_gold),
                                            )
                                            .clicked()
                                        {
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
            });

            ui.add_space(8.0);

            // ----------------------------------------------------------------
            // SUB-PANEL 3: SYMMETRICAL 2-COLUMN SPLIT
            // ----------------------------------------------------------------
            ui.columns(2, |cols| {
                p.sub_card_frame().show(&mut cols[0], |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(64.0);
                    ui.label(
                        RichText::new("Browser Download Policy")
                            .size(14.0)
                            .strong()
                            .color(title_col),
                    );
                    ui.add_space(4.0);
                    ui.radio_value(
                        &mut self.block_all_downloads,
                        false,
                        "Allow Safe Contest Files (.txt, .zip, .pdf)",
                    );
                    ui.radio_value(
                        &mut self.block_all_downloads,
                        true,
                        "Block All File Downloads Completely",
                    );
                });

                p.sub_card_frame().show(&mut cols[1], |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(64.0);
                    ui.label(
                        RichText::new("Auto-Unlock Contest Timer")
                            .size(14.0)
                            .strong()
                            .color(title_col),
                    );
                    ui.add_space(8.0);
                    ui.columns(5, |t_cols| {
                        for (idx, (mins, label)) in [
                            (0, "Manual"),
                            (60, "1h"),
                            (120, "2h"),
                            (180, "3h"),
                            (300, "5h"),
                        ]
                        .iter()
                        .enumerate()
                        {
                            let is_sel = self.duration_minutes == *mins;
                            let btn = egui::Button::new(
                                RichText::new(*label)
                                    .size(12.5)
                                    .strong()
                                    .color(if is_sel {
                                        p.text_on_gold
                                    } else {
                                        p.text_primary
                                    }),
                            )
                            .fill(if is_sel { p.gold_core } else { p.bg_card })
                            .rounding(Rounding::same(6.0))
                            .min_size(egui::vec2(t_cols[idx].available_width(), 32.0));

                            if t_cols[idx].add(btn).clicked() {
                                self.duration_minutes = *mins;
                            }
                        }
                    });
                });
            });
        });
    }

    // ========================================================================
    // PAGE 4: SET SESSION PASSWORD & ACTIVATE LOCKDOWN
    // ========================================================================
    fn render_page_4_activate(&mut self, ui: &mut egui::Ui, p: AppPalette) {
        let title_col = if p.is_dark { p.gold_core } else { p.navy_core };

        p.card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(
                RichText::new("Set Session Password & Activate Lockdown")
                    .size(18.0)
                    .strong()
                    .color(title_col),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Summary Box of Pages 1, 2, and 3 with uniform right-aligned Edit buttons
            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Configuration Summary")
                        .size(14.0)
                        .strong()
                        .color(title_col),
                );
                ui.add_space(6.0);

                let ide_list_str = if self.selected_ides.is_empty() {
                    "None selected".to_string()
                } else {
                    self.selected_ides
                        .iter()
                        .map(|i| i.display_name())
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("1. IDEs: {}", ide_list_str))
                            .size(13.0)
                            .color(p.text_primary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn = egui::Button::new(RichText::new("Edit").size(12.0))
                            .min_size(egui::vec2(72.0, 24.0));
                        if ui.add(btn).clicked() {
                            self.current_page = WizardPage::IdeSetup;
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "2. Recording: {} ({}/)",
                            if self.auto_start_recording {
                                "Enabled"
                            } else {
                                "Disabled"
                            },
                            self.recordings_dir
                        ))
                        .size(13.0)
                        .color(p.text_primary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn = egui::Button::new(RichText::new("Edit").size(12.0))
                            .min_size(egui::vec2(72.0, 24.0));
                        if ui.add(btn).clicked() {
                            self.current_page = WizardPage::VideoRecording;
                        }
                    });
                });

                ui.horizontal(|ui| {
                    let extra_count = self.extra_domains.len();
                    ui.label(
                        RichText::new(format!(
                            "3. Whitelist: https://{}{} ({})",
                            normalize_domain(&self.domain_input),
                            if extra_count > 0 {
                                format!(" + {} extra", extra_count)
                            } else {
                                String::new()
                            },
                            if self.duration_minutes == 0 {
                                "Manual Unlock".to_string()
                            } else {
                                format!("{} mins", self.duration_minutes)
                            }
                        ))
                        .size(13.0)
                        .color(p.text_primary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn = egui::Button::new(RichText::new("Edit").size(12.0))
                            .min_size(egui::vec2(72.0, 24.0));
                        if ui.add(btn).clicked() {
                            self.current_page = WizardPage::WebsitePolicy;
                        }
                    });
                });
            });

            ui.add_space(10.0);

            let (strength_score, strength_label) =
                evaluate_password_strength(&self.lock_password_input);
            let strength_color = match strength_score {
                0 => p.text_muted,
                1 => Color32::from_rgb(255, 120, 120),
                2 => p.gold_core,
                _ => Color32::from_rgb(110, 231, 183),
            };

            // Symmetrical Full-Width Password Setup Card
            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Session Password")
                            .size(14.0)
                            .strong()
                            .color(title_col),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let show_btn = egui::Button::new(
                            RichText::new(if self.show_lock_password {
                                "🙈 Hide"
                            } else {
                                "👁 Show"
                            })
                            .size(12.0)
                            .strong(),
                        )
                        .min_size(egui::vec2(90.0, 26.0));
                        if ui.add(show_btn).clicked() {
                            self.show_lock_password = !self.show_lock_password;
                        }
                    });
                });
                ui.add_space(8.0);

                // Row 1: Set Session Password
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(155.0, 36.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(
                                RichText::new("Password:")
                                    .size(14.0)
                                    .strong()
                                    .color(p.text_primary),
                            );
                        },
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(125.0, 36.0),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    RichText::new(format!("Strength: {}", strength_label))
                                        .size(12.5)
                                        .strong()
                                        .color(strength_color),
                                );
                            },
                        );

                        ui.add(
                            egui::TextEdit::singleline(&mut self.lock_password_input)
                                .password(!self.show_lock_password)
                                .hint_text("Enter session password")
                                .font(FontId::proportional(15.0))
                                .margin(Margin::symmetric(12.0, 8.0))
                                .desired_width(ui.available_width()),
                        );
                    });
                });

                ui.add_space(6.0);

                // Row 2: Confirm Password
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(155.0, 36.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(
                                RichText::new("Confirm Password:")
                                    .size(14.0)
                                    .strong()
                                    .color(p.text_primary),
                            );
                        },
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(125.0, 36.0),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if !self.lock_password_confirm.is_empty() {
                                    if self.lock_password_input == self.lock_password_confirm {
                                        ui.label(
                                            RichText::new("✔ Match")
                                                .size(12.5)
                                                .strong()
                                                .color(title_col),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new("✕ Mismatch")
                                                .size(12.5)
                                                .strong()
                                                .color(Color32::from_rgb(255, 120, 120)),
                                        );
                                    }
                                } else {
                                    ui.label(
                                        RichText::new("Required")
                                            .size(12.0)
                                            .color(p.text_muted),
                                    );
                                }
                            },
                        );

                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.lock_password_confirm)
                                .password(!self.show_lock_password)
                                .hint_text("Re-enter password")
                                .font(FontId::proportional(15.0))
                                .margin(Margin::symmetric(12.0, 8.0))
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.handle_start_lockdown();
                        }
                    });
                });
            });

            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                let cta_button = egui::Button::new(
                    RichText::new("🔒 Set Session Password & Activate Lockdown")
                        .size(16.0)
                        .strong()
                        .color(p.text_on_gold),
                )
                .fill(p.gold_core)
                .stroke(Stroke::new(1.5_f32, p.cream_core))
                .rounding(Rounding::same(10.0))
                .min_size(egui::vec2(460.0, 44.0));

                if ui.add(cta_button).clicked() {
                    self.handle_start_lockdown();
                }
            });
        });
    }

    // ========================================================================
    // LOCKED SESSION VIEW (PASSWORD UNLOCK)
    // ========================================================================
    fn render_locked_session(&mut self, ui: &mut egui::Ui, p: AppPalette) {
        let title_col = if p.is_dark { p.gold_core } else { p.navy_core };

        p.banner_frame(p.danger_bg, p.gold_core).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                p.draw_proctorx_pixel_logo(ui, 40.0);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("🔒 PROCTORX LOCKDOWN ACTIVE")
                            .size(17.0)
                            .strong()
                            .color(p.gold_core),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Allowed: https://{}",
                            self.session.allowed_domain
                        ))
                        .size(13.0)
                        .color(p.cream_core),
                    );
                });

                if let Some(rem_secs) = self.session.remaining_timer_secs() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let hrs = rem_secs / 3600;
                        let mins = (rem_secs % 3600) / 60;
                        let secs = rem_secs % 60;
                        p.badge(
                            ui,
                            &format!("⏱ {:02}:{:02}:{:02}", hrs, mins, secs),
                            p.warning_fg,
                            p.warning_bg,
                        );
                    });
                }
            });
        });

        ui.add_space(12.0);

        p.card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("Quick Actions")
                    .size(15.5)
                    .strong()
                    .color(title_col),
            );
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                for dom in self.session.all_allowed_domains() {
                    if ui.button(format!("🌐 Open {}", dom)).clicked() {
                        let _ = open_allowed_website(&dom);
                    }
                }

                for ide_name in self.session.all_selected_ides() {
                    if ui.button(format!("💻 Launch {}", ide_name)).clicked() {
                        match IdeChoice::from_name(&ide_name).launch() {
                            Ok(m) => self.push_log(m, false),
                            Err(e) => self.push_log(e, true),
                        }
                    }
                }

                if ui.button("📂 Recordings Folder").clicked() {
                    let _ = ScreenRecorder::open_recordings_folder(&self.recordings_dir);
                }
            });
        });

        ui.add_space(12.0);

        p.card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("Unlock Contest Environment")
                    .size(15.5)
                    .strong()
                    .color(title_col),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(145.0, 36.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new("Session Password:")
                                .size(14.0)
                                .strong()
                                .color(p.text_primary),
                        );
                    },
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let show_btn = egui::Button::new(
                        RichText::new(if self.show_unlock_password {
                            "🙈 Hide"
                        } else {
                            "👁 Show"
                        })
                        .size(13.0)
                        .strong(),
                    )
                    .min_size(egui::vec2(90.0, 36.0));
                    if ui.add(show_btn).clicked() {
                        self.show_unlock_password = !self.show_unlock_password;
                    }

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.unlock_password_input)
                            .password(!self.show_unlock_password)
                            .hint_text("Enter session password")
                            .font(FontId::proportional(15.0))
                            .margin(Margin::symmetric(12.0, 8.0))
                            .desired_width(ui.available_width()),
                    );
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.handle_unlock_session();
                    }
                });
            });

            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                let unlock_btn = egui::Button::new(
                    RichText::new("🔓 Verify Password & Unlock")
                        .size(15.5)
                        .strong()
                        .color(p.text_on_gold),
                )
                .fill(p.gold_core)
                .stroke(Stroke::new(1.5_f32, p.cream_core))
                .rounding(Rounding::same(10.0))
                .min_size(egui::vec2(380.0, 42.0));

                if ui.add(unlock_btn).clicked() {
                    self.handle_unlock_session();
                }
            });
        });
    }
}
