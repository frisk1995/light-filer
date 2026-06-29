use egui::{Color32, FontId, Rounding, ScrollArea, Sense, Stroke, Vec2};
use crate::{
    app::{ContextAction, FerroApp},
    entry::{Entry, EntryKind},
    icons,
    theme::Tokens,
};

const TILE_W: f32 = 110.0;
const TILE_H: f32 = 90.0;
const GRID_PADDING: f32 = 14.0;
const GRID_GAP: f32 = 8.0;

pub fn show(app: &mut FerroApp, ctx: &egui::Context) {
    let tok = app.tokens.clone();
    show_sidebar(app, ctx, &tok);
    show_preview(app, ctx, &tok);
    show_grid(app, ctx, &tok);
}

fn show_sidebar(app: &mut FerroApp, ctx: &egui::Context, tok: &Tokens) {
    egui::SidePanel::left("modern_sidebar")
        .resizable(false)
        .exact_width(190.0)
        .frame(
            egui::Frame::none()
                .fill(tok.bg)
                .inner_margin(egui::Margin::symmetric(8.0, 10.0)),
        )
        .show(ctx, |ui| {
            crate::ui::section_label(ui, tok, "QUICK ACCESS");

            let projects = std::path::PathBuf::from("C:\\dev");
            let is_proj = app.main_pane.path == projects;
            sidebar_item(ui, tok, icons::STAR, "Projects", &projects, is_proj, app);
            sidebar_item(
                ui,
                tok,
                icons::SCHEDULE,
                "Recent",
                &std::path::PathBuf::from("C:\\"),
                false,
                app,
            );

            ui.add_space(8.0);
            // Show current dir's parent as a mini tree
            if let Some(name) = app.main_pane.path.file_name().map(|n| n.to_string_lossy().to_string()) {
                crate::ui::section_label(ui, tok, &name.to_uppercase());
                let dirs: Vec<(String, std::path::PathBuf)> = app
                    .main_pane
                    .entries
                    .iter()
                    .filter(|e| e.kind == EntryKind::Dir)
                    .map(|e| (e.name.clone(), e.path.clone()))
                    .collect();
                let mut nav_to: Option<std::path::PathBuf> = None;
                for (dname, dpath) in &dirs {
                    let (rect, resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 24.0),
                        Sense::click(),
                    );
                    if resp.hovered() {
                        ui.painter()
                            .rect_filled(rect, Rounding::same(5.0), tok.hover);
                    }
                    ui.painter().text(
                        egui::pos2(rect.min.x + 14.0, rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        icons::FOLDER.to_string(),
                        FontId::proportional(13.0),
                        Color32::from_rgb(0xcf, 0x9b, 0x53),
                    );
                    ui.painter().text(
                        egui::pos2(rect.min.x + 26.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        dname.as_str(),
                        FontId::proportional(12.0),
                        tok.dim,
                    );
                    if resp.clicked() {
                        nav_to = Some(dpath.clone());
                    }
                }
                if let Some(p) = nav_to {
                    app.navigate_to(p);
                }
            }
        });
}

fn sidebar_item(
    ui: &mut egui::Ui,
    tok: &Tokens,
    icon: char,
    label: &str,
    path: &std::path::Path,
    is_active: bool,
    app: &mut FerroApp,
) {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 26.0),
        Sense::click(),
    );
    let bg = if is_active {
        ui.painter().rect_filled(rect, Rounding::same(6.0), tok.accent_soft);
        tok.accent
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, Rounding::same(6.0), tok.hover);
        tok.dim
    } else {
        tok.dim
    };

    ui.painter().text(
        egui::pos2(rect.min.x + 14.0, rect.center().y),
        egui::Align2::CENTER_CENTER,
        icon.to_string(),
        FontId::proportional(14.0),
        bg,
    );
    ui.painter().text(
        egui::pos2(rect.min.x + 26.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.5),
        if is_active { tok.text } else { tok.dim },
    );
    if resp.clicked() {
        app.navigate_to(path.to_path_buf());
    }
}

fn show_preview(app: &mut FerroApp, ctx: &egui::Context, tok: &Tokens) {
    egui::SidePanel::right("modern_preview")
        .resizable(false)
        .exact_width(300.0)
        .frame(
            egui::Frame::none()
                .fill(tok.bg)
                .stroke(Stroke::new(1.0, tok.border)),
        )
        .show(ctx, |ui| {
            let Some(idx) = app.preview_idx else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Select a file to preview")
                            .color(tok.faint)
                            .size(12.0),
                    );
                });
                return;
            };
            let Some(entry) = app.main_pane.entries.get(idx).cloned() else {
                return;
            };

            // Header
            let header_rect = ui
                .allocate_exact_size(Vec2::new(300.0, 90.0), Sense::hover())
                .0;
            ui.painter().rect_filled(header_rect, Rounding::ZERO, tok.bg);
            ui.painter().line_segment(
                [header_rect.left_bottom(), header_rect.right_bottom()],
                Stroke::new(1.0, tok.border),
            );

            ui.painter().text(
                egui::pos2(header_rect.center().x, header_rect.min.y + 28.0),
                egui::Align2::CENTER_CENTER,
                entry.icon_char().to_string(),
                FontId::proportional(46.0),
                entry.icon_color(tok.accent),
            );
            ui.painter().text(
                egui::pos2(header_rect.center().x, header_rect.min.y + 60.0),
                egui::Align2::CENTER_CENTER,
                &entry.name,
                FontId::new(14.0, egui::FontFamily::Monospace),
                tok.text,
            );
            ui.painter().text(
                egui::pos2(header_rect.center().x, header_rect.min.y + 76.0),
                egui::Align2::CENTER_CENTER,
                entry.path.to_string_lossy().as_ref(),
                FontId::monospace(10.0),
                tok.faint,
            );

            // Code preview for text files
            let ext = entry.name.rsplit('.').next().unwrap_or("").to_lowercase();
            let is_text = matches!(
                ext.as_str(),
                "rs" | "toml" | "md" | "txt" | "json" | "yaml" | "yml" | "lock"
            );

            if is_text && entry.kind == EntryKind::File {
                let preview = read_preview(&entry.path, 20);
                egui::Frame::none()
                    .fill(tok.list)
                    .rounding(Rounding::same(8.0))
                    .stroke(Stroke::new(1.0, tok.border))
                    .inner_margin(egui::Margin::same(12.0))
                    .outer_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for line in &preview {
                                ui.label(
                                    egui::RichText::new(line)
                                        .font(FontId::monospace(11.0))
                                        .color(tok.text),
                                );
                            }
                        });
                    });
            }

            // Metadata
            ui.add_space(4.0);
            meta_row(ui, tok, "Type", entry.type_label());
            if let Some(sz) = entry.size {
                meta_row(ui, tok, "Size", &entry.size_display());
            }
            meta_row(ui, tok, "Modified", &entry.modified_display());

            // Open / More buttons
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let btn_h = 34.0;
                let total_w = ui.available_width() - 16.0;
                let open_w = total_w - 42.0 - 8.0;

                let (open_rect, open_resp) = ui.allocate_exact_size(Vec2::new(open_w, btn_h), Sense::click());
                ui.painter()
                    .rect_filled(open_rect, Rounding::same(8.0), tok.accent);
                ui.painter().text(
                    open_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} Open", icons::OPEN_IN_NEW),
                    FontId::proportional(13.0),
                    Color32::WHITE,
                );
                if open_resp.clicked() {
                    let _ = open::that(&entry.path);
                }

                let (more_rect, _) = ui.allocate_exact_size(Vec2::new(42.0, btn_h), Sense::click());
                ui.painter()
                    .rect_filled(more_rect, Rounding::same(8.0), tok.elev);
                ui.painter().rect_stroke(
                    more_rect,
                    Rounding::same(8.0),
                    Stroke::new(1.0, tok.border),
                );
                ui.painter().text(
                    more_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icons::MORE_HORIZ.to_string(),
                    FontId::proportional(16.0),
                    tok.dim,
                );
            });
        });
}

fn show_grid(app: &mut FerroApp, ctx: &egui::Context, tok: &Tokens) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(tok.list))
        .show(ctx, |ui| {
            let count_text = format!("{} items", app.main_pane.entries.len());
            let (label_rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::hover());
            ui.painter().text(
                egui::pos2(label_rect.min.x + GRID_PADDING, label_rect.center().y),
                egui::Align2::LEFT_CENTER,
                &count_text,
                FontId::proportional(11.0),
                tok.dim,
            );

            let avail_w = ui.available_width() - GRID_PADDING * 2.0;
            let cols = ((avail_w + GRID_GAP) / (TILE_W + GRID_GAP)).floor().max(1.0) as usize;

            let show_hidden = app.show_hidden;
            let entries: Vec<_> = app.main_pane.entries.iter()
                .filter(|e| show_hidden || !e.is_hidden)
                .cloned()
                .collect();
            let preview_idx = app.preview_idx;

            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(GRID_PADDING - 4.0);
                egui::Grid::new("modern_grid")
                    .num_columns(cols)
                    .spacing(Vec2::splat(GRID_GAP))
                    .min_col_width(TILE_W)
                    .max_col_width(TILE_W + 20.0)
                    .show(ui, |ui| {
                        for (i, entry) in entries.iter().enumerate() {
                            let is_selected = preview_idx == Some(i);
                            let (tile_rect, resp) = ui.allocate_exact_size(
                                Vec2::new(TILE_W, TILE_H),
                                Sense::click(),
                            );

                            if is_selected {
                                ui.painter().rect_filled(
                                    tile_rect,
                                    Rounding::same(10.0),
                                    tok.accent_soft,
                                );
                                ui.painter().rect_stroke(
                                    tile_rect,
                                    Rounding::same(10.0),
                                    Stroke::new(1.0, tok.accent),
                                );
                            } else if resp.hovered() {
                                ui.painter().rect_filled(
                                    tile_rect,
                                    Rounding::same(10.0),
                                    tok.hover,
                                );
                            }

                            // Icon (42px centered)
                            ui.painter().text(
                                egui::pos2(tile_rect.center().x, tile_rect.min.y + 34.0),
                                egui::Align2::CENTER_CENTER,
                                entry.icon_char().to_string(),
                                FontId::proportional(42.0),
                                entry.icon_color(tok.accent),
                            );

                            // Name (11.5px, centered, ellipsis)
                            let name = if entry.name.chars().count() > 14 {
                                format!("{}…", entry.name.chars().take(12).collect::<String>())
                            } else {
                                entry.name.clone()
                            };
                            ui.painter().text(
                                egui::pos2(tile_rect.center().x, tile_rect.min.y + 68.0),
                                egui::Align2::CENTER_CENTER,
                                &name,
                                FontId::proportional(11.5),
                                if is_selected { tok.text } else { tok.dim },
                            );

                            if resp.clicked() {
                                app.preview_idx = Some(i);
                            }
                            if resp.double_clicked() {
                                if entry.kind == EntryKind::Dir {
                                    app.navigate_to(entry.path.clone());
                                } else {
                                    let _ = open::that(&entry.path);
                                }
                            }

                            let ep = entry.path.clone();
                            let is_dir = entry.kind == EntryKind::Dir;
                            resp.context_menu(|ui| {
                                ui.set_min_width(180.0);
                                if ui.button("開く").clicked() {
                                    app.pending_action = Some(ContextAction::Open(ep.clone()));
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("パスをコピー").clicked() {
                                    app.pending_action = Some(ContextAction::CopyPath(ep.clone()));
                                    ui.close_menu();
                                }
                                if is_dir {
                                    if ui.button("ターミナルで開く").clicked() {
                                        app.pending_action = Some(ContextAction::OpenTerminal(ep.clone()));
                                        ui.close_menu();
                                    }
                                    if ui.button("クイックアクセスに追加").clicked() {
                                        app.pending_action = Some(ContextAction::AddQuickAccess(ep.clone()));
                                        ui.close_menu();
                                    }
                                }
                                ui.separator();
                                if ui.button(egui::RichText::new("削除").color(egui::Color32::from_rgb(0xc8, 0x37, 0x2c))).clicked() {
                                    app.pending_action = Some(ContextAction::Delete(vec![ep.clone()]));
                                    ui.close_menu();
                                }
                            });

                            if (i + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
                ui.add_space(GRID_PADDING);
            });
        });
}

fn read_preview(path: &std::path::Path, max_lines: usize) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .take(max_lines)
        .map(|l| l.to_owned())
        .collect()
}

fn meta_row(ui: &mut egui::Ui, tok: &Tokens, label: &str, value: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::hover());
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, tok.border),
    );
    ui.painter().text(
        egui::pos2(rect.min.x + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.5),
        tok.faint,
    );
    ui.painter().text(
        egui::pos2(rect.max.x - 16.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        value,
        FontId::proportional(11.5),
        tok.text,
    );
}

fn open_path(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()?;
    }
    Ok(())
}
