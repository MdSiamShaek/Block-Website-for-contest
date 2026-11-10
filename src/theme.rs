use eframe::egui::{self, Color32, Margin, Rounding, Stroke, Style, Visuals};

/// "Neon Sunshine Bliss" Color Palette from Coolors:
/// - **Deep Yale Navy (`#0D3B66` / `rgb(13, 59, 102)`):** Primary structural base, deep surfaces, and high-contrast ink.
/// - **Warm Lemon Cream (`#FAF0CA` / `rgb(250, 240, 202)`):** Crisp warm foreground typography (in Navy Mode) or light canvas (in Cream Mode).
/// - **Sunshine Gold (`#F4D35E` / `rgb(244, 211, 94)`):** Primary interactive accent, active step indicator, highlights, and call-to-action buttons.
#[derive(Clone, Copy)]
pub struct AppPalette {
    pub is_dark: bool,
    // Core Palette Constants
    pub navy_core: Color32,     // #0D3B66
    pub cream_core: Color32,    // #FAF0CA
    pub gold_core: Color32,     // #F4D35E
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
        let navy_core = Color32::from_rgb(13, 59, 102);   // #0D3B66
        let cream_core = Color32::from_rgb(250, 240, 202); // #FAF0CA
        let gold_core = Color32::from_rgb(244, 211, 94);   // #F4D35E

        Self {
            is_dark: true,
            navy_core,
            cream_core,
            gold_core,
            bg_canvas: navy_core,                              // #0D3B66 (Exact palette navy)
            bg_header: Color32::from_rgb(9, 43, 76),           // #092B4C (Deepened #0D3B66 header)
            bg_card: Color32::from_rgb(18, 73, 124),           // #12497C (Elevated #0D3B66 card surface)
            bg_card_subtle: Color32::from_rgb(24, 88, 148),    // #185894 (Interactive widget surface)
            bg_input: Color32::from_rgb(8, 38, 68),            // #082644 (Recessed input field)
            border_subtle: Color32::from_rgb(56, 118, 172),    // #3876AC (Harmonious navy border)
            text_primary: cream_core,                          // #FAF0CA (Exact palette cream)
            text_secondary: Color32::from_rgb(224, 214, 178),  // #E0D6B2 (Softened #FAF0CA)
            text_muted: Color32::from_rgb(172, 188, 204),      // #ACBCCC
            text_on_gold: Color32::from_rgb(9, 43, 76),        // #092B4C (High-contrast navy ink on #F4D35E gold buttons)
            accent_primary: gold_core,                         // #F4D35E (Exact palette gold)
            accent_hover: Color32::from_rgb(249, 224, 130),    // #F9E082 (Brightened #F4D35E on hover)
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
        let navy_core = Color32::from_rgb(13, 59, 102);   // #0D3B66
        let cream_core = Color32::from_rgb(250, 240, 202); // #FAF0CA
        let gold_core = Color32::from_rgb(244, 211, 94);   // #F4D35E

        Self {
            is_dark: false,
            navy_core,
            cream_core,
            gold_core,
            bg_canvas: cream_core,                             // #FAF0CA (Exact palette cream)
            bg_header: navy_core,                              // #0D3B66 (Exact palette navy header)
            bg_card: Color32::from_rgb(253, 248, 225),         // #FDF8E1 (Lightened #FAF0CA card)
            bg_card_subtle: gold_core,                         // #F4D35E (Exact palette gold subtle fill)
            bg_input: Color32::from_rgb(255, 252, 240),        // #FFFCF0
            border_subtle: Color32::from_rgb(185, 168, 115),   // #B9A873
            text_primary: navy_core,                           // #0D3B66 (Exact palette navy ink)
            text_secondary: Color32::from_rgb(32, 82, 130),    // #205282
            text_muted: Color32::from_rgb(78, 108, 138),       // #4E6C8A
            text_on_gold: navy_core,                           // #0D3B66
            accent_primary: gold_core,                         // #F4D35E (Exact palette gold)
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

        // Non-interactive widgets (cards, labels, frames)
        visuals.widgets.noninteractive.bg_fill = self.bg_card;
        visuals.widgets.noninteractive.weak_bg_fill = self.bg_card_subtle;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

        // Inactive interactive widgets (buttons, inputs, combo boxes)
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

        // Hovered widgets (#F4D35E Sunshine Gold highlight with #0D3B66 Navy text)
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
}
