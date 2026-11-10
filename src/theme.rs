use eframe::egui::{self, Color32, IconData, Margin, Pos2, Rounding, Shape, Stroke, Style, Vec2, Visuals};
use std::sync::Arc;

/// "Neon Sunshine Bliss" Color Palette from Coolors:
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
    // Semantic Status Badges (harmonized with #0D3B66 / #FAF0CA / #F4D35E)
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
            bg_header: Color32::from_rgb(9, 43, 76),           // #092B4C
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
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.inactive.rounding = Rounding::same(7.0);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = self.accent_hover;
        visuals.widgets.hovered.weak_bg_fill = self.accent_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.navy_core);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, self.text_on_gold);
        visuals.widgets.hovered.rounding = Rounding::same(7.0);

        // Active / clicked widgets
        visuals.widgets.active.bg_fill = self.accent_primary;
        visuals.widgets.active.weak_bg_fill = self.accent_primary;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, self.cream_core);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, self.text_on_gold);
        visuals.widgets.active.rounding = Rounding::same(7.0);

        // Selection highlight
        visuals.selection.bg_fill = self.gold_core;
        visuals.selection.stroke = Stroke::new(1.0_f32, self.navy_core);

        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(13.0, 7.0);
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

    /// Returns a clean sub-section card Frame for nested groups (such as on Page 3).
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

    /// Draws the custom vector Contest Lockdown Logo Emblem (`< Keyhole >` inside a double-bordered Shield)
    /// using the exact Neon Sunshine Bliss palette (`#0D3B66`, `#FAF0CA`, `#F4D35E`).
    pub fn draw_logo(&self, ui: &mut egui::Ui, size: f32) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        let c = rect.center();
        let r = size * 0.46;

        // 1. Deep Navy circular badge backdrop (#0D3B66) with thin gold ring
        painter.circle(
            c,
            r + 1.5,
            self.navy_core,
            Stroke::new(1.5_f32, self.gold_core.gamma_multiply(0.65)),
        );

        // Helper closure to compute a 6-point geometric shield polygon scaled by `s`
        let shield_pts = |s: f32| -> Vec<Pos2> {
            vec![
                Pos2::new(c.x, c.y - r * 0.84 * s),                // Top crest
                Pos2::new(c.x + r * 0.72 * s, c.y - r * 0.50 * s), // Top-right shoulder
                Pos2::new(c.x + r * 0.62 * s, c.y + r * 0.22 * s), // Mid-right flank
                Pos2::new(c.x, c.y + r * 0.86 * s),                // Bottom tip
                Pos2::new(c.x - r * 0.62 * s, c.y + r * 0.22 * s), // Mid-left flank
                Pos2::new(c.x - r * 0.72 * s, c.y - r * 0.50 * s), // Top-left shoulder
            ]
        };

        // 2. Outer Shield in Sunshine Gold (#F4D35E)
        let outer = shield_pts(1.0);
        painter.add(Shape::convex_polygon(
            outer.clone(),
            Color32::from_rgb(13, 59, 102),
            Stroke::new((size * 0.055).max(1.8), self.gold_core),
        ));

        // 3. Inner Shield in Warm Cream (#FAF0CA)
        let inner = shield_pts(0.76);
        painter.add(Shape::closed_line(
            inner,
            Stroke::new((size * 0.038).max(1.2), self.cream_core),
        ));

        // 4. Center Keyhole in Sunshine Gold (#F4D35E)
        let kh_center = Pos2::new(c.x, c.y - r * 0.12);
        painter.circle_filled(kh_center, r * 0.16, self.gold_core);
        let keyhole_base = vec![
            Pos2::new(c.x - r * 0.08, c.y - r * 0.02),
            Pos2::new(c.x + r * 0.08, c.y - r * 0.02),
            Pos2::new(c.x + r * 0.14, c.y + r * 0.34),
            Pos2::new(c.x - r * 0.14, c.y + r * 0.34),
        ];
        painter.add(Shape::convex_polygon(
            keyhole_base,
            self.gold_core,
            Stroke::NONE,
        ));

        // 5. Left & Right Code Brackets `<` and `>` in Warm Cream (#FAF0CA)
        let bracket_stroke = Stroke::new((size * 0.045).max(1.4), self.cream_core);
        let left_bracket = vec![
            Pos2::new(c.x - r * 0.24, c.y - r * 0.18),
            Pos2::new(c.x - r * 0.44, c.y + r * 0.04),
            Pos2::new(c.x - r * 0.24, c.y + r * 0.26),
        ];
        let right_bracket = vec![
            Pos2::new(c.x + r * 0.24, c.y - r * 0.18),
            Pos2::new(c.x + r * 0.44, c.y + r * 0.04),
            Pos2::new(c.x + r * 0.24, c.y + r * 0.26),
        ];
        painter.add(Shape::line(left_bracket, bracket_stroke));
        painter.add(Shape::line(right_bracket, bracket_stroke));
    }
}

/// Generates a 64x64 RGBA native OS window & dock icon matching the Shield + Lock logo
/// in `#0D3B66` Deep Navy, `#FAF0CA` Cream, and `#F4D35E` Sunshine Gold.
pub fn build_window_icon() -> Arc<IconData> {
    let w = 64_usize;
    let h = 64_usize;
    let mut rgba = vec![0_u8; w * h * 4];

    let navy = [13_u8, 59, 102, 255];   // #0D3B66
    let cream = [250_u8, 240, 202, 255]; // #FAF0CA
    let gold = [244_u8, 211, 94, 255];   // #F4D35E

    for y in 0..h {
        for x in 0..w {
            let fx = (x as f32 - 31.5) / 31.5;
            let fy = (y as f32 - 31.5) / 31.5;
            let dist = (fx * fx + fy * fy).sqrt();
            let idx = (y * w + x) * 4;

            if dist <= 0.95 {
                // Base Navy circle
                let mut color = navy;

                // Shield boundary equation
                let ax = fx.abs();
                let shield_dist = if fy < -0.1 {
                    ax * 1.15 + (fy + 0.55).abs() * 0.45
                } else {
                    ax * 1.25 + (fy + 0.1) * 0.85
                };

                if (0.64..=0.74).contains(&shield_dist) && fy > -0.75 && fy < 0.78 {
                    color = gold;
                } else if (0.50..=0.56).contains(&shield_dist) && fy > -0.62 && fy < 0.64 {
                    color = cream;
                } else {
                    // Keyhole in the center
                    let kh_dist = (fx * fx + (fy + 0.10) * (fy + 0.10)).sqrt();
                    let in_keyhole_top = kh_dist <= 0.15;
                    let in_keyhole_slot = ax <= (0.07 + (fy + 0.05) * 0.18) && (-0.05..=0.34).contains(&fy);
                    if in_keyhole_top || in_keyhole_slot {
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
