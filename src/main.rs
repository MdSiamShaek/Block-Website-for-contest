mod state;

use eframe::egui;
use state::{verify_password, LockdownSession};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 560.0])
            .with_min_inner_size([540.0, 480.0])
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
    lock_password_input: String,
    lock_password_confirm: String,
    unlock_password_input: String,
    status_message: String,
    is_error: bool,
}

impl ContestLockdownApp {
    fn new() -> Self {
        let session = LockdownSession::load();
        let domain_input = session.allowed_domain.clone();
        Self {
            session,
            domain_input,
            lock_password_input: String::new(),
            lock_password_confirm: String::new(),
            unlock_password_input: String::new(),
            status_message: "Ready. Configure your contest settings and set a session password to begin.".to_string(),
            is_error: false,
        }
    }
}

impl eframe::App for ContestLockdownApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Contest Lockdown & Proctoring Suite");
            ui.label("Cross-platform Website Whitelisting, IDE Installer & Screen Recorder");
            ui.separator();

            if !self.session.is_locked {
                ui.label("Target Allowed Website:");
                ui.text_edit_singleline(&mut self.domain_input);
                ui.add_space(8.0);

                ui.label("Set Session Lock Password (required to unlock later):");
                ui.add(egui::TextEdit::singleline(&mut self.lock_password_input).password(true));
                ui.label("Confirm Session Password:");
                ui.add(egui::TextEdit::singleline(&mut self.lock_password_confirm).password(true));
                ui.add_space(12.0);

                if ui.button("Enable Contest Lockdown").clicked() {
                    if self.lock_password_input.is_empty() {
                        self.status_message = "Please enter a session password before locking.".to_string();
                        self.is_error = true;
                    } else if self.lock_password_input != self.lock_password_confirm {
                        self.status_message = "Passwords do not match.".to_string();
                        self.is_error = true;
                    } else {
                        self.session = LockdownSession::new_locked(
                            &self.domain_input,
                            &self.lock_password_input,
                            "Visual Studio Code",
                            None,
                        );
                        let _ = self.session.save();
                        self.lock_password_input.clear();
                        self.lock_password_confirm.clear();
                        self.is_error = false;
                        self.status_message = format!(
                            "Lockdown active for https://{}",
                            self.session.allowed_domain
                        );
                    }
                }
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 50, 50),
                    format!("LOCKDOWN ACTIVE — Only https://{} is allowed", self.session.allowed_domain),
                );
                ui.label(format!("Started at: {}", self.session.started_at));
                ui.add_space(12.0);

                ui.label("Enter Session Password to Unlock:");
                ui.add(egui::TextEdit::singleline(&mut self.unlock_password_input).password(true));

                if ui.button("Unlock & Restore Normal Access").clicked() {
                    if verify_password(&self.unlock_password_input, &self.session.password_sha256) {
                        self.session.is_locked = false;
                        let _ = LockdownSession::clear();
                        self.unlock_password_input.clear();
                        self.is_error = false;
                        self.status_message = "Session unlocked and normal access restored.".to_string();
                    } else {
                        self.is_error = true;
                        self.status_message = "Access Denied: Incorrect session password!".to_string();
                    }
                }
            }

            ui.separator();
            let color = if self.is_error {
                egui::Color32::from_rgb(220, 60, 60)
            } else {
                egui::Color32::from_rgb(60, 180, 90)
            };
            ui.colored_label(color, &self.status_message);
        });
    }
}
