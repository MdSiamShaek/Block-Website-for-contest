use eframe::egui::{
    self, Align2, Color32, ColorImage, FontId, IconData, Margin, Pos2, RichText, Rounding, Sense,
    Stroke, Style, TextureHandle, TextureOptions, Vec2, Visuals,
};
use std::sync::Arc;

/// Embedded bytes of `/Users/siam/Desktop/Block-Website-for-contest/ProctorX Logo.png`
pub const PROCTORX_LOGO_PNG: &[u8] = include_bytes!("../ProctorX Logo.png");

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
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        ctx.set_style(style);
    }

    /// Returns a styled container card Frame with balanced padding.
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
            .inner_margin(Margin::symmetric(22.0, 18.0))
    }

    /// Returns a clean sub-section card Frame with balanced padding.
    pub fn sub_card_frame(&self) -> egui::Frame {
        egui::Frame::none()
            .fill(self.bg_input)
            .stroke(Stroke::new(1.0_f32, self.border_subtle))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(18.0, 14.0))
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
            .inner_margin(Margin::symmetric(12.0, 5.0))
            .show(ui, |ui| {
                ui.label(RichText::new(text).color(fg).strong().size(12.0));
            });
    }

    /// Renders a custom button of exact `[width, height]` with **100% horizontally & vertically centered text**.
    #[allow(clippy::too_many_arguments)]
    pub fn centered_button(
        &self,
        ui: &mut egui::Ui,
        size: Vec2,
        label: &str,
        font_size: f32,
        fill: Color32,
        text_color: Color32,
        stroke: Stroke,
        rounding: f32,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let is_hovered = response.hovered() && ui.is_enabled();
            let bg = if !ui.is_enabled() {
                fill.gamma_multiply(0.45)
            } else if is_hovered {
                self.accent_hover
            } else {
                fill
            };
            let fg = if !ui.is_enabled() {
                text_color.gamma_multiply(0.5)
            } else if is_hovered {
                self.text_on_gold
            } else {
                text_color
            };
            let border = if is_hovered && ui.is_enabled() {
                Stroke::new(1.6_f32, self.cream_core)
            } else {
                stroke
            };

            let painter = ui.painter();
            painter.rect(rect, Rounding::same(rounding), bg, border);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(font_size),
                fg,
            );
        }
        response
    }

    /// Renders a two-line centered toggle card (used on Tab 3 for Reference & Judge sites).
    pub fn centered_two_line_card(
        &self,
        ui: &mut egui::Ui,
        size: Vec2,
        line1: &str,
        line2: &str,
        is_active: bool,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let is_hovered = response.hovered();
            let bg = if is_active {
                self.gold_core
            } else if is_hovered {
                self.bg_card_subtle
            } else {
                self.bg_card
            };
            let fg1 = if is_active {
                self.text_on_gold
            } else {
                self.text_primary
            };
            let fg2 = if is_active {
                self.text_on_gold.gamma_multiply(0.85)
            } else {
                self.text_secondary
            };
            let border = Stroke::new(
                if is_active || is_hovered {
                    1.5_f32
                } else {
                    1.0_f32
                },
                if is_active {
                    self.cream_core
                } else {
                    self.border_subtle
                },
            );

            let painter = ui.painter();
            painter.rect(rect, Rounding::same(8.0), bg, border);
            painter.text(
                Pos2::new(rect.center().x, rect.center().y - 7.5),
                Align2::CENTER_CENTER,
                line1,
                FontId::proportional(13.0),
                fg1,
            );
            painter.text(
                Pos2::new(rect.center().x, rect.center().y + 8.5),
                Align2::CENTER_CENTER,
                line2,
                FontId::proportional(11.5),
                fg2,
            );
        }
        response
    }

    /// Draws the loaded `ProctorX Logo.png` texture at the specified `size`.
    pub fn draw_logo_image(&self, ui: &mut egui::Ui, logo_tex: &TextureHandle, size: Vec2) {
        let img = egui::Image::from_texture(logo_tex)
            .fit_to_exact_size(size)
            .rounding(Rounding::same(6.0));
        ui.add(img);
    }

    /// Creates a `LayoutJob` for the two-tone **ProctorX** wordmark (`Proctor` in `#FAF0CA` + `X` in `#F4D35E`).
    pub fn proctorx_wordmark_job(&self, font_size: f32) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "Proctor",
            0.0,
            egui::TextFormat {
                font_id: FontId::proportional(font_size),
                color: self.cream_core,
                ..Default::default()
            },
        );
        job.append(
            "X",
            0.0,
            egui::TextFormat {
                font_id: FontId::proportional(font_size),
                color: self.gold_core,
                ..Default::default()
            },
        );
        job
    }
}

/// Decodes `ProctorX Logo.png` and crops to the rounded-square emblem (`x: 30%..70%, y: 18%..58%`)
/// so it renders at full clarity in the header bar without duplicate tiny text underneath.
pub fn load_logo_texture(ctx: &egui::Context) -> TextureHandle {
    let img = image::load_from_memory(PROCTORX_LOGO_PNG)
        .expect("Failed to decode ProctorX Logo.png");
    let (w, h) = (img.width(), img.height());
    // Crop tightly around the rounded-square emblem in ProctorX Logo.png
    let crop_x = (w as f32 * 0.305) as u32;
    let crop_y = (h as f32 * 0.185) as u32;
    let crop_w = (w as f32 * 0.39) as u32;
    let crop_h = (h as f32 * 0.39) as u32;
    let cropped = img.crop_imm(crop_x, crop_y, crop_w, crop_h).into_rgba8();

    let (cw, ch) = cropped.dimensions();
    let color_image =
        ColorImage::from_rgba_unmultiplied([cw as usize, ch as usize], cropped.as_raw());
    ctx.load_texture("proctorx_logo", color_image, TextureOptions::LINEAR)
}

/// Decodes `ProctorX Logo.png` into a native OS window & dock icon (`IconData`).
pub fn build_window_icon() -> Arc<IconData> {
    if let Ok(img) = image::load_from_memory(PROCTORX_LOGO_PNG) {
        let rgba_img = img.into_rgba8();
        let (w, h) = rgba_img.dimensions();
        return Arc::new(IconData {
            rgba: rgba_img.into_raw(),
            width: w,
            height: h,
        });
    }

    Arc::new(IconData {
        rgba: vec![0; 64 * 64 * 4],
        width: 64,
        height: 64,
    })
}
