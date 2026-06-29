use egui::{Color32, FontId, RichText, Rounding, ScrollArea, Sense, Stroke, Vec2};
use crate::{
    app::{ContextAction, FerroApp},
    entry::EntryKind,
    icons,
    theme::Tokens,
    ui::col_header,
};

const ROW_H: f32 = 30.0;
const TREE_ROW_H: f32 = 26.0;

const NEW_FILE_TEMPLATES: &[(&str, &str)] = &[
    ("テキスト ドキュメント (.txt)",                    "新しいテキスト ドキュメント.txt"),
    ("Microsoft Word 文書 (.docx)",                    "新しい Microsoft Word 文書.docx"),
    ("Microsoft Excel ワークシート (.xlsx)",            "新しい Microsoft Excel ワークシート.xlsx"),
    ("Microsoft PowerPoint プレゼンテーション (.pptx)", "新しい Microsoft PowerPoint プレゼンテーション.pptx"),
    ("リッチ テキスト ドキュメント (.rtf)",              "新しいリッチ テキスト ドキュメント.rtf"),
];

pub fn show(app: &mut FerroApp, ctx: &egui::Context) {
    let tok = app.tokens.clone();
    show_sidebar(app, ctx, &tok);
    show_list(app, ctx, &tok);
}

fn show_sidebar(app: &mut FerroApp, ctx: &egui::Context, tok: &Tokens) {
    egui::SidePanel::left("explorer_sidebar")
        .resizable(false)
        .exact_width(222.0)
        .frame(
            egui::Frame::none()
                .fill(tok.bg)
                .inner_margin(egui::Margin::symmetric(8.0, 10.0)),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;

            // Quick Access (dynamic)
            crate::ui::section_label(ui, tok, "QUICK ACCESS");
            let qa: Vec<(usize, String, std::path::PathBuf)> = app.quick_access
                .iter()
                .enumerate()
                .map(|(i, (n, p))| (i, n.clone(), p.clone()))
                .collect();
            for (idx, name, path) in qa {
                let icon = if name == "Home" { icons::HOME }
                    else if name == "Downloads" { icons::DOWNLOAD }
                    else if name == "Desktop" { icons::STAR }
                    else { icons::FOLDER };
                let is_active = app.main_pane.path == path;
                let (rect, resp) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 28.0), Sense::click());
                let paint = ui.painter();
                if is_active {
                    paint.rect_filled(rect, Rounding::same(6.0), tok.accent_soft);
                    let bar = egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height() - 8.0))
                        .translate(Vec2::new(0.0, 4.0));
                    paint.rect_filled(bar, Rounding::same(2.0), tok.accent);
                } else if resp.hovered() {
                    paint.rect_filled(rect, Rounding::same(6.0), tok.hover);
                }
                let icon_color = if is_active { tok.accent } else { tok.dim };
                paint.text(egui::pos2(rect.min.x + 14.0, rect.center().y),
                    egui::Align2::CENTER_CENTER, icon.to_string(),
                    FontId::proportional(16.0), icon_color);
                paint.text(egui::pos2(rect.min.x + 26.0, rect.center().y),
                    egui::Align2::LEFT_CENTER, &name,
                    FontId::proportional(13.0),
                    if is_active { tok.text } else { tok.dim });
                if resp.clicked() { app.navigate_to(path.clone()); }
                resp.context_menu(|ui| {
                    if ui.button("Remove from Quick Access").clicked() {
                        app.pending_action = Some(ContextAction::RemoveQuickAccess(idx));
                        ui.close_menu();
                    }
                });
            }

            // OneDrive（検出された場合のみ表示）
            let onedrive = app.onedrive_paths.clone();
            if !onedrive.is_empty() {
                ui.add_space(8.0);
                crate::ui::section_label(ui, tok, "CLOUD");
                for (name, path) in &onedrive {
                    let is_active = app.main_pane.path.starts_with(path);
                    let (rect, resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 28.0), Sense::click());
                    let paint = ui.painter();
                    if is_active {
                        paint.rect_filled(rect, Rounding::same(6.0), tok.accent_soft);
                        let bar = egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height() - 8.0))
                            .translate(Vec2::new(0.0, 4.0));
                        paint.rect_filled(bar, Rounding::same(2.0), tok.accent);
                    } else if resp.hovered() {
                        paint.rect_filled(rect, Rounding::same(6.0), tok.hover);
                    }
                    let icon_color = if is_active { tok.accent } else { Color32::from_rgb(0x00, 0x7a, 0xff) };
                    paint.text(egui::pos2(rect.min.x + 14.0, rect.center().y),
                        egui::Align2::CENTER_CENTER, icons::CLOUD.to_string(),
                        FontId::proportional(16.0), icon_color);
                    paint.text(egui::pos2(rect.min.x + 26.0, rect.center().y),
                        egui::Align2::LEFT_CENTER, name.as_str(),
                        FontId::proportional(13.0),
                        if is_active { tok.text } else { tok.dim });
                    if resp.clicked() { app.navigate_to(path.clone()); }
                }
            }

            ui.add_space(8.0);
            crate::ui::section_label(ui, tok, "THIS PC");
            let drives = app.drives.clone();
            for drive in &drives {
                drive_item(ui, tok, app, drive);
            }

            ui.add_space(8.0);
            crate::ui::section_label(ui, tok, "FOLDERS");
            show_folder_tree(ui, tok, app);
        });
}

fn dirs_path(kind: &str) -> std::path::PathBuf {
    match kind {
        "home" => dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\")),
        "download" => dirs::download_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\")),
        _ => std::path::PathBuf::from("C:\\"),
    }
}

fn sidebar_item(
    ui: &mut egui::Ui,
    tok: &Tokens,
    icon: char,
    label: &str,
    path: &std::path::Path,
    app: &mut FerroApp,
) {
    let is_active = app.main_pane.path == path;
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 28.0),
        Sense::click(),
    );
    let paint = ui.painter();

    let bg = if is_active {
        paint.rect_filled(rect, Rounding::same(6.0), tok.accent_soft);
        // Accent left bar
        let bar = egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height() - 8.0));
        let bar = bar.translate(Vec2::new(0.0, 4.0));
        paint.rect_filled(bar, Rounding::same(2.0), tok.accent);
        tok.accent
    } else if resp.hovered() {
        paint.rect_filled(rect, Rounding::same(6.0), tok.hover);
        tok.dim
    } else {
        tok.dim
    };

    let icon_color = if is_active { tok.accent } else { bg };
    paint.text(
        egui::pos2(rect.min.x + 14.0, rect.center().y),
        egui::Align2::CENTER_CENTER,
        icon.to_string(),
        FontId::proportional(16.0),
        icon_color,
    );
    paint.text(
        egui::pos2(rect.min.x + 26.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        if is_active { tok.text } else { tok.dim },
    );

    if resp.clicked() {
        app.navigate_to(path.to_path_buf());
    }
}

fn drive_item(ui: &mut egui::Ui, tok: &Tokens, app: &mut FerroApp, drive: &crate::fs::DriveInfo) {
    let is_active = app.main_pane.path.starts_with(&drive.path);

    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 50.0),
        Sense::click(),
    );
    if is_active {
        ui.painter().rect_filled(rect, Rounding::same(6.0), tok.accent_soft);
        let bar = egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height() - 8.0))
            .translate(Vec2::new(0.0, 4.0));
        ui.painter().rect_filled(bar, Rounding::same(2.0), tok.accent);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, Rounding::same(6.0), tok.hover);
    }

    let icon_color = if is_active { tok.accent } else { tok.dim };
    ui.painter().text(
        egui::pos2(rect.min.x + 14.0, rect.min.y + 14.0),
        egui::Align2::CENTER_CENTER,
        icons::STORAGE.to_string(),
        FontId::proportional(16.0),
        icon_color,
    );
    ui.painter().text(
        egui::pos2(rect.min.x + 26.0, rect.min.y + 10.0),
        egui::Align2::LEFT_CENTER,
        format!("{}:", drive.letter),
        FontId::proportional(13.0),
        if is_active { tok.text } else { tok.dim },
    );

    // 使用量バー
    if drive.total_bytes > 0 {
        let used = drive.total_bytes.saturating_sub(drive.free_bytes);
        let ratio = (used as f64 / drive.total_bytes as f64).clamp(0.0, 1.0) as f32;
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 6.0, rect.min.y + 30.0),
            Vec2::new(rect.width() - 12.0, 4.0),
        );
        ui.painter().rect_filled(bar_rect, Rounding::same(2.0), tok.border);
        let fill_w = bar_rect.width() * ratio;
        if fill_w > 0.0 {
            let fill = egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, 4.0));
            let bar_color = if ratio > 0.9 {
                egui::Color32::from_rgb(0xc8, 0x37, 0x2c)
            } else {
                tok.accent
            };
            ui.painter().rect_filled(fill, Rounding::same(2.0), bar_color);
        }
        let free_gb = drive.free_bytes as f64 / 1024.0_f64.powi(3);
        let total_gb = drive.total_bytes as f64 / 1024.0_f64.powi(3);
        let free_text = format!("{:.0} GB / {:.0} GB", free_gb, total_gb);
        ui.painter().text(
            egui::pos2(rect.min.x + 6.0, rect.min.y + 40.0),
            egui::Align2::LEFT_CENTER,
            free_text,
            FontId::monospace(10.0),
            tok.faint,
        );
    }

    if resp.clicked() {
        app.navigate_to(drive.path.clone());
    }
}

fn show_folder_tree(ui: &mut egui::Ui, tok: &Tokens, app: &mut FerroApp) {
    // Show ancestors of current path as tree nodes
    let current = app.main_pane.path.clone();
    let components: Vec<std::path::PathBuf> = {
        let mut v = Vec::new();
        let mut p = current.clone();
        loop {
            v.push(p.clone());
            if !p.pop() { break; }
        }
        v.reverse();
        v
    };

    for (depth, path) in components.iter().enumerate() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let is_current = *path == current;
        let indent = depth as f32 * 12.0;
        let (rect, resp) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), TREE_ROW_H),
            Sense::click(),
        );
        let paint = ui.painter();

        if is_current {
            paint.rect_filled(rect, Rounding::same(6.0), tok.accent_soft);
            let bar = egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height() - 8.0))
                .translate(Vec2::new(0.0, 4.0));
            paint.rect_filled(bar, Rounding::same(2.0), tok.accent);
        } else if resp.hovered() {
            paint.rect_filled(rect, Rounding::same(6.0), tok.hover);
        }

        let icon = if is_current { icons::FOLDER_OPEN } else { icons::FOLDER };
        let icon_color = if is_current {
            tok.accent
        } else {
            Color32::from_rgb(0xcf, 0x9b, 0x53)
        };
        let x = rect.min.x + indent + 14.0;
        paint.text(
            egui::pos2(x, rect.center().y),
            egui::Align2::CENTER_CENTER,
            icon.to_string(),
            FontId::proportional(15.0),
            icon_color,
        );
        paint.text(
            egui::pos2(x + 12.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &name,
            FontId::proportional(12.5),
            if is_current { tok.text } else { tok.dim },
        );

        if resp.clicked() && !is_current {
            app.navigate_to(path.clone());
        }
        let ctx_path = path.clone();
        resp.context_menu(|ui| {
            ui.set_min_width(180.0);
            if ui.button("クイックアクセスに追加").clicked() {
                app.pending_action = Some(ContextAction::AddQuickAccess(ctx_path));
                ui.close_menu();
            }
        });
    }
}

// ─── File list panel ─────────────────────────────────────────────────────────

fn show_list(app: &mut FerroApp, ctx: &egui::Context, tok: &Tokens) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(tok.list))
        .show(ctx, |ui| {
            // Column header
            let header_h = 32.0;
            let avail_w = ui.available_width();
            let col_widths = col_widths(avail_w);

            let header_rect = ui.allocate_exact_size(
                Vec2::new(avail_w, header_h),
                Sense::hover(),
            ).0;
            ui.painter()
                .rect_filled(header_rect, Rounding::ZERO, tok.bg);
            draw_header(ui, tok, header_rect, &col_widths);

            // Separator line
            ui.painter().line_segment(
                [
                    header_rect.left_bottom(),
                    header_rect.right_bottom(),
                ],
                Stroke::new(1.0, tok.border),
            );

            // File rows (filter hidden if needed)
            let show_hidden = app.show_hidden;
            let entries: Vec<_> = app.main_pane.entries.iter()
                .filter(|e| show_hidden || !e.is_hidden)
                .cloned()
                .collect();
            let selection = app.main_pane.selection.clone();

            // Background right-click (empty area)
            let bg_resp = ui.interact(
                ui.available_rect_before_wrap(),
                egui::Id::new("list_bg"),
                Sense::click(),
            );
            bg_resp.context_menu(|ui| {
                ui.set_min_width(180.0);
                ui.menu_button("新規作成", |ui| {
                    ui.set_min_width(260.0);
                    if ui.button("フォルダー").clicked() {
                        app.pending_action = Some(ContextAction::NewFolder);
                        ui.close_menu();
                    }
                    ui.separator();
                    for (label, filename) in NEW_FILE_TEMPLATES {
                        if ui.button(*label).clicked() {
                            app.pending_action = Some(ContextAction::NewFile(filename.to_string()));
                            ui.close_menu();
                        }
                    }
                });
                ui.separator();
                if ui.button("更新").clicked() {
                    app.pending_action = Some(ContextAction::Refresh);
                    ui.close_menu();
                }
            });

            ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_width(avail_w);
                for (i, entry) in entries.iter().enumerate() {
                    let is_selected = selection.contains(&i);
                    let row_rect = ui.allocate_exact_size(
                        Vec2::new(avail_w, ROW_H),
                        Sense::click(),
                    );
                    let (rect, resp) = row_rect;

                    // Row background
                    if is_selected {
                        ui.painter()
                            .rect_filled(rect, Rounding::ZERO, tok.accent_soft);
                        // Left accent bar
                        let bar = egui::Rect::from_min_size(
                            rect.min + Vec2::new(0.0, 4.0),
                            Vec2::new(2.0, rect.height() - 8.0),
                        );
                        ui.painter()
                            .rect_filled(bar, Rounding::same(2.0), tok.accent);
                    } else if resp.hovered() {
                        ui.painter()
                            .rect_filled(rect, Rounding::ZERO, tok.hover);
                    }

                    draw_row(ui, tok, &rect, entry, is_selected, &col_widths);

                    // Handle click
                    if resp.clicked() {
                        let multi = ui.input(|i| i.modifiers.ctrl || i.modifiers.shift);
                        if multi {
                            if is_selected {
                                app.main_pane.selection.remove(&i);
                            } else {
                                app.main_pane.selection.insert(i);
                            }
                        } else {
                            app.main_pane.selection.clear();
                            app.main_pane.selection.insert(i);
                        }
                    }
                    if resp.double_clicked() {
                        let path = entry.path.clone();
                        if entry.kind == EntryKind::Dir {
                            app.navigate_to(path);
                        } else {
                            let _ = open::that(&path);
                        }
                    }

                    // Context menu
                    let entry_path = entry.path.clone();
                    let is_dir = entry.kind == EntryKind::Dir;
                    let sel_paths: Vec<std::path::PathBuf> = if is_selected && app.main_pane.selection.len() > 1 {
                        app.main_pane.selection.iter()
                            .filter_map(|&j| entries.get(j).map(|e| e.path.clone()))
                            .collect()
                    } else {
                        vec![entry_path.clone()]
                    };
                    resp.context_menu(|ui| {
                        ui.set_min_width(180.0);
                        if ui.button("開く").clicked() {
                            app.pending_action = Some(ContextAction::Open(entry_path.clone()));
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("パスをコピー").clicked() {
                            app.pending_action = Some(ContextAction::CopyPath(entry_path.clone()));
                            ui.close_menu();
                        }
                        if is_dir {
                            if ui.button("ターミナルで開く").clicked() {
                                app.pending_action = Some(ContextAction::OpenTerminal(entry_path.clone()));
                                ui.close_menu();
                            }
                            if ui.button("クイックアクセスに追加").clicked() {
                                app.pending_action = Some(ContextAction::AddQuickAccess(entry_path.clone()));
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("名前の変更").clicked() {
                            app.pending_action = Some(ContextAction::Rename(entry_path.clone()));
                            ui.close_menu();
                        }
                        if ui.button(egui::RichText::new("削除").color(egui::Color32::from_rgb(0xc8, 0x37, 0x2c))).clicked() {
                            app.pending_action = Some(ContextAction::Delete(sel_paths.clone()));
                            ui.close_menu();
                        }
                    });
                }
            });
        });
}

fn col_widths(avail_w: f32) -> [f32; 4] {
    let fixed = 96.0 + 136.0 + 152.0;
    let flex = (avail_w - fixed).max(120.0);
    [flex, 96.0, 136.0, 152.0]
}

fn draw_header(
    ui: &mut egui::Ui,
    tok: &Tokens,
    header_rect: egui::Rect,
    col_widths: &[f32; 4],
) {
    let y = header_rect.center().y;
    let mut x = header_rect.min.x + 8.0;
    let headers = ["Name", "Size", "Type", "Modified"];
    for (i, &h) in headers.iter().enumerate() {
        let align = if i == 1 {
            egui::Align2::RIGHT_CENTER
        } else {
            egui::Align2::LEFT_CENTER
        };
        let tx = if i == 1 { x + col_widths[i] - 8.0 } else { x };
        ui.painter().text(
            egui::pos2(tx, y),
            align,
            h,
            FontId::proportional(10.5),
            tok.faint,
        );
        x += col_widths[i];
    }
}

fn draw_row(
    ui: &mut egui::Ui,
    tok: &Tokens,
    rect: &egui::Rect,
    entry: &crate::entry::Entry,
    is_selected: bool,
    col_widths: &[f32; 4],
) {
    let y = rect.center().y;
    let mut x = rect.min.x + 8.0;

    // Icon
    let icon_color = entry.icon_color(tok.accent);
    ui.painter().text(
        egui::pos2(x + 9.0, y),
        egui::Align2::CENTER_CENTER,
        entry.icon_char().to_string(),
        FontId::proportional(18.0),
        icon_color,
    );
    x += 22.0;

    // Name
    let text_color = if is_selected { tok.text } else if entry.is_hidden { tok.dim } else { tok.text };
    let name_width = col_widths[0] - 30.0;
    let name = truncate_str(&entry.name, name_width, 13.0);
    let name_galley = ui.fonts(|f| {
        f.layout_no_wrap(name.clone(), FontId::proportional(13.0), text_color)
    });
    let name_px_width = name_galley.rect.width();
    ui.painter().galley(egui::pos2(x, y - name_galley.rect.height() * 0.5), name_galley, text_color);
    // オンライン専用ファイルはクラウドバッジを表示
    if entry.is_cloud_only {
        let badge_x = (x + name_px_width + 4.0).min(rect.min.x + col_widths[0] - 20.0);
        ui.painter().text(
            egui::pos2(badge_x, y),
            egui::Align2::LEFT_CENTER,
            icons::CLOUD_SYNC.to_string(),
            FontId::proportional(13.0),
            Color32::from_rgb(0x00, 0x7a, 0xff),
        );
    }
    x = rect.min.x + col_widths[0];

    // Size (right-aligned)
    ui.painter().text(
        egui::pos2(x + col_widths[1] - 8.0, y),
        egui::Align2::RIGHT_CENTER,
        entry.size_display(),
        FontId::monospace(11.0),
        tok.dim,
    );
    x += col_widths[1];

    // Type
    ui.painter().text(
        egui::pos2(x + 4.0, y),
        egui::Align2::LEFT_CENTER,
        entry.type_label(),
        FontId::proportional(12.0),
        tok.dim,
    );
    x += col_widths[2];

    // Modified
    ui.painter().text(
        egui::pos2(x + 4.0, y),
        egui::Align2::LEFT_CENTER,
        entry.modified_display(),
        FontId::monospace(11.0),
        tok.dim,
    );
}

fn truncate_str(s: &str, _max_width: f32, _font_size: f32) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > 28 {
        format!("{}…", chars[..26].iter().collect::<String>())
    } else {
        s.to_owned()
    }
}
