use eframe::egui::{self, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, TextureHandle, Vec2};

use super::theme::colors;

/// 海报网格列数和单卡宽度。列宽随可用空间变化，打开详情栏时会自动少一列。
pub fn grid_metrics(available_width: f32) -> (usize, f32) {
    let gap = 10.0;
    let min_card = 118.0;
    let columns = ((available_width + gap) / (min_card + gap))
        .floor()
        .max(2.0) as usize;
    let columns = columns.clamp(2, 7);
    let card_w = (available_width - gap * (columns.saturating_sub(1) as f32)) / columns as f32;
    (columns, card_w.max(96.0))
}

/// 绘制一张海报卡片。点击海报、标题或年份都会返回 clicked。
pub fn draw_poster_card(
    ui: &mut egui::Ui,
    width: f32,
    texture: Option<&TextureHandle>,
    title: &str,
    year: Option<i32>,
    selected: bool,
    watched: bool,
    progress: Option<f32>,
) -> egui::Response {
    let height = width * 1.5;
    ui.vertical(|ui| {
        ui.set_width(width);
        ui.spacing_mut().item_spacing.y = 4.0;
        let (rect, image_response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        paint_poster(ui, rect, texture);
        if watched {
            paint_watched_badge(ui, rect);
        }
        if let Some(progress) = progress {
            paint_progress(ui, rect, progress);
        }
        let stroke_color = if selected {
            colors().accent
        } else if image_response.hovered() {
            colors().poster_edge
        } else {
            Color32::TRANSPARENT
        };
        if stroke_color != Color32::TRANSPARENT {
            ui.painter().rect_stroke(
                rect,
                Rounding::same(8.0),
                Stroke::new(if selected { 2.5 } else { 2.0 }, stroke_color),
            );
        }

        ui.add_space(6.0);
        let title_response = draw_clamped_title(ui, title, width);
        let year_text = year
            .map(|year| year.to_string())
            .unwrap_or_else(|| "—".to_owned());
        let year_response = ui.add(egui::Label::new(
            egui::RichText::new(year_text).size(12.0).color(colors().muted),
        ));

        image_response.union(title_response).union(year_response)
    })
    .inner
}

fn paint_poster(ui: &egui::Ui, rect: Rect, texture: Option<&TextureHandle>) {
    if let Some(texture) = texture {
        egui::Image::from_texture(texture)
            .rounding(8.0)
            .bg_fill(colors().poster)
            .paint_at(ui, rect);
    } else {
        ui.painter()
            .rect_filled(rect, Rounding::same(8.0), colors().poster);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "无图",
            FontId::proportional(16.0),
            colors().muted,
        );
    }
}

fn paint_watched_badge(ui: &egui::Ui, poster_rect: Rect) {
    let badge = Rect::from_min_size(
        Pos2::new(poster_rect.right() - 70.0, poster_rect.top() + 8.0),
        Vec2::new(62.0, 22.0),
    );
    ui.painter().rect_filled(
        badge,
        Rounding::same(6.0),
        colors().badge_bg,
    );
    ui.painter().text(
        badge.center(),
        egui::Align2::CENTER_CENTER,
        "✓ 已看",
        FontId::proportional(11.0),
        colors().badge_text,
    );
}

fn paint_progress(ui: &egui::Ui, poster_rect: Rect, progress: f32) {
    let bar = Rect::from_min_max(
        Pos2::new(poster_rect.left(), poster_rect.bottom() - 4.0),
        poster_rect.right_bottom(),
    );
    ui.painter()
        .rect_filled(bar, Rounding::ZERO, Color32::from_rgba_unmultiplied(0, 0, 0, 0x88));
    let mut fill = bar;
    fill.max.x = bar.min.x + bar.width() * progress.clamp(0.0, 1.0);
    ui.painter().rect_filled(fill, Rounding::ZERO, colors().accent);
}

fn draw_clamped_title(ui: &mut egui::Ui, title: &str, width: f32) -> egui::Response {
    let font = FontId::proportional(12.0);
    let color = colors().text;
    let galley = ui.fonts(|fonts| fonts.layout(title.to_owned(), font, color, width));
    let height = 32.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    ui.painter()
        .with_clip_rect(rect)
        .galley(rect.left_top(), galley, color);
    response
}
