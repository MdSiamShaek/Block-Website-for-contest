use eframe::egui::{
    self, Align2, Color32, ColorImage, FontId, IconData, Margin, Pos2, Rect, Rounding, Sense,
    Shape, Stroke, Style, TextureHandle, TextureOptions, Vec2, Visuals,
};
use std::sync::Arc;

/// Embedded bytes of `/Users/siam/Desktop/Block-Website-for-contest/ProctorX Logo.png`
pub const PROCTORX_LOGO_PNG: &[u8] = include_bytes!("../ProctorX Logo.png");

// ============================================================================
// STANDARDIZED UI DIMENSIONS (100% CONSISTENT BUTTON, ICON & INPUT SIZES)
// ============================================================================
pub const ICON_SIZE: f32 = 14.0;
pub const LOGO_HEADER_SIZE: f32 = 38.0;
pub const LOGO_CARD_SIZE: f32 = 24.0;

pub const HEADER_BTN_SIZE: Vec2 = Vec2::new(128.0, 32.0);
pub const SMALL_ACTION_BTN_SIZE: Vec2 = Vec2::new(104.0, 30.0);
pub const INPUT_BAR_BTN_SIZE: Vec2 = Vec2::new(148.0, 36.0);
pub const NAV_BTN_SIZE: Vec2 = Vec2::new(195.0, 36.0);
pub const PRIMARY_CTA_HEIGHT: f32 = 38.0;
pub const TAB_BAR_HEIGHT: f32 = 36.0;
pub const STD_ROUNDING: f32 = 8.0;
pub const SMALL_ROUNDING: f32 = 6.0;

/// Unified vector icon set drawn at an exact `14x14` size so no missing-glyph `□` boxes
/// or mismatched emoji sizes ever occur across platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppIcon {
    Lock,
    Unlock,
    Record,
    Stop,
    Sun,
    Moon,
    Check,
    Download,
    Play,
    Folder,
    Globe,
    Plus,
    Close,
    Eye,
    EyeOff,
    ArrowLeft,
    ArrowRight,
    Edit,
    Trash,
}

/// "Neon Sunshine Bliss" Color Palette (`#0D3B66`, `#FAF0CA`, `#F4D35E`) for **ProctorX**:
#[derive(Clone, Copy)]
pub struct AppPalette {
    pub is_dark: bool,
    pub navy_core: Color32,  // #0D3B66
    pub cream_core: Color32, // #FAF0CA
    pub gold_core: Color32,  // #F4D35E
    pub bg_canvas: Color32,
    pub bg_header: Color32,
    pub bg_card: Color32,
    pub bg_card_subtle: Color32,
    pub bg_input: Color32,
    pub border_subtle: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_on_gold: Color32,
    pub accent_primary: Color32,
    pub accent_hover: Color32,
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
            bg_canvas: navy_core,
            bg_header: Color32::from_rgb(8, 36, 66),
            bg_card: Color32::from_rgb(18, 73, 124),
            bg_card_subtle: Color32::from_rgb(24, 88, 148),
            bg_input: Color32::from_rgb(8, 38, 68),
            border_subtle: Color32::from_rgb(56, 118, 172),
            text_primary: cream_core,
            text_secondary: Color32::from_rgb(224, 214, 178),
            text_muted: Color32::from_rgb(172, 188, 204),
            text_on_gold: Color32::from_rgb(8, 36, 66),
            accent_primary: gold_core,
            accent_hover: Color32::from_rgb(249, 224, 130),
            success_fg: cream_core,
            success_bg: Color32::from_rgb(22, 101, 78),
            danger_fg: Color32::from_rgb(255, 228, 225),
            danger_bg: Color32::from_rgb(153, 27, 48),
            warning_fg: Color32::from_rgb(13, 59, 102),
            warning_bg: gold_core,
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
            bg_canvas: cream_core,
            bg_header: navy_core,
            bg_card: Color32::from_rgb(253, 248, 225),
            bg_card_subtle: gold_core,
            bg_input: Color32::from_rgb(255, 252, 240),
            border_subtle: Color32::from_rgb(185, 168, 115),
            text_primary: navy_core,
            text_secondary: Color32::from_rgb(32, 82, 130),
            text_muted: Color32::from_rgb(78, 108, 138),
            text_on_gold: navy_core,
            accent_primary: gold_core,
            accent_hover: Color32::from_rgb(235, 198, 68),
            success_fg: Color32::from_rgb(13, 59, 102),
            success_bg: Color32::from_rgb(187, 235, 205),
            danger_fg: Color32::from_rgb(250, 240, 202),
            danger_bg: Color32::from_rgb(168, 32, 48),
            warning_fg: navy_core,
            warning_bg: gold_core,
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

        visuals.widgets.noninteractive.bg_fill = self.bg_card;
        visuals.widgets.noninteractive.weak_bg_fill = self.bg_card_subtle;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.noninteractive.rounding = Rounding::same(STD_ROUNDING);

        let inactive_bg = if self.is_dark {
            self.bg_card_subtle
        } else {
            Color32::from_rgb(247, 232, 170)
        };
        visuals.widgets.inactive.bg_fill = inactive_bg;
        visuals.widgets.inactive.weak_bg_fill = inactive_bg;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.2_f32, self.border_subtle);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.inactive.rounding = Rounding::same(STD_ROUNDING);

        visuals.widgets.hovered.bg_fill = self.accent_hover;
        visuals.widgets.hovered.weak_bg_fill = self.accent_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.navy_core);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, self.text_on_gold);
        visuals.widgets.hovered.rounding = Rounding::same(STD_ROUNDING);

        visuals.widgets.active.bg_fill = self.accent_primary;
        visuals.widgets.active.weak_bg_fill = self.accent_primary;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, self.cream_core);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, self.text_on_gold);
        visuals.widgets.active.rounding = Rounding::same(STD_ROUNDING);

        visuals.selection.bg_fill = self.gold_core;
        visuals.selection.stroke = Stroke::new(1.0_f32, self.navy_core);

        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        ctx.set_style(style);
    }

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
            .inner_margin(Margin::symmetric(20.0, 14.0))
    }

    pub fn sub_card_frame(&self) -> egui::Frame {
        egui::Frame::none()
            .fill(self.bg_input)
            .stroke(Stroke::new(1.0_f32, self.border_subtle))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(16.0, 12.0))
    }

    pub fn banner_frame(&self, bg: Color32, border: Color32) -> egui::Frame {
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.5_f32, border))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(18.0, 14.0))
    }

    /// Draws a crisp `14x14` vector icon centered at `center` with `color`.
    pub fn paint_icon(painter: &egui::Painter, center: Pos2, icon: AppIcon, color: Color32) {
        let stroke = Stroke::new(1.7_f32, color);

        match icon {
            AppIcon::Lock => {
                let body = Rect::from_center_size(
                    Pos2::new(center.x, center.y + 2.0),
                    Vec2::new(10.0, 7.5),
                );
                painter.rect_stroke(body, Rounding::same(1.5), stroke);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 3.0, center.y - 1.8),
                        Pos2::new(center.x - 3.0, center.y - 4.5),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + 3.0, center.y - 1.8),
                        Pos2::new(center.x + 3.0, center.y - 4.5),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - 3.0, center.y - 4.5),
                        Pos2::new(center.x + 3.0, center.y - 4.5),
                    ],
                    stroke,
                );
            }
            AppIcon::Unlock => {
                let body = Rect::from_center_size(
                    Pos2::new(center.x, center.y + 2.0),
                    Vec2::new(10.0, 7.5),
                );
                painter.rect_stroke(body, Rounding::same(1.5), stroke);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 3.0, center.y - 1.8),
                        Pos2::new(center.x - 3.0, center.y - 5.0),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - 3.0, center.y - 5.0),
                        Pos2::new(center.x + 2.5, center.y - 5.0),
                    ],
                    stroke,
                );
            }
            AppIcon::Record => {
                painter.circle_filled(center, 4.5, color);
                painter.circle_stroke(center, 6.2, Stroke::new(1.2_f32, color));
            }
            AppIcon::Stop => {
                let sq = Rect::from_center_size(center, Vec2::splat(8.5));
                painter.rect_filled(sq, Rounding::same(1.5), color);
            }
            AppIcon::Sun => {
                painter.circle_stroke(center, 3.2, stroke);
                for i in 0..8 {
                    let angle = (i as f32) * std::f32::consts::TAU / 8.0;
                    let p1 = Pos2::new(center.x + angle.cos() * 4.8, center.y + angle.sin() * 4.8);
                    let p2 = Pos2::new(center.x + angle.cos() * 6.5, center.y + angle.sin() * 6.5);
                    painter.line_segment([p1, p2], stroke);
                }
            }
            AppIcon::Moon => {
                painter.circle_stroke(center, 5.0, stroke);
                painter.line_segment(
                    [
                        Pos2::new(center.x + 1.5, center.y - 3.8),
                        Pos2::new(center.x + 1.5, center.y + 3.8),
                    ],
                    stroke,
                );
            }
            AppIcon::Check => {
                let pts = vec![
                    Pos2::new(center.x - 4.8, center.y + 0.2),
                    Pos2::new(center.x - 1.2, center.y + 3.8),
                    Pos2::new(center.x + 5.0, center.y - 3.8),
                ];
                painter.add(Shape::line(pts, Stroke::new(2.0_f32, color)));
            }
            AppIcon::Download => {
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - 5.2),
                        Pos2::new(center.x, center.y + 2.5),
                    ],
                    stroke,
                );
                let arrow = vec![
                    Pos2::new(center.x - 3.5, center.y - 0.8),
                    Pos2::new(center.x, center.y + 2.8),
                    Pos2::new(center.x + 3.5, center.y - 0.8),
                ];
                painter.add(Shape::line(arrow, stroke));
                painter.line_segment(
                    [
                        Pos2::new(center.x - 4.8, center.y + 5.2),
                        Pos2::new(center.x + 4.8, center.y + 5.2),
                    ],
                    stroke,
                );
            }
            AppIcon::Play => {
                let tri = vec![
                    Pos2::new(center.x - 3.5, center.y - 4.8),
                    Pos2::new(center.x + 4.8, center.y),
                    Pos2::new(center.x - 3.5, center.y + 4.8),
                ];
                painter.add(Shape::convex_polygon(tri, color, Stroke::NONE));
            }
            AppIcon::Folder => {
                let body = Rect::from_center_size(
                    Pos2::new(center.x, center.y + 1.0),
                    Vec2::new(11.0, 7.5),
                );
                painter.rect_stroke(body, Rounding::same(1.5), stroke);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 5.5, center.y - 2.8),
                        Pos2::new(center.x - 1.0, center.y - 2.8),
                    ],
                    stroke,
                );
            }
            AppIcon::Globe => {
                painter.circle_stroke(center, 5.5, stroke);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 5.5, center.y),
                        Pos2::new(center.x + 5.5, center.y),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - 5.5),
                        Pos2::new(center.x, center.y + 5.5),
                    ],
                    stroke,
                );
            }
            AppIcon::Plus => {
                painter.line_segment(
                    [
                        Pos2::new(center.x - 4.5, center.y),
                        Pos2::new(center.x + 4.5, center.y),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - 4.5),
                        Pos2::new(center.x, center.y + 4.5),
                    ],
                    stroke,
                );
            }
            AppIcon::Close => {
                painter.line_segment(
                    [
                        Pos2::new(center.x - 3.8, center.y - 3.8),
                        Pos2::new(center.x + 3.8, center.y + 3.8),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + 3.8, center.y - 3.8),
                        Pos2::new(center.x - 3.8, center.y + 3.8),
                    ],
                    stroke,
                );
            }
            AppIcon::Eye => {
                let rect = Rect::from_center_size(center, Vec2::new(11.0, 6.5));
                painter.rect_stroke(rect, Rounding::same(3.2), stroke);
                painter.circle_filled(center, 1.8, color);
            }
            AppIcon::EyeOff => {
                let rect = Rect::from_center_size(center, Vec2::new(11.0, 6.5));
                painter.rect_stroke(rect, Rounding::same(3.2), stroke);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 5.2, center.y + 4.5),
                        Pos2::new(center.x + 5.2, center.y - 4.5),
                    ],
                    stroke,
                );
            }
            AppIcon::ArrowLeft => {
                painter.line_segment(
                    [
                        Pos2::new(center.x + 4.8, center.y),
                        Pos2::new(center.x - 4.2, center.y),
                    ],
                    stroke,
                );
                let head = vec![
                    Pos2::new(center.x - 1.0, center.y - 3.8),
                    Pos2::new(center.x - 4.8, center.y),
                    Pos2::new(center.x - 1.0, center.y + 3.8),
                ];
                painter.add(Shape::line(head, stroke));
            }
            AppIcon::ArrowRight => {
                painter.line_segment(
                    [
                        Pos2::new(center.x - 4.8, center.y),
                        Pos2::new(center.x + 4.2, center.y),
                    ],
                    stroke,
                );
                let head = vec![
                    Pos2::new(center.x + 1.0, center.y - 3.8),
                    Pos2::new(center.x + 4.8, center.y),
                    Pos2::new(center.x + 1.0, center.y + 3.8),
                ];
                painter.add(Shape::line(head, stroke));
            }
            AppIcon::Edit => {
                painter.line_segment(
                    [
                        Pos2::new(center.x - 4.0, center.y + 4.0),
                        Pos2::new(center.x + 4.0, center.y - 4.0),
                    ],
                    Stroke::new(2.0_f32, color),
                );
                painter.circle_filled(Pos2::new(center.x - 4.5, center.y + 4.5), 1.0, color);
            }
            AppIcon::Trash => {
                let body = Rect::from_center_size(
                    Pos2::new(center.x, center.y + 1.2),
                    Vec2::new(8.0, 8.0),
                );
                painter.rect_stroke(body, Rounding::same(1.0), stroke);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 5.2, center.y - 2.8),
                        Pos2::new(center.x + 5.2, center.y - 2.8),
                    ],
                    stroke,
                );
            }
        }
    }

    /// Renders a non-interactive status pill of exact `size` with an optional uniform `14x14` vector icon and centered text.
    pub fn uniform_badge(
        &self,
        ui: &mut egui::Ui,
        size: Vec2,
        icon: Option<AppIcon>,
        text: &str,
        fg: Color32,
        bg: Color32,
    ) {
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect(
                rect,
                Rounding::same(STD_ROUNDING),
                bg,
                Stroke::new(1.2_f32, fg.gamma_multiply(0.55)),
            );

            let font = FontId::proportional(12.5);
            let galley = painter.layout_no_wrap(text.to_string(), font, fg);
            let text_w = galley.size().x;

            if let Some(ic) = icon {
                let gap = 6.0;
                let total_w = ICON_SIZE + gap + text_w;
                let start_x = rect.center().x - total_w * 0.5;
                let icon_center = Pos2::new(start_x + ICON_SIZE * 0.5, rect.center().y);
                Self::paint_icon(painter, icon_center, ic, fg);
                painter.galley(
                    Pos2::new(
                        start_x + ICON_SIZE + gap,
                        rect.center().y - galley.size().y * 0.5,
                    ),
                    galley,
                    fg,
                );
            } else {
                painter.galley(
                    Pos2::new(
                        rect.center().x - text_w * 0.5,
                        rect.center().y - galley.size().y * 0.5,
                    ),
                    galley,
                    fg,
                );
            }
        }
    }

    /// Renders an interactive button of exact `size` with an optional uniform `14x14` vector icon
    /// (on the left or right) and **100% centered text + icon group**.
    #[allow(clippy::too_many_arguments)]
    pub fn icon_button(
        &self,
        ui: &mut egui::Ui,
        size: Vec2,
        icon: Option<AppIcon>,
        icon_on_right: bool,
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
            let bg = if is_hovered {
                self.accent_hover
            } else {
                fill
            };
            let fg = if is_hovered {
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

            let font = FontId::proportional(font_size);
            let galley = painter.layout_no_wrap(label.to_string(), font, fg);
            let text_w = galley.size().x;
            let text_h = galley.size().y;

            if let Some(ic) = icon {
                let gap = 7.0;
                let total_w = ICON_SIZE + gap + text_w;
                let start_x = rect.center().x - total_w * 0.5;
                if icon_on_right {
                    painter.galley(
                        Pos2::new(start_x, rect.center().y - text_h * 0.5),
                        galley,
                        fg,
                    );
                    let icon_center =
                        Pos2::new(start_x + text_w + gap + ICON_SIZE * 0.5, rect.center().y);
                    Self::paint_icon(painter, icon_center, ic, fg);
                } else {
                    let icon_center = Pos2::new(start_x + ICON_SIZE * 0.5, rect.center().y);
                    Self::paint_icon(painter, icon_center, ic, fg);
                    painter.galley(
                        Pos2::new(start_x + ICON_SIZE + gap, rect.center().y - text_h * 0.5),
                        galley,
                        fg,
                    );
                }
            } else {
                painter.galley(
                    Pos2::new(rect.center().x - text_w * 0.5, rect.center().y - text_h * 0.5),
                    galley,
                    fg,
                );
            }
        }
        response
    }

    /// Shortcut for a centered button without an icon.
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
        self.icon_button(
            ui,
            size,
            None,
            false,
            label,
            font_size,
            fill,
            text_color,
            stroke,
            rounding,
        )
    }

    /// Renders a two-line centered toggle card with a uniform `14x14` `Check` or `Plus` vector icon.
    pub fn centered_two_line_card(
        &self,
        ui: &mut egui::Ui,
        size: Vec2,
        label: &str,
        domain: &str,
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
            painter.rect(rect, Rounding::same(STD_ROUNDING), bg, border);

            let font1 = FontId::proportional(13.0);
            let galley1 = painter.layout_no_wrap(label.to_string(), font1, fg1);
            let gap = 5.0;
            let total_w = ICON_SIZE + gap + galley1.size().x;
            let start_x = rect.center().x - total_w * 0.5;
            let row1_y = rect.center().y - 8.0;

            let icon = if is_active {
                AppIcon::Check
            } else {
                AppIcon::Plus
            };
            Self::paint_icon(
                painter,
                Pos2::new(start_x + ICON_SIZE * 0.5, row1_y),
                icon,
                fg1,
            );
            painter.galley(
                Pos2::new(
                    start_x + ICON_SIZE + gap,
                    row1_y - galley1.size().y * 0.5,
                ),
                galley1,
                fg1,
            );

            painter.text(
                Pos2::new(rect.center().x, rect.center().y + 9.0),
                Align2::CENTER_CENTER,
                format!("({})", domain),
                FontId::proportional(11.5),
                fg2,
            );
        }
        response
    }

    /// Draws the loaded `ProctorX Logo.png` texture at the specified `size`.
    pub fn draw_logo_image(&self, ui: &mut egui::Ui, logo_tex: &TextureHandle, size: Vec2) -> egui::Response {
        let img = egui::Image::from_texture(logo_tex)
            .fit_to_exact_size(size)
            .rounding(Rounding::same(6.0))
            .sense(Sense::click());
        ui.add(img)
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

/// Decodes `ProctorX Logo.png` with a clean margin around the rounded-square emblem (`x: 28%..72%, y: 16%..60%`)
/// so the outer gold border of `ProctorX Logo.png` is never clipped.
pub fn load_logo_texture(ctx: &egui::Context) -> TextureHandle {
    let img = image::load_from_memory(PROCTORX_LOGO_PNG)
        .expect("Failed to decode ProctorX Logo.png");
    let (w, h) = (img.width(), img.height());
    let crop_x = (w as f32 * 0.28) as u32;
    let crop_y = (h as f32 * 0.16) as u32;
    let crop_w = (w as f32 * 0.44) as u32;
    let crop_h = (h as f32 * 0.44) as u32;
    let cropped = img.crop_imm(crop_x, crop_y, crop_w, crop_h).into_rgba8();

    let (cw, ch) = cropped.dimensions();
    let color_image =
        ColorImage::from_rgba_unmultiplied([cw as usize, ch as usize], cropped.as_raw());
    ctx.load_texture("proctorx_logo", color_image, TextureOptions::LINEAR)
}

/// Decodes `ProctorX Logo.png` and resizes to a 128x128 native OS window & dock icon (`IconData`).
pub fn build_window_icon() -> Arc<IconData> {
    if let Ok(img) = image::load_from_memory(PROCTORX_LOGO_PNG) {
        let resized =
            image::imageops::resize(&img, 128, 128, image::imageops::FilterType::Lanczos3);
        let (w, h) = resized.dimensions();
        return Arc::new(IconData {
            rgba: resized.into_raw(),
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
