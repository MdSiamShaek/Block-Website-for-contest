use eframe::egui::{
    self, Color32, IconData, Margin, Pos2, RichText, Rounding, Shape, Stroke, Style, Vec2, Visuals,
};
use std::sync::Arc;

/// "Neon Sunshine Bliss" Color Palette (`#0D3B66`, `#FAF0CA`, `#F4D35E`) for **ProctorX**:
/// - **Deep Yale Navy (`#0D3B66` / `rgb(13, 59, 102)`):** Primary structural base, deep surfaces, and high-contrast ink.
/// - **Warm Lemon Cream (`#FAF0CA` / `rgb(250, 240, 202)`):** Crisp warm foreground typography (in Navy Mode) or light canvas (in Cream Mode).
/// - **Sunshine Gold (`#F4D35E` / `rgb(244, 211, 94)`):** Primary interactive accent, active step indicator, highlights, and call-to-action buttons.
#[derive(Clone, Copy)]
pub struct AppPalette {
    pub is_dark: bool,
    // Core Palette Constants
    pub navy_core: Color32,  // #0D3B66
    pub cream_core: Color32, // #FAF0CA
    pub gold_core: Color32,  // #F4D35E
    // Surface Hierarchy
    pub bg_canvas: Color32,
    pub bg_header: Color32,
    pub bg_card: Color32,
    pub bg_card_subtle: Color32,
    pub bg_input: Color32,
    pub border_subtle: Color32,
    // Typography
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_on_gold: Color32,
    // Accents
    pub accent_primary: Color32,
    pub accent_hover: Color32,
    // Semantic Status Badges
    pub success_fg: Color32,
    pub success_bg: Color32,
    pub danger_fg: Color32,
    pub danger_bg: Color32,
    pub warning_fg: Color32,
    pub warning_bg: Color32,
}

impl AppPalette {
    /// Navy Mode (Default): `#0D3B66` Deep Navy canvas, `#FAF0CA` Warm Cream text, `#F4D35E` Sunshine Gold accents.
    pub fn dark() -> Self {
        let navy_core = Color32::from_rgb(13, 59, 102); // #0D3B66
        let cream_core = Color32::from_rgb(250, 240, 202); // #FAF0CA
        let gold_core = Color32::from_rgb(244, 211, 94); // #F4D35E

        Self {
            is_dark: true,
            navy_core,
            cream_core,
            gold_core,
            bg_canvas: navy_core,                              // #0D3B66
            bg_header: Color32::from_rgb(11, 50, 88),          // #0B3258
            bg_card: Color32::from_rgb(18, 73, 124),           // #12497C
            bg_card_subtle: Color32::from_rgb(24, 88, 148),    // #185894
            bg_input: Color32::from_rgb(8, 38, 68),            // #082644
            border_subtle: Color32::from_rgb(56, 118, 172),    // #3876AC
            text_primary: cream_core,                          // #FAF0CA
            text_secondary: Color32::from_rgb(224, 214, 178),  // #E0D6B2
            text_muted: Color32::from_rgb(172, 188, 204),      // #ACBCCC
            text_on_gold: Color32::from_rgb(9, 43, 76),        // #092B4C
            accent_primary: gold_core,                         // #F4D35E
            accent_hover: Color32::from_rgb(249, 224, 130),    // #F9E082
            success_fg: cream_core,                            // #FAF0CA
            success_bg: Color32::from_rgb(22, 101, 78),        // #16654E
            danger_fg: Color32::from_rgb(255, 228, 225),       // #FFE4E1
            danger_bg: Color32::from_rgb(153, 27, 48),         // #991B30
            warning_fg: Color32::from_rgb(13, 59, 102),        // #0D3B66
            warning_bg: gold_core,                             // #F4D35E
        }
    }

    /// Cream Mode: `#FAF0CA` Warm Cream canvas, `#0D3B66` Deep Navy text & headers, `#F4D35E` Sunshine Gold highlights.
    pub fn light() -> Self {
        let navy_core = Color32::from_rgb(13, 59, 102); // #0D3B66
        let cream_core = Color32::from_rgb(250, 240, 202); // #FAF0CA
        let gold_core = Color32::from_rgb(244, 211, 94); // #F4D35E

        Self {
            is_dark: false,
            navy_core,
            cream_core,
            gold_core,
            bg_canvas: cream_core,                             // #FAF0CA
            bg_header: navy_core,                              // #0D3B66
            bg_card: Color32::from_rgb(253, 248, 225),         // #FDF8E1
            bg_card_subtle: gold_core,                         // #F4D35E
            bg_input: Color32::from_rgb(255, 252, 240),        // #FFFCF0
            border_subtle: Color32::from_rgb(185, 168, 115),   // #B9A873
            text_primary: navy_core,                           // #0D3B66
            text_secondary: Color32::from_rgb(32, 82, 130),    // #205282
            text_muted: Color32::from_rgb(78, 108, 138),       // #4E6C8A
            text_on_gold: navy_core,                           // #0D3B66
            accent_primary: gold_core,                         // #F4D35E
            accent_hover: Color32::from_rgb(235, 198, 68),     // #EBC644
            success_fg: Color32::from_rgb(13, 59, 102),        // #0D3B66
            success_bg: Color32::from_rgb(187, 235, 205),      // #BBEBCD
            danger_fg: Color32::from_rgb(250, 240, 202),       // #FAF0CA
            danger_bg: Color32::from_rgb(168, 32, 48),         // #A82030
            warning_fg: navy_core,                             // #0D3B66
            warning_bg: gold_core,                             // #F4D35E
        }
    }

    /// Applies the Neon Sunshine Bliss palette (`#0D3B66`, `#FAF0CA`, `#F4D35E`) to `egui::Context`.
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style: Style = (*ctx.style()).clone();
        let mut visuals = if self.is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.panel_fill = self.bg_canvas;
        visuals.window_fill = self.bg_card;
        visuals.extreme_bg_color = self.bg_input;
        visuals.faint_bg_color = self.bg_card_subtle;
        visuals.override_text_color = Some(self.text_primary);

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = self.bg_card;
        visuals.widgets.noninteractive.weak_bg_fill = self.bg_card_subtle;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

        // Inactive interactive widgets
        let inactive_bg = if self.is_dark {
            self.bg_card_subtle
        } else {
            Color32::from_rgb(247, 232, 170)
        };
        visuals.widgets.inactive.bg_fill = inactive_bg;
        visuals.widgets.inactive.weak_bg_fill = inactive_bg;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.2_f32, self.border_subtle);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.inactive.rounding = Rounding::same(8.0);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = self.accent_hover;
        visuals.widgets.hovered.weak_bg_fill = self.accent_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.navy_core);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, self.text_on_gold);
        visuals.widgets.hovered.rounding = Rounding::same(8.0);

        // Active / clicked widgets
        visuals.widgets.active.bg_fill = self.accent_primary;
        visuals.widgets.active.weak_bg_fill = self.accent_primary;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, self.cream_core);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, self.text_on_gold);
        visuals.widgets.active.rounding = Rounding::same(8.0);

        // Selection highlight
        visuals.selection.bg_fill = self.gold_core;
        visuals.selection.stroke = Stroke::new(1.0_f32, self.navy_core);

        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        ctx.set_style(style);
    }

    /// Returns a styled container card Frame.
    pub fn card_frame(&self) -> egui::Frame {
        let stroke_color = if self.is_dark {
            self.gold_core.gamma_multiply(0.45)
        } else {
            self.navy_core.gamma_multiply(0.35)
        };
        egui::Frame::none()
            .fill(self.bg_card)
            .stroke(Stroke::new(1.2_f32, stroke_color))
            .rounding(Rounding::same(12.0))
            .inner_margin(Margin::symmetric(20.0, 18.0))
    }

    /// Returns a clean sub-section card Frame for nested groups.
    pub fn sub_card_frame(&self) -> egui::Frame {
        egui::Frame::none()
            .fill(self.bg_input)
            .stroke(Stroke::new(1.0_f32, self.border_subtle))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(16.0, 14.0))
    }

    /// Returns a highlighted banner Frame for semantic state callouts.
    pub fn banner_frame(&self, bg: Color32, border: Color32) -> egui::Frame {
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.5_f32, border))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(18.0, 14.0))
    }

    /// Renders a rounded status pill badge.
    pub fn badge(&self, ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0_f32, fg.gamma_multiply(0.5)))
            .rounding(Rounding::same(12.0))
            .inner_margin(Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(text).color(fg).strong().size(12.0));
            });
    }

    /// Draws the rounded-square ProctorX Shield + Checkmark Icon matching the user's reference screenshot:
    /// - Rounded square border in `#F4D35E` Sunshine Gold on `#0D3B66` Deep Navy fill
    /// - Inner geometric shield in `#F4D35E` Sunshine Gold with a crisp checkmark inside
    pub fn draw_proctorx_shield_badge(&self, ui: &mut egui::Ui, size: f32) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        // 1. Rounded square container (#0D3B66 fill, #F4D35E border)
        let box_rounding = Rounding::same((size * 0.25).max(8.0));
        painter.rect(
            rect.shrink(1.0),
            box_rounding,
            self.navy_core,
            Stroke::new((size * 0.042).max(1.6), self.gold_core),
        );

        let c = rect.center();
        let r = size * 0.27;

        // 2. Shield outline inside the rounded square
        let shield_pts = vec![
            Pos2::new(c.x, c.y - r * 0.95),                // Top center crest
            Pos2::new(c.x + r * 0.78, c.y - r * 0.58),     // Top-right corner
            Pos2::new(c.x + r * 0.72, c.y + r * 0.18),     // Mid-right flank
            Pos2::new(c.x, c.y + r * 0.95),                // Bottom tip
            Pos2::new(c.x - r * 0.72, c.y + r * 0.18),     // Mid-left flank
            Pos2::new(c.x - r * 0.78, c.y - r * 0.58),     // Top-left corner
        ];
        painter.add(Shape::closed_line(
            shield_pts,
            Stroke::new((size * 0.048).max(1.8), self.gold_core),
        ));

        // 3. Crisp Checkmark inside the shield
        let check_pts = vec![
            Pos2::new(c.x - r * 0.32, c.y + r * 0.02),
            Pos2::new(c.x - r * 0.06, c.y + r * 0.28),
            Pos2::new(c.x + r * 0.36, c.y - r * 0.22),
        ];
        painter.add(Shape::line(
            check_pts,
            Stroke::new((size * 0.048).max(1.8), self.gold_core),
        ));
    }

    /// Renders the inline **ProctorX** brand title (`Proctor` in `#FAF0CA` Cream + `X` in `#F4D35E` Gold).
    pub fn render_proctorx_wordmark(&self, ui: &mut egui::Ui, font_size: f32) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label(
                RichText::new("Proctor")
                    .size(font_size)
                    .strong()
                    .color(self.cream_core),
            );
            ui.label(
                RichText::new("X")
                    .size(font_size)
                    .strong()
                    .color(self.gold_core),
            );
        });
    }

    /// Renders the centered **ProctorX** hero banner matching the user's screenshot:
    /// - Rounded square Shield+Checkmark icon
    /// - `ProctorX` wordmark
    /// - `SECURE. MONITOR. CONTROL.` motto in `#F4D35E` Gold
    /// - Subtle divider line + `Cross-Platform Contest Lockdown & Proctoring Suite` subtitle
    pub fn render_proctorx_hero_banner(&self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(self.navy_core)
            .stroke(Stroke::new(1.2_f32, self.gold_core.gamma_multiply(0.5)))
            .rounding(Rounding::same(14.0))
            .inner_margin(Margin::symmetric(24.0, 14.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    self.draw_proctorx_shield_badge(ui, 52.0);
                    ui.add_space(4.0);

                    // Centered "ProctorX" title
                    let mut job = egui::text::LayoutJob::default();
                    job.append(
                        "Proctor",
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::proportional(24.0),
                            color: self.cream_core,
                            ..Default::default()
                        },
                    );
                    job.append(
                        "X",
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::proportional(24.0),
                            color: self.gold_core,
                            ..Default::default()
                        },
                    );
                    ui.label(job);

                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("SECURE. MONITOR. CONTROL.")
                            .size(13.0)
                            .strong()
                            .color(self.gold_core),
                    );
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Cross-Platform Contest Lockdown & Proctoring Suite")
                            .size(12.5)
                            .color(self.cream_core),
                    );
                });
            });
    }
}

/// Generates a 64x64 RGBA native OS window & dock icon matching the ProctorX Shield+Checkmark logo
/// in `#0D3B66` Deep Navy, `#FAF0CA` Cream, and `#F4D35E` Sunshine Gold.
pub fn build_window_icon() -> Arc<IconData> {
    let w = 64_usize;
    let h = 64_usize;
    let mut rgba = vec![0_u8; w * h * 4];

    let navy = [13_u8, 59, 102, 255];   // #0D3B66
    let gold = [244_u8, 211, 94, 255];  // #F4D35E

    for y in 0..h {
        for x in 0..w {
            let fx = (x as f32 - 31.5) / 31.5;
            let fy = (y as f32 - 31.5) / 31.5;
            let idx = (y * w + x) * 4;

            // Rounded square SDF
            let qx = fx.abs() - 0.68;
            let qy = fy.abs() - 0.68;
            let r_dist = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - 0.22;

            if r_dist <= 0.0 {
                let mut color = navy;
                if r_dist >= -0.065 {
                    color = gold;
                } else {
                    // Inner shield outline
                    let ax = fx.abs();
                    let shield_d = if fy < -0.05 {
                        ax * 1.45 + (fy + 0.42).abs() * 0.55
                    } else {
                        ax * 1.55 + (fy + 0.05) * 1.05
                    };
                    if (0.42..=0.52).contains(&shield_d) && (-0.52..=0.55).contains(&fy) {
                        color = gold;
                    }
                }
                rgba[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    Arc::new(IconData {
        rgba,
        width: w as u32,
        height: h as u32,
    })
}
