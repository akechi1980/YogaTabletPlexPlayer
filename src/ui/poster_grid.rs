use eframe::egui;

use crate::models::MediaItem;

/// 海报图在界面上的显示尺寸（与缓存转码尺寸比例一致）。
pub const POSTER_WIDTH: f32 = 140.0;
pub const POSTER_HEIGHT: f32 = 210.0;
/// 单张卡片占用宽度（含标题与边距），用于计算列数。
pub const POSTER_CARD_WIDTH: f32 = 164.0;

/// 根据可用宽度计算列数，至少一列。
pub fn poster_columns(available_width: f32) -> usize {
    let columns = (available_width / POSTER_CARD_WIDTH).floor() as usize;
    columns.max(1)
}

/// 缩略图未就绪时在格子里绘制占位矩形与标题文字。
pub fn draw_placeholder(ui: &mut egui::Ui, _title: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(POSTER_WIDTH, POSTER_HEIGHT),
        egui::Sense::click(),
    );
    ui.painter()
        .rect_filled(rect, 6.0, egui::Color32::from_rgb(42, 46, 54));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "无图",
        egui::FontId::proportional(18.0),
        egui::Color32::LIGHT_GRAY,
    );

    response
}

/// 返回标题包含搜索词（不区分大小写）的条目在 `items` 中的下标。
pub fn filtered_items(items: &[MediaItem], needle: &str) -> Vec<usize> {
    let trimmed = needle.trim();
    if trimmed.is_empty() {
        return (0..items.len()).collect();
    }

    let lowered = trimmed.to_lowercase();
    items.iter()
        .enumerate()
        .filter(|(_, item)| item.title.to_lowercase().contains(&lowered))
        .map(|(index, _)| index)
        .collect()
}
