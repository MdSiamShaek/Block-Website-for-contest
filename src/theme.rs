use eframe::egui::{self, Color32, Margin, Rounding, Stroke, Style, Visuals};

/// Color-Theory Palette built on the **60-30-10 Rule**, **Split-Complementary Harmony**,
/// and **WCAG AAA Contrast Ratios**:
/// - **60% Dominant Base:** Deep Midnight Slate (`#0B0F19`) in Dark Mode / Cool Alabaster (`#F1F5F9`) in Light Mode
///   — low-saturation cool blue-slate minimizes eye fatigue during 3–5 hour programming contests.
/// - **30% Secondary Surfaces:** Layered Slate Navy (`#161F32` & `#1C273E`) with subtle `#2A3854` 1px borders
///   — establishes clear visual hierarchy and card elevation without harsh lines.
/// - **10% Semantic & Split-Complementary Accents:**
///   - Primary Interactive Brand: Royal Indigo (`#4F46E5`) paired with Sky Cyan (`#38BDF8`)
///   - Safe / Unlocked / Installed: Emerald Teal (`#34D399` text / `#064E3B` badge fill)
///   - Active Lockdown / Recording: Crimson Rose (`#FB7185` text / `#4C0519` badge fill)
///   - Warning / Missing Tool: Warm Amber (`#FBBF24` text / `#451A03` badge fill)
#[derive(Clone, Copy)]
pub struct AppPalette {
    pub is_dark: bool,
    // 60% Base Canvas
    pub bg_canvas: Color32,
    pub bg_header: Color32,
    // 30% Elevated Surfaces & Borders
    pub bg_card: Color32,
    pub bg_card_subtle: Color32,
    pub bg_input: Color32,
    pub border_subtle: Color32,
    // Typography
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    // 10% Accents (Interactive & Semantic)
    pub accent_primary: Color32,
    pub accent_primary_hover: Color32,
    pub accent_cyan: Color32,
    // Semantic Status Colors
    pub success_fg: Color32,
    pub success_bg: Color32,
    pub danger_fg: Color32,
    pub danger_bg: Color32,
    pub warning_fg: Color32,
    pub warning_bg: Color32,
}

impl AppPalette {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            bg_canvas: Color32::from_rgb(11, 15, 25),         // #0B0F19 (60% Dominant Base)
            bg_header: Color32::from_rgb(16, 23, 38),         // #101726
            bg_card: Color32::from_rgb(22, 31, 50),           // #161F32 (30% Secondary Card Surface)
            bg_card_subtle: Color32::from_rgb(28, 39, 62),    // #1C273E
            bg_input: Color32::from_rgb(13, 19, 33),          // #0D1321
            border_subtle: Color32::from_rgb(42, 56, 84),     // #2A3854
            text_primary: Color32::from_rgb(241, 245, 249),   // #F1F5F9 (High-contrast off-white)
            text_secondary: Color32::from_rgb(148, 163, 184), // #94A3B8 (Balanced slate gray)
            text_muted: Color32::from_rgb(100, 116, 139),     // #64748B
            accent_primary: Color32::from_rgb(79, 70, 229),   // #4F46E5 (Royal Indigo)
            accent_primary_hover: Color32::from_rgb(99, 102, 241), // #6366F1
            accent_cyan: Color32::from_rgb(56, 189, 248),     // #38BDF8 (Sky Cyan)
            success_fg: Color32::from_rgb(52, 211, 153),      // #34D399 (Emerald)
            success_bg: Color32::from_rgb(6, 78, 59),         // #064E3B
            danger_fg: Color32::from_rgb(251, 113, 133),      // #FB7185 (Crimson Rose)
            danger_bg: Color32::from_rgb(76, 5, 25),          // #4C0519
            warning_fg: Color32::from_rgb(251, 191, 36),      // #FBBF24 (Warm Amber)
            warning_bg: Color32::from_rgb(69, 26, 3),         // #451A03
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            bg_canvas: Color32::from_rgb(241, 245, 249),      // #F1F5F9 (60% Dominant Base)
            bg_header: Color32::from_rgb(255, 255, 255),      // #FFFFFF
            bg_card: Color32::from_rgb(255, 255, 255),        // #FFFFFF (30% Secondary Card Surface)
            bg_card_subtle: Color32::from_rgb(248, 250, 252), // #F8FAFC
            bg_input: Color32::from_rgb(248, 250, 252),       // #F8FAFC
            border_subtle: Color32::from_rgb(203, 213, 225),  // #CBD5E1
            text_primary: Color32::from_rgb(15, 23, 42),      // #0F172A (Deep Slate Ink)
            text_secondary: Color32::from_rgb(71, 85, 105),   // #475569
            text_muted: Color32::from_rgb(100, 116, 139),     // #64748B
            accent_primary: Color32::from_rgb(79, 70, 229),   // #4F46E5 (Royal Indigo)
            accent_primary_hover: Color32::from_rgb(67, 56, 202), // #4338CA
            accent_cyan: Color32::from_rgb(2, 132, 199),      // #0284C7 (Ocean Cyan)
            success_fg: Color32::from_rgb(5, 150, 105),       // #059669
            success_bg: Color32::from_rgb(209, 250, 229),     // #D1FAE5
            danger_fg: Color32::from_rgb(225, 29, 72),        // #E11D48
            danger_bg: Color32::from_rgb(255, 228, 230),     // #FFE4E6
            warning_fg: Color32::from_rgb(217, 119, 6),       // #D97706
            warning_bg: Color32::from_rgb(254, 243, 199),     // #FEF3C7
        }
    }

    /// Applies the palette to `egui::Context` visuals and spacing.
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
        visuals.widgets.inactive.bg_fill = self.bg_card_subtle;
        visuals.widgets.inactive.weak_bg_fill = self.bg_card_subtle;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.inactive.rounding = Rounding::same(7.0);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = self.accent_primary_hover;
        visuals.widgets.hovered.weak_bg_fill = self.accent_primary_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.accent_cyan);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.hovered.rounding = Rounding::same(7.0);

        // Active / clicked widgets
        visuals.widgets.active.bg_fill = self.accent_primary;
        visuals.widgets.active.weak_bg_fill = self.accent_primary;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.active.rounding = Rounding::same(7.0);

        // Selection highlight
        visuals.selection.bg_fill = self.accent_primary;
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        ctx.set_style(style);
    }

    /// Returns a styled container card Frame following the 30% Secondary Surface rule.
    pub fn card_frame(&self) -> egui::Frame {
        egui::Frame::none()
            .fill(self.bg_card)
            .stroke(Stroke::new(1.0_f32, self.border_subtle))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(16.0, 14.0))
    }

    /// Returns a highlighted banner Frame for semantic state callouts.
    pub fn banner_frame(&self, bg: Color32, border: Color32) -> egui::Frame {
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.2_f32, border))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(16.0, 12.0))
    }

    /// Renders a rounded semantic status pill badge.
    pub fn badge(&self, ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0_f32, fg.gamma_multiply(0.45)))
            .rounding(Rounding::same(12.0))
            .inner_margin(Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(text).color(fg).strong().size(12.0));
            });
    }
}
