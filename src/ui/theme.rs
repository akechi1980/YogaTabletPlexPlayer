//! 夜间铁灰、白天灰白两套界面色。

use std::sync::atomic::{AtomicU8, Ordering};

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};

use crate::config::ThemeMode;

static MODE: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy)]
pub struct Palette {
    pub light: bool,
    pub bg: Color32,
    pub top: Color32,
    pub panel: Color32,
    pub field: Color32,
    pub muted: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub tab_active: Color32,
    pub tab_text: Color32,
    pub danger: Color32,
    pub danger_bg: Color32,
    pub danger_fill: Color32,
    pub danger_hover: Color32,
    pub online: Color32,
    pub line: Color32,
    pub hover: Color32,
    pub stroke: Color32,
    pub window: Color32,
    pub extreme: Color32,
    pub settings_bg: Color32,
    pub card: Color32,
    pub card_line: Color32,
    pub track: Color32,
    pub summary: Color32,
    pub veil: Color32,
    pub poster: Color32,
    pub poster_edge: Color32,
    pub badge_bg: Color32,
    pub badge_text: Color32,
    pub brand: Color32,
    pub brand_ink: Color32,
}

const NIGHT: Palette = Palette {
    light: false,
    bg: Color32::from_rgb(0x45, 0x49, 0x4f),
    top: Color32::from_rgb(0x3c, 0x40, 0x46),
    panel: Color32::from_rgb(0x4e, 0x53, 0x5a),
    field: Color32::from_rgb(0x5a, 0x5f, 0x66),
    muted: Color32::from_rgb(0xc9, 0xcd, 0xd3),
    text: Color32::from_rgb(0xf5, 0xf6, 0xf7),
    accent: Color32::from_rgb(0xe5, 0xad, 0x70),
    accent_text: Color32::from_rgb(0x23, 0x1c, 0x16),
    tab_active: Color32::from_rgb(0x5a, 0x53, 0x48),
    tab_text: Color32::from_rgb(0xf6, 0xd7, 0xb4),
    danger: Color32::from_rgb(0xe0, 0xa8, 0xac),
    danger_bg: Color32::from_rgb(0x5a, 0x40, 0x44),
    danger_fill: Color32::from_rgb(0x99, 0x3b, 0x48),
    danger_hover: Color32::from_rgb(0x8d, 0x3a, 0x44),
    online: Color32::from_rgb(0xa8, 0xd4, 0xb4),
    line: Color32::from_rgb(0x65, 0x6a, 0x71),
    hover: Color32::from_rgb(0x65, 0x6a, 0x72),
    stroke: Color32::from_rgb(0x6e, 0x73, 0x7a),
    window: Color32::from_rgb(0x55, 0x5a, 0x61),
    extreme: Color32::from_rgb(0x3e, 0x42, 0x48),
    settings_bg: Color32::from_rgb(0x40, 0x44, 0x4a),
    card: Color32::from_rgb(0x52, 0x57, 0x5e),
    card_line: Color32::from_rgb(0x6a, 0x6f, 0x76),
    track: Color32::from_rgb(0x5c, 0x61, 0x68),
    summary: Color32::from_rgb(0xd5, 0xd8, 0xdd),
    veil: Color32::from_rgba_premultiplied(0x16, 0x17, 0x1a, 0xb0),
    poster: Color32::from_rgb(0x3e, 0x43, 0x49),
    poster_edge: Color32::from_rgb(0xc4, 0xb4, 0xa4),
    badge_bg: Color32::from_rgba_premultiplied(0x19, 0x26, 0x1e, 0xe8),
    badge_text: Color32::from_rgb(0xbc, 0xe0, 0xc5),
    brand: Color32::from_rgb(0xd4, 0xa3, 0x6c),
    brand_ink: Color32::from_rgb(0x1a, 0x16, 0x14),
};

const DAY: Palette = Palette {
    light: true,
    bg: Color32::from_rgb(0xe6, 0xe8, 0xec),
    top: Color32::from_rgb(0xf6, 0xf7, 0xf9),
    panel: Color32::from_rgb(0xfb, 0xfc, 0xfd),
    field: Color32::from_rgb(0xff, 0xff, 0xff),
    muted: Color32::from_rgb(0x62, 0x68, 0x73),
    text: Color32::from_rgb(0x2a, 0x2e, 0x34),
    accent: Color32::from_rgb(0xd4, 0x92, 0x4f),
    accent_text: Color32::from_rgb(0x2a, 0x21, 0x18),
    tab_active: Color32::from_rgb(0xf4, 0xe6, 0xd4),
    tab_text: Color32::from_rgb(0x8d, 0x5a, 0x28),
    danger: Color32::from_rgb(0xa3, 0x3d, 0x48),
    danger_bg: Color32::from_rgb(0xf8, 0xe4, 0xe7),
    danger_fill: Color32::from_rgb(0xb5, 0x52, 0x5c),
    danger_hover: Color32::from_rgb(0xc4, 0x68, 0x70),
    online: Color32::from_rgb(0x2e, 0x8a, 0x4e),
    line: Color32::from_rgb(0xd4, 0xd8, 0xdf),
    hover: Color32::from_rgb(0xe4, 0xe7, 0xec),
    stroke: Color32::from_rgb(0xc6, 0xcb, 0xd3),
    window: Color32::from_rgb(0xff, 0xff, 0xff),
    extreme: Color32::from_rgb(0xff, 0xff, 0xff),
    settings_bg: Color32::from_rgb(0xee, 0xf0, 0xf4),
    card: Color32::from_rgb(0xff, 0xff, 0xff),
    card_line: Color32::from_rgb(0xd8, 0xdc, 0xe3),
    track: Color32::from_rgb(0xd7, 0xdb, 0xe2),
    summary: Color32::from_rgb(0x5a, 0x61, 0x6a),
    veil: Color32::from_rgba_premultiplied(0x17, 0x19, 0x1c, 0x8c),
    poster: Color32::from_rgb(0xd8, 0xdc, 0xe2),
    poster_edge: Color32::from_rgb(0xb0, 0x89, 0x68),
    badge_bg: Color32::from_rgba_premultiplied(0x19, 0x26, 0x1e, 0xe8),
    badge_text: Color32::from_rgb(0xbc, 0xe0, 0xc5),
    brand: Color32::from_rgb(0xd4, 0xa3, 0x6c),
    brand_ink: Color32::from_rgb(0x1a, 0x16, 0x14),
};

pub fn set_mode(mode: ThemeMode) {
    MODE.store(
        match mode {
            ThemeMode::Night => 0,
            ThemeMode::Day => 1,
        },
        Ordering::Relaxed,
    );
}

pub fn mode() -> ThemeMode {
    if MODE.load(Ordering::Relaxed) == 1 {
        ThemeMode::Day
    } else {
        ThemeMode::Night
    }
}

pub fn colors() -> Palette {
    if mode() == ThemeMode::Day {
        DAY
    } else {
        NIGHT
    }
}

pub fn apply(ctx: &egui::Context) {
    let palette = colors();
    let mut visuals = if palette.light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    visuals.panel_fill = palette.bg;
    visuals.window_fill = palette.window;
    visuals.extreme_bg_color = palette.extreme;
    visuals.faint_bg_color = palette.field;
    visuals.code_bg_color = palette.field;
    visuals.widgets.noninteractive.bg_fill = palette.bg;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.muted);
    visuals.widgets.inactive.bg_fill = palette.field;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.inactive.rounding = Rounding::same(9.0);
    visuals.widgets.hovered.bg_fill = palette.hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.hovered.rounding = Rounding::same(9.0);
    visuals.widgets.active.bg_fill = palette.tab_active;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.active.rounding = Rounding::same(9.0);
    visuals.widgets.open.bg_fill = palette.field;
    visuals.widgets.open.rounding = Rounding::same(9.0);
    visuals.selection.bg_fill = palette.accent;
    visuals.selection.stroke = Stroke::new(1.0, palette.accent_text);
    visuals.window_rounding = Rounding::same(15.0);
    visuals.window_stroke = Stroke::new(1.0, palette.card_line);
    visuals.hyperlink_color = palette.accent;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(14.0, 6.0);
    style.spacing.interact_size.y = 32.0;
    ctx.set_style(style);
}
