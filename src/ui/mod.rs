pub mod explorer;
pub mod modern;

use egui::{Color32, FontId, Response, RichText, Stroke, Ui, Vec2};
use crate::theme::Tokens;

// ── Shared widget helpers ────────────────────────────────────────────────────

pub fn nav_button(ui: &mut Ui, tok: &Tokens, icon: char, enabled: bool) -> Response {
    let size = Vec2::splat(30.0);
    let color = if enabled { tok.dim } else { tok.faint };
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let paint = ui.painter();
    if resp.hovered() && enabled {
        paint.rect_filled(rect, egui::Rounding::same(7.0), tok.hover);
    }
    let icon_color = if resp.hovered() && enabled { tok.text } else { color };
    paint.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon.to_string(),
        FontId::proportional(20.0),
        icon_color,
    );
    resp
}

pub fn nav_button_active(ui: &mut Ui, tok: &Tokens, icon: char, active: bool) -> Response {
    let size = Vec2::splat(30.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let paint = ui.painter();
    if active {
        paint.rect_filled(rect, egui::Rounding::same(7.0), tok.accent_soft);
    } else if resp.hovered() {
        paint.rect_filled(rect, egui::Rounding::same(7.0), tok.hover);
    }
    let color = if active { tok.accent } else if resp.hovered() { tok.text } else { tok.dim };
    paint.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon.to_string(),
        FontId::proportional(20.0),
        color,
    );
    resp
}

pub fn segment_button(
    ui: &mut Ui,
    tok: &Tokens,
    icon: char,
    active: bool,
) -> Response {
    let size = Vec2::new(30.0, 26.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let paint = ui.painter();
    if active {
        paint.rect_filled(rect, egui::Rounding::same(6.0), tok.accent_soft);
    }
    let color = if active { tok.accent } else { tok.dim };
    paint.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon.to_string(),
        FontId::proportional(18.0),
        color,
    );
    resp
}

pub fn mono_label(ui: &mut Ui, text: impl Into<String>, size: f32, color: Color32) {
    ui.label(
        RichText::new(text)
            .font(FontId::monospace(size))
            .color(color),
    );
}

pub fn section_label(ui: &mut Ui, tok: &Tokens, text: &str) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(text)
            .font(FontId::proportional(10.5))
            .color(tok.faint)
            .strong(),
    );
    ui.add_space(2.0);
}

pub fn col_header(ui: &mut Ui, tok: &Tokens, text: &str) {
    ui.label(
        RichText::new(text)
            .font(FontId::proportional(10.5))
            .color(tok.faint)
            .strong(),
    );
}

pub fn divider(ui: &mut Ui, tok: &Tokens) {
    ui.painter().line_segment(
        [
            ui.cursor().min,
            egui::pos2(ui.cursor().min.x + ui.available_width(), ui.cursor().min.y),
        ],
        Stroke::new(1.0, tok.border),
    );
}
