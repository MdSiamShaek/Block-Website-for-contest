#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ide;
mod policy;
mod recorder;
mod state;
mod theme;

use eframe::egui::{self, Color32, FontId, Margin, RichText, Rounding, Stroke, TextureHandle, Vec2};
use ide::{IdeChoice, IdeInstallTracker};
use policy::{disable_browser_lockdown, enable_browser_lockdown, open_allowed_website};
use recorder::ScreenRecorder;
use state::{
    evaluate_password_strength, normalize_domain, verify_password, LockdownSession,
};
use std::time::Duration;
use theme::{build_window_icon, load_logo_texture, AppPalette};

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
            .with_min_inner_size([760.0, 640.0])
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
    logo_texture: TextureHandle,
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
        let logo_texture = load_logo_texture(&cc.egui_ctx);

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
            logo_texture,
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
        // SINGLE-ROW PROCTORX TOP HEADER BAR (USING "ProctorX Logo.png")
        // ====================================================================
        egui::TopBottomPanel::top("header_panel")
            .frame(
                egui::Frame::none()
                    .fill(p.bg_header)
                    .stroke(Stroke::new(1.5_f32, p.gold_core))
                    .inner_margin(Margin::symmetric(24.0, 12.0)),
            )
            .show(ctx, |ui| {
                // Single horizontal row:
                // Left: [ProctorX Logo.png] ProctorX  •  SECURE. MONITOR. CONTROL.
                // Right: [🔓 UNLOCKED] [⏹ Recorder Idle] [☀ Cream Mode]
                ui.horizontal(|ui| {
                    p.draw_logo_image(ui, &self.logo_texture, Vec2::splat(40.0));
                    ui.add_space(6.0);
                    ui.label(p.proctorx_wordmark_job(23.0));
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
                        if p
                            .centered_button(
                                ui,
                                Vec2::new(114.0, 28.0),
                                theme_label,
                                12.0,
                                p.gold_core,
                                p.text_on_gold,
                                Stroke::NONE,
                                12.0,
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

                // Evenly Spaced, Centered 4-Tab Navigation Bar (shown when Unlocked)
                if !self.session.is_locked {
                    ui.add_space(10.0);
                    let gap = 12.0_f32;
                    let total_w = ui.available_width();
                    let tab_w = ((total_w - gap * 3.0) / 4.0).max(120.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for page in WizardPage::ALL {
                            let is_active = self.current_page == page;
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

                            if p
                                .centered_button(
                                    ui,
                                    Vec2::new(tab_w, 36.0),
                                    page.short_tab_label(),
                                    13.5,
                                    fill,
                                    text_col,
                                    Stroke::new(1.2_f32, border_col),
                                    8.0,
                                )
                                .clicked()
                            {
                                self.current_page = page;
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
                        // Left Button (Fixed 195x36, Centered Text)
                        if let Some(prev_page) = self.current_page.prev() {
                            if p
                                .centered_button(
                                    ui,
                                    Vec2::new(nav_btn_w, nav_btn_h),
                                    &format!("← Back: {}", prev_page.short_tab_label()),
                                    13.0,
                                    p.navy_core,
                                    p.cream_core,
                                    Stroke::new(1.2_f32, p.gold_core),
                                    8.0,
                                )
                                .clicked()
                            {
                                self.current_page = prev_page;
                            }
                        } else {
                            ui.add_enabled_ui(false, |ui| {
                                let _ = p.centered_button(
                                    ui,
                                    Vec2::new(nav_btn_w, nav_btn_h),
                                    "← First Step",
                                    13.0,
                                    p.navy_core,
                                    p.cream_core,
                                    Stroke::new(1.0_f32, p.border_subtle),
                                    8.0,
                                );
                            });
                        }

                        // Right Button placed via right_to_left first so Center gets exact remaining width
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(next_page) = self.current_page.next() {
                                if p
                                    .centered_button(
                                        ui,
                                        Vec2::new(nav_btn_w, nav_btn_h),
                                        &format!("Next: {} →", next_page.short_tab_label()),
                                        13.0,
                                        p.gold_core,
                                        p.text_on_gold,
                                        Stroke::new(1.2_f32, p.cream_core),
                                        8.0,
                                    )
                                    .clicked()
                                {
                                    self.current_page = next_page;
                                }
                            } else if p
                                .centered_button(
                                    ui,
                                    Vec2::new(nav_btn_w, nav_btn_h),
                                    "🔒 Activate Lockdown",
                                    13.0,
                                    p.gold_core,
                                    p.text_on_gold,
                                    Stroke::new(1.2_f32, p.cream_core),
                                    8.0,
                                )
                                .clicked()
                            {
                                self.handle_start_lockdown();
                            }

                            // Center Title centered across the exact remaining middle gap
                            ui.allocate_ui_with_layout(
                                Vec2::new(ui.available_width(), nav_btn_h),
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
                    .max_height(44.0)
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
                    .inner_margin(Margin::symmetric(24.0, 14.0)),
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
    // TAB 1: CONTEST IDE INSTALLER & LAUNCHER (BALANCED SPACING & CENTERED BTNS)
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

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Render uniform, evenly spaced IDE rows
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
                    .inner_margin(Margin::symmetric(16.0, 10.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            if ui
                                .checkbox(
                                    &mut is_checked,
                                    RichText::new(ide.display_name()).size(14.5).strong(),
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
                                    .size(12.5)
                                    .color(p.text_secondary),
                            );

                            // Right-aligned uniform 98x28 centered action button + status label
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ide_installed {
                                    if p
                                        .centered_button(
                                            ui,
                                            Vec2::new(98.0, 28.0),
                                            "🚀 Launch",
                                            12.5,
                                            p.gold_core,
                                            p.text_on_gold,
                                            Stroke::NONE,
                                            6.0,
                                        )
                                        .clicked()
                                    {
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
                                    ui.add_enabled_ui(false, |ui| {
                                        let _ = p.centered_button(
                                            ui,
                                            Vec2::new(98.0, 28.0),
                                            "⏳ Queued",
                                            12.0,
                                            p.navy_core,
                                            p.cream_core,
                                            Stroke::new(1.0_f32, p.border_subtle),
                                            6.0,
                                        );
                                    });
                                    ui.add_space(6.0);
                                    ui.spinner();
                                } else {
                                    if p
                                        .centered_button(
                                            ui,
                                            Vec2::new(98.0, 28.0),
                                            "⬇ Install",
                                            12.5,
                                            p.navy_core,
                                            p.cream_core,
                                            Stroke::new(1.0_f32, p.gold_core),
                                            6.0,
                                        )
                                        .clicked()
                                    {
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
                ui.add_space(2.0);
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

            ui.add_space(8.0);

            // Symmetrical 2-Column Batch Action Buttons + Auto-Checkboxes
            let col_gap = 16.0_f32;
            let half_w = ((ui.available_width() - col_gap) * 0.5).max(200.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = col_gap;

                ui.vertical(|ui| {
                    ui.set_width(half_w);
                    ui.add_enabled_ui(!selected_missing.is_empty(), |ui| {
                        if p
                            .centered_button(
                                ui,
                                Vec2::new(half_w, 38.0),
                                &format!(
                                    "⬇ Install Selected Missing IDEs ({})",
                                    selected_missing.len()
                                ),
                                13.5,
                                p.gold_core,
                                p.text_on_gold,
                                Stroke::new(1.2_f32, p.cream_core),
                                8.0,
                            )
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
                    });
                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut self.auto_install_ide,
                        "Auto-install missing IDEs on Lockdown",
                    );
                });

                ui.vertical(|ui| {
                    ui.set_width(half_w);
                    ui.add_enabled_ui(!selected_installed.is_empty(), |ui| {
                        if p
                            .centered_button(
                                ui,
                                Vec2::new(half_w, 38.0),
                                &format!(
                                    "🚀 Launch Selected Installed IDEs ({})",
                                    selected_installed.len()
                                ),
                                13.5,
                                p.gold_core,
                                p.text_on_gold,
                                Stroke::new(1.2_f32, p.cream_core),
                                8.0,
                            )
                            .clicked()
                        {
                            for ide in &selected_installed {
                                match ide.launch() {
                                    Ok(m) => self.push_log(m, false),
                                    Err(e) => self.push_log(e, true),
                                }
                            }
                        }
                    });
                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut self.auto_launch_ide,
                        "Auto-launch installed IDEs on Lockdown",
                    );
                });
            });
        });
    }

    // ========================================================================
    // TAB 2: AUTOMATED DESKTOP VIDEO RECORDING (BALANCED SPACING)
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
            ui.add_space(10.0);

            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.columns(2, |cols| {
                    cols[0].checkbox(
                        &mut self.auto_start_recording,
                        RichText::new("Auto-start full-screen MP4 recording on Lockdown")
                            .size(14.0)
                            .strong(),
                    );
                    cols[1].checkbox(
                        &mut self.recorder.capture_audio,
                        RichText::new("Include system/microphone audio track")
                            .size(14.0),
                    );
                });

                ui.add_space(14.0);
                ui.label(
                    RichText::new("Video Output Directory:")
                        .size(14.0)
                        .strong()
                        .color(p.text_primary),
                );
                ui.add_space(6.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if p
                        .centered_button(
                            ui,
                            Vec2::new(210.0, 38.0),
                            "📂 Open Recordings Folder",
                            13.5,
                            p.gold_core,
                            p.text_on_gold,
                            Stroke::NONE,
                            8.0,
                        )
                        .clicked()
                    {
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
                            .margin(Margin::symmetric(12.0, 9.0))
                            .desired_width(ui.available_width()),
                    );
                });
            });

            ui.add_space(14.0);
            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.columns(3, |cols| {
                    cols[0].vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("H.264 MP4 Encoder")
                                .size(14.0)
                                .strong()
                                .color(title_col),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("libx264 ultrafast • 15 FPS")
                                .size(12.5)
                                .color(p.text_secondary),
                        );
                        ui.add_space(4.0);
                    });
                    cols[1].vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Cross-OS Capture")
                                .size(14.0)
                                .strong()
                                .color(title_col),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("Win GDI • macOS • Linux X11")
                                .size(12.5)
                                .color(p.text_secondary),
                        );
                        ui.add_space(4.0);
                    });
                    cols[2].vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Crash-Safe Finalize")
                                .size(14.0)
                                .strong()
                                .color(title_col),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("Auto-saves MP4 on exit")
                                .size(12.5)
                                .color(p.text_secondary),
                        );
                        ui.add_space(4.0);
                    });
                });

                if !ffmpeg_ok && !is_installing_ffmpeg {
                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        if p
                            .centered_button(
                                ui,
                                Vec2::new(310.0, 38.0),
                                "⬇ Install FFmpeg Engine Automatically",
                                13.5,
                                p.gold_core,
                                p.text_on_gold,
                                Stroke::new(1.2_f32, p.cream_core),
                                8.0,
                            )
                            .clicked()
                        {
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
    // TAB 3: WEBSITE WHITELIST & BROWSER POLICY (BALANCED SPACING & CENTERED CARDS)
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

            ui.add_space(6.0);
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
                    // Left fixed 90x38 centered prefix pill
                    let _ = p.centered_button(
                        ui,
                        Vec2::new(90.0, 38.0),
                        "https://",
                        14.5,
                        p.gold_core,
                        p.text_on_gold,
                        Stroke::NONE,
                        8.0,
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if p
                            .centered_button(
                                ui,
                                Vec2::new(135.0, 38.0),
                                "🌐 Test Link",
                                13.5,
                                p.gold_core,
                                p.text_on_gold,
                                Stroke::NONE,
                                8.0,
                            )
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
                                .margin(Margin::symmetric(12.0, 9.0))
                                .desired_width(ui.available_width()),
                        );
                    });
                });
            });

            ui.add_space(10.0);

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

                let card_gap = 12.0_f32;
                let card_w = ((ui.available_width() - card_gap * 3.0) / 4.0).max(110.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = card_gap;
                    for (domain, label) in presets {
                        let is_enabled = self.extra_domains.iter().any(|d| d == domain);
                        let prefix = if is_enabled { "✔" } else { "+" };
                        let line1 = format!("{} {}", prefix, label);

                        if p
                            .centered_two_line_card(
                                ui,
                                Vec2::new(card_w, 44.0),
                                &line1,
                                domain,
                                is_enabled,
                            )
                            .clicked()
                        {
                            self.toggle_extra_domain(domain);
                        }
                    }
                });

                ui.add_space(10.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_clicked = p
                        .centered_button(
                            ui,
                            Vec2::new(150.0, 38.0),
                            "+ Add Domain",
                            13.5,
                            p.gold_core,
                            p.text_on_gold,
                            Stroke::NONE,
                            8.0,
                        )
                        .clicked();

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.new_extra_domain_input)
                            .hint_text("Custom domain (e.g. docs.python.org)")
                            .font(FontId::proportional(15.0))
                            .margin(Margin::symmetric(12.0, 9.0))
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
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        let mut remove_idx = None;
                        for (idx, dom) in self.extra_domains.iter().enumerate() {
                            egui::Frame::none()
                                .fill(p.gold_core)
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(10.0, 4.0))
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

            ui.add_space(10.0);

            // ----------------------------------------------------------------
            // SUB-PANEL 3: SYMMETRICAL 2-COLUMN SPLIT
            // ----------------------------------------------------------------
            let split_gap = 12.0_f32;
            let split_w = ((ui.available_width() - split_gap) * 0.5).max(200.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = split_gap;

                ui.allocate_ui(Vec2::new(split_w, 82.0), |ui| {
                    p.sub_card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_min_height(56.0);
                        ui.label(
                            RichText::new("Browser Download Policy")
                                .size(14.0)
                                .strong()
                                .color(title_col),
                        );
                        ui.add_space(6.0);
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
                });

                ui.allocate_ui(Vec2::new(split_w, 82.0), |ui| {
                    p.sub_card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_min_height(56.0);
                        ui.label(
                            RichText::new("Auto-Unlock Contest Timer")
                                .size(14.0)
                                .strong()
                                .color(title_col),
                        );
                        ui.add_space(10.0);

                        let t_gap = 8.0_f32;
                        let btn_w = ((ui.available_width() - t_gap * 4.0) / 5.0).max(42.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = t_gap;
                            for (mins, label) in [
                                (0, "Manual"),
                                (60, "1h"),
                                (120, "2h"),
                                (180, "3h"),
                                (300, "5h"),
                            ] {
                                let is_sel = self.duration_minutes == mins;
                                let fill = if is_sel { p.gold_core } else { p.bg_card };
                                let fg = if is_sel {
                                    p.text_on_gold
                                } else {
                                    p.text_primary
                                };
                                let border = Stroke::new(
                                    1.0_f32,
                                    if is_sel {
                                        p.cream_core
                                    } else {
                                        p.border_subtle
                                    },
                                );
                                if p
                                    .centered_button(
                                        ui,
                                        Vec2::new(btn_w, 34.0),
                                        label,
                                        12.5,
                                        fill,
                                        fg,
                                        border,
                                        6.0,
                                    )
                                    .clicked()
                                {
                                    self.duration_minutes = mins;
                                }
                            }
                        });
                    });
                });
            });
        });
    }

    // ========================================================================
    // TAB 4: SET SESSION PASSWORD & ACTIVATE LOCKDOWN (BALANCED SPACING)
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
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(8.0);

            // Summary Box of Pages 1, 2, and 3 with centered right-aligned Edit buttons
            p.sub_card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Configuration Summary")
                        .size(14.0)
                        .strong()
                        .color(title_col),
                );
                ui.add_space(8.0);

                let ide_list_str = if self.selected_ides.is_empty() {
                    "None selected".to_string()
                } else {
                    self.selected_ides
                        .iter()
                        .map(|i| i.display_name())
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let rows = [
                    (
                        format!("1. IDEs: {}", ide_list_str),
                        WizardPage::IdeSetup,
                    ),
                    (
                        format!(
                            "2. Recording: {} ({}/)",
                            if self.auto_start_recording {
                                "Enabled"
                            } else {
                                "Disabled"
                            },
                            self.recordings_dir
                        ),
                        WizardPage::VideoRecording,
                    ),
                    (
                        format!(
                            "3. Whitelist: https://{}{} ({})",
                            normalize_domain(&self.domain_input),
                            if !self.extra_domains.is_empty() {
                                format!(" + {} extra", self.extra_domains.len())
                            } else {
                                String::new()
                            },
                            if self.duration_minutes == 0 {
                                "Manual Unlock".to_string()
                            } else {
                                format!("{} mins", self.duration_minutes)
                            }
                        ),
                        WizardPage::WebsitePolicy,
                    ),
                ];

                for (text, target_page) in rows {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(text).size(13.5).color(p.text_primary));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if p
                                .centered_button(
                                    ui,
                                    Vec2::new(76.0, 26.0),
                                    "Edit",
                                    12.0,
                                    p.bg_card,
                                    p.text_primary,
                                    Stroke::new(1.0_f32, p.border_subtle),
                                    6.0,
                                )
                                .clicked()
                            {
                                self.current_page = target_page;
                            }
                        });
                    });
                    ui.add_space(4.0);
                }
            });

            ui.add_space(12.0);

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
                        let show_label = if self.show_lock_password {
                            "🙈 Hide"
                        } else {
                            "👁 Show"
                        };
                        if p
                            .centered_button(
                                ui,
                                Vec2::new(90.0, 26.0),
                                show_label,
                                12.0,
                                p.bg_card,
                                p.text_primary,
                                Stroke::new(1.0_f32, p.border_subtle),
                                6.0,
                            )
                            .clicked()
                        {
                            self.show_lock_password = !self.show_lock_password;
                        }
                    });
                });
                ui.add_space(8.0);

                // Row 1: Set Session Password
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(150.0, 38.0),
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
                            Vec2::new(125.0, 38.0),
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
                                .margin(Margin::symmetric(12.0, 9.0))
                                .desired_width(ui.available_width()),
                        );
                    });
                });

                ui.add_space(8.0);

                // Row 2: Confirm Password
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(150.0, 38.0),
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
                            Vec2::new(125.0, 38.0),
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
                                .margin(Margin::symmetric(12.0, 9.0))
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.handle_start_lockdown();
                        }
                    });
                });
            });

            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                if p
                    .centered_button(
                        ui,
                        Vec2::new(460.0, 44.0),
                        "🔒 Set Session Password & Activate Lockdown",
                        16.0,
                        p.gold_core,
                        p.text_on_gold,
                        Stroke::new(1.5_f32, p.cream_core),
                        10.0,
                    )
                    .clicked()
                {
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
                p.draw_logo_image(ui, &self.logo_texture, Vec2::splat(40.0));
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
            ui.add_space(8.0);

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
                    Vec2::new(145.0, 38.0),
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
                    let show_label = if self.show_unlock_password {
                        "🙈 Hide"
                    } else {
                        "👁 Show"
                    };
                    if p
                        .centered_button(
                            ui,
                            Vec2::new(90.0, 38.0),
                            show_label,
                            13.0,
                            p.bg_card_subtle,
                            p.text_primary,
                            Stroke::new(1.0_f32, p.border_subtle),
                            8.0,
                        )
                        .clicked()
                    {
                        self.show_unlock_password = !self.show_unlock_password;
                    }

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.unlock_password_input)
                            .password(!self.show_unlock_password)
                            .hint_text("Enter session password")
                            .font(FontId::proportional(15.0))
                            .margin(Margin::symmetric(12.0, 9.0))
                            .desired_width(ui.available_width()),
                    );
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.handle_unlock_session();
                    }
                });
            });

            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                if p
                    .centered_button(
                        ui,
                        Vec2::new(380.0, 44.0),
                        "🔓 Verify Password & Unlock",
                        15.5,
                        p.gold_core,
                        p.text_on_gold,
                        Stroke::new(1.5_f32, p.cream_core),
                        10.0,
                    )
                    .clicked()
                {
                    self.handle_unlock_session();
                }
            });
        });
    }
}
