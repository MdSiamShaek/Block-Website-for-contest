use eframe::egui::{
    self, Color32, IconData, Margin, Pos2, Rect, RichText, Rounding, Stroke, Style, Vec2, Visuals,
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

/// 15x14 100% mirror-symmetrical pixel-art matrix matching `ProctorX Pixel Keyhole Logo.png`:
/// - Upper curved horns (rows 0..6)
/// - Wide horizontal chevron shoulders (row 7)
/// - Clean diagonal hollow V-arms meeting at a sharp single-pixel point (rows 8..13)
pub const PROCTORX_PIXEL_EMBLEM: [&str; 14] = [
    "...##.....##...",
    "..##.......##..",
    "..##.......##..",
    "...##.....##...",
    "...##.....##...",
    "....##...##....",
    "....##...##....",
    "..####...####..",
    "...##.....##...",
    "....##...##....",
    ".....##.##.....",
    "......###......",
    "......###......",
    ".......#.......",
];

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
            bg_header: Color32::from_rgb(8, 36, 66),           // #082442
            bg_card: Color32::from_rgb(18, 73, 124),           // #12497C
            bg_card_subtle: Color32::from_rgb(24, 88, 148),    // #185894
            bg_input: Color32::from_rgb(8, 38, 68),            // #082644
            border_subtle: Color32::from_rgb(56, 118, 172),    // #3876AC
            text_primary: cream_core,                          // #FAF0CA
            text_secondary: Color32::from_rgb(224, 214, 178),  // #E0D6B2
            text_muted: Color32::from_rgb(172, 188, 204),      // #ACBCCC
            text_on_gold: Color32::from_rgb(8, 36, 66),        // #082442
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

    /// Returns a styled container card Frame with symmetrical margins.
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
            .inner_margin(Margin::symmetric(20.0, 16.0))
    }

    /// Returns a clean sub-section card Frame with symmetrical margins.
    pub fn sub_card_frame(&self) -> egui::Frame {
        egui::Frame::none()
            .fill(self.bg_input)
            .stroke(Stroke::new(1.0_f32, self.border_subtle))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(16.0, 12.0))
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
                ui.label(RichText::new(text).color(fg).strong().size(12.0));
            });
    }

    /// Draws the exact **ProctorX Pixel Keyhole Logo** from `ProctorX Pixel Keyhole Logo.png`,
    /// 100% bilaterally mirror-symmetrical and centered:
    /// - Deep Navy (`#061C38`) rounded-square interior
    /// - Bold Sunshine Gold (`#F4D35E`) rounded-square border
    /// - 15x14 palindromic pixel-art V-Keyhole emblem centered inside the square
    pub fn draw_proctorx_pixel_logo(&self, ui: &mut egui::Ui, size: f32) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        let border_w = (size * 0.068).max(2.2);
        let box_rounding = Rounding::same((size * 0.22).max(6.0));

        // 1. Rounded-square background & bold gold border
        painter.rect(
            rect.shrink(border_w * 0.5),
            box_rounding,
            Color32::from_rgb(6, 28, 56),
            Stroke::new(border_w, self.gold_core),
        );

        // 2. Render the 15x14 mirror-symmetric pixel grid centered at `rect.center()`
        let cols = 15_usize;
        let rows = PROCTORX_PIXEL_EMBLEM.len();
        let pixel_size = (size * 0.52) / (cols as f32);
        let grid_w = (cols as f32) * pixel_size;
        let grid_h = (rows as f32) * pixel_size;
        let start_x = rect.center().x - grid_w * 0.5;
        let start_y = rect.center().y - grid_h * 0.5;

        for (r_idx, row_str) in PROCTORX_PIXEL_EMBLEM.iter().enumerate() {
            for (c_idx, ch) in row_str.chars().enumerate() {
                if ch == '#' {
                    let min = Pos2::new(
                        start_x + (c_idx as f32) * pixel_size,
                        start_y + (r_idx as f32) * pixel_size,
                    );
                    let max = Pos2::new(min.x + pixel_size + 0.35, min.y + pixel_size + 0.35);
                    painter.rect_filled(
                        Rect::from_min_max(min, max),
                        Rounding::ZERO,
                        self.gold_core,
                    );
                }
            }
        }
    }

    /// Creates a `LayoutJob` for the two-tone **ProctorX** wordmark (`Proctor` in `#FAF0CA` + `X` in `#F4D35E`).
    pub fn proctorx_wordmark_job(&self, font_size: f32) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "Proctor",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(font_size),
                color: self.cream_core,
                ..Default::default()
            },
        );
        job.append(
            "X",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(font_size),
                color: self.gold_core,
                ..Default::default()
            },
        );
        job
    }
}

/// Generates a 64x64 RGBA native OS window & dock icon matching the symmetrical
/// **ProctorX Pixel Keyhole Logo** in `#0D3B66` Deep Navy and `#F4D35E` Sunshine Gold.
pub fn build_window_icon() -> Arc<IconData> {
    let w = 64_usize;
    let h = 64_usize;
    let mut rgba = vec![0_u8; w * h * 4];

    let navy = [6_u8, 28, 56, 255];    // Deep Navy backdrop
    let gold = [244_u8, 211, 94, 255]; // #F4D35E Sunshine Gold

    for y in 0..h {
        for x in 0..w {
            let fx = (x as f32 - 31.5) / 31.5;
            let fy = (y as f32 - 31.5) / 31.5;
            let idx = (y * w + x) * 4;

            let qx = fx.abs() - 0.66;
            let qy = fy.abs() - 0.66;
            let r_dist =
                (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - 0.22;

            if r_dist <= 0.0 {
                let color = if r_dist >= -0.095 { gold } else { navy };
                rgba[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    // Stamp the 15x14 symmetrical pixel emblem at 2x2 pixels per cell (30x28 centered at 17..47, 18..46)
    let scale = 2_usize;
    let rows = PROCTORX_PIXEL_EMBLEM.len();
    let start_x = (w - 15 * scale) / 2;
    let start_y = (h - rows * scale) / 2;

    for (r_idx, row_str) in PROCTORX_PIXEL_EMBLEM.iter().enumerate() {
        for (c_idx, ch) in row_str.chars().enumerate() {
            if ch == '#' {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = start_x + c_idx * scale + dx;
                        let py = start_y + r_idx * scale + dy;
                        let idx = (py * w + px) * 4;
                        rgba[idx..idx + 4].copy_from_slice(&gold);
                    }
                }
            }
        }
    }

    Arc::new(IconData {
        rgba,
        width: w as u32,
        height: h as u32,
    })
}
