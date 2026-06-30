use std::path::PathBuf;
use crossbeam_channel::{Receiver, Sender};
use egui::{Color32, FontId, Rounding, Sense, Stroke, Vec2};

use crate::{
    fonts,
    fs::{DriveInfo, FsMsg, ScanResult, list_drives, list_onedrive_paths, spawn_worker},
    icons,
    settings::{self, FontChoice},
    state::{PaneState, ViewMode},
    theme::{Accent, Theme, Tokens},
    ui,
};

fn config_path() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("filox");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("quick_access.txt"))
}

fn load_quick_access() -> Vec<(String, PathBuf)> {
    let path = match config_path() { Some(p) => p, None => return Vec::new() };
    let text = match std::fs::read_to_string(&path) { Ok(t) => t, Err(_) => return Vec::new() };
    text.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let name = parts.next()?.to_owned();
            let p = PathBuf::from(parts.next()?);
            Some((name, p))
        })
        .collect()
}

fn save_quick_access(qa: &[(String, PathBuf)]) {
    let Some(path) = config_path() else { return };
    let text: String = qa.iter()
        .map(|(n, p)| format!("{}\t{}\n", n, p.display()))
        .collect();
    let _ = std::fs::write(path, text);
}

pub enum ContextAction {
    Open(PathBuf),
    CopyPath(PathBuf),
    OpenTerminal(PathBuf),
    NewFolder,
    NewFile(String),
    Delete(Vec<PathBuf>),
    Rename(PathBuf),
    AddQuickAccess(PathBuf),
    RemoveQuickAccess(usize),
    Refresh,
}

pub struct FerroApp {
    pub main_pane: PaneState,
    pub view_mode: ViewMode,
    pub theme: Theme,
    pub accent: Accent,
    pub tokens: Tokens,
    pub search_text: String,
    pub preview_idx: Option<usize>,
    pub quick_access: Vec<(String, PathBuf)>,
    pub pending_action: Option<ContextAction>,
    pub delete_confirm: Option<Vec<PathBuf>>,
    pub rename_state: Option<(PathBuf, String)>,
    pub path_input_open: bool,
    pub path_input_text: String,
    pub path_input_error: bool,
    pub show_hidden: bool,
    pub drives: Vec<DriveInfo>,
    pub onedrive_paths: Vec<(String, PathBuf)>,
    pub settings_open: bool,
    pub font_choice: FontChoice,
    pub name_col_extra: f32,
    pub file_open_status: Option<(String, std::time::Instant)>,

    req_tx: Sender<FsMsg>,
    res_rx: Receiver<ScanResult>,
}

impl FerroApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let saved = settings::load();

        fonts::setup(&cc.egui_ctx, &saved.font);

        let start = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("C:\\"));

        let (req_tx, res_rx) = spawn_worker();
        let _ = req_tx.send(FsMsg::Request(start.clone()));

        let theme = if saved.theme_dark { Theme::Dark } else { Theme::Light };
        let accent = Accent::Rust;

        let quick_access = {
            let saved_qa = load_quick_access();
            if saved_qa.is_empty() {
                vec![
                    ("Home".to_owned(),      dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"))),
                    ("Downloads".to_owned(), dirs::download_dir().unwrap_or_else(|| PathBuf::from("C:\\"))),
                    ("Desktop".to_owned(),   dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("C:\\"))),
                ]
            } else {
                saved_qa
            }
        };

        Self {
            main_pane: PaneState::new(start),
            view_mode: ViewMode::Explorer,
            theme,
            accent,
            tokens: Tokens::new(theme, accent),
            search_text: String::new(),
            preview_idx: None,
            quick_access,
            pending_action: None,
            delete_confirm: None,
            rename_state: None,
            path_input_open: false,
            path_input_text: String::new(),
            path_input_error: false,
            show_hidden: saved.show_hidden,
            drives: list_drives(),
            onedrive_paths: list_onedrive_paths(),
            settings_open: false,
            font_choice: saved.font,
            name_col_extra: 0.0,
            file_open_status: None,
            req_tx,
            res_rx,
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        let old = self.main_pane.path.clone();
        self.main_pane.back_stack.push(old);
        self.main_pane.forward_stack.clear();
        self.main_pane.path = path.clone();
        self.main_pane.loading = true;
        self.main_pane.selection.clear();
        self.preview_idx = None;
        self.search_text.clear();
        let _ = self.req_tx.send(FsMsg::Request(path));
    }

    pub fn navigate_back(&mut self) {
        if let Some(prev) = self.main_pane.back_stack.pop() {
            let current = self.main_pane.path.clone();
            self.main_pane.forward_stack.push(current);
            self.main_pane.path = prev.clone();
            self.main_pane.loading = true;
            self.main_pane.selection.clear();
            let _ = self.req_tx.send(FsMsg::Request(prev));
        }
    }

    pub fn navigate_forward(&mut self) {
        if let Some(next) = self.main_pane.forward_stack.pop() {
            let current = self.main_pane.path.clone();
            self.main_pane.back_stack.push(current);
            self.main_pane.path = next.clone();
            self.main_pane.loading = true;
            self.main_pane.selection.clear();
            let _ = self.req_tx.send(FsMsg::Request(next));
        }
    }

    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.main_pane.path.parent().map(|p| p.to_path_buf()) {
            self.navigate_to(parent);
        }
    }

    pub fn refresh(&mut self) {
        let path = self.main_pane.path.clone();
        self.main_pane.loading = true;
        let _ = self.req_tx.send(FsMsg::Request(path));
    }

    pub fn navigate_left(&mut self, path: PathBuf) {
        let old = self.main_pane.path.clone();
        self.main_pane.back_stack.push(old);
        self.main_pane.forward_stack.clear();
        self.main_pane.path = path.clone();
        self.main_pane.loading = true;
        self.main_pane.selection.clear();
        let _ = self.req_tx.send(FsMsg::Request(path));
    }

    pub fn process_pending_action(&mut self, ctx: &egui::Context) {
        let action = self.pending_action.take();
        match action {
            None => {}
            Some(ContextAction::Open(path)) => {
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                let _ = open::that(&path);
                self.file_open_status = Some((name, std::time::Instant::now()));
            }
            Some(ContextAction::CopyPath(path)) => {
                ctx.output_mut(|o| o.copied_text = path.to_string_lossy().to_string());
            }
            Some(ContextAction::OpenTerminal(path)) => {
                let dir = if path.is_dir() { path.clone() } else {
                    path.parent().unwrap_or(&path).to_path_buf()
                };
                if std::process::Command::new("wt")
                    .args(["-d", &dir.to_string_lossy()])
                    .spawn()
                    .is_err()
                {
                    let _ = std::process::Command::new("powershell")
                        .args(["-NoExit", "-Command", &format!("cd '{}'", dir.display())])
                        .spawn();
                }
            }
            Some(ContextAction::NewFolder) => {
                let base = "新しいフォルダー";
                let target = unique_path(&self.main_pane.path, base, "");
                let _ = std::fs::create_dir(&target);
                self.refresh();
                self.rename_state = Some((target.clone(), base.to_owned()));
            }
            Some(ContextAction::NewFile(filename)) => {
                let (stem, ext) = split_stem_ext(&filename);
                let target = unique_path(&self.main_pane.path, &stem, &ext);
                let content: &[u8] = if ext == "rtf" { b"{\\rtf1}" } else { b"" };
                let _ = std::fs::write(&target, content);
                self.refresh();
                let display_name = target.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(filename);
                self.rename_state = Some((target, display_name));
            }
            Some(ContextAction::Delete(paths)) => {
                self.delete_confirm = Some(paths);
            }
            Some(ContextAction::Rename(path)) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                self.rename_state = Some((path, name));
            }
            Some(ContextAction::AddQuickAccess(path)) => {
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                if !self.quick_access.iter().any(|(_, p)| p == &path) {
                    self.quick_access.push((name, path));
                    save_quick_access(&self.quick_access);
                }
            }
            Some(ContextAction::RemoveQuickAccess(idx)) => {
                if idx < self.quick_access.len() {
                    self.quick_access.remove(idx);
                    save_quick_access(&self.quick_access);
                }
            }
            Some(ContextAction::Refresh) => { self.refresh(); }
        }
    }

    fn poll_results(&mut self) {
        while let Ok(result) = self.res_rx.try_recv() {
            if result.path == self.main_pane.path {
                self.main_pane.entries = result.entries;
                self.main_pane.scan_time_ms = Some(result.elapsed_ms);
                self.main_pane.free_bytes = result.free_bytes;
                self.main_pane.loading = false;
            }
        }
    }

    fn update_tokens(&mut self) {
        self.tokens = Tokens::new(self.theme, self.accent);
    }

    pub fn show_delete_confirm(&mut self, ctx: &egui::Context) {
        if self.delete_confirm.is_none() { return; }

        let paths = self.delete_confirm.as_ref().unwrap();
        let count = paths.len();
        let label = if count == 1 {
            let name = paths[0].file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            format!("「{}」を削除しますか？", name)
        } else {
            format!("{}個のアイテムを削除しますか？", count)
        };

        let mut confirmed = false;
        let mut cancelled = false;

        egui::Window::new("削除の確認")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(320.0);
                ui.add_space(4.0);
                ui.label(egui::RichText::new(&label).size(14.0));
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("この操作は元に戻せません。")
                        .size(12.0)
                        .color(self.tokens.dim),
                );
                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(
                        egui::RichText::new("削除").color(egui::Color32::from_rgb(0xc8, 0x37, 0x2c))
                    ).clicked() {
                        confirmed = true;
                    }
                    ui.add_space(8.0);
                    if ui.button("キャンセル").clicked() {
                        cancelled = true;
                    }
                });
                ui.add_space(4.0);
            });

        if confirmed {
            let paths = self.delete_confirm.take().unwrap();
            for p in &paths {
                if p.is_dir() { let _ = std::fs::remove_dir_all(p); }
                else { let _ = std::fs::remove_file(p); }
            }
            self.main_pane.selection.clear();
            self.refresh();
        } else if cancelled {
            self.delete_confirm = None;
        }
    }

    pub fn open_path_input(&mut self) {
        self.path_input_text = self.main_pane.path.to_string_lossy().to_string();
        self.path_input_error = false;
        self.path_input_open = true;
    }

    pub fn show_path_input(&mut self, ctx: &egui::Context) {
        if !self.path_input_open { return; }

        let mut submitted = false;
        let mut cancelled = false;

        egui::Window::new("パスを開く")
            .collapsible(false)
            .resizable(false)
            .fixed_size([360.0, 80.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, -60.0])
            .show(ctx, |ui| {
                ui.set_min_width(340.0);
                ui.add_space(4.0);

                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.path_input_text)
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::monospace(13.0))
                        .hint_text("C:\\Users\\..."),
                );
                resp.request_focus();

                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submitted = true;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancelled = true;
                }

                if self.path_input_error {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("パスが見つかりません")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(0xc8, 0x37, 0x2c)),
                    );
                }

                ui.add_space(8.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("開く").strong()).clicked() {
                        submitted = true;
                    }
                    ui.add_space(8.0);
                    if ui.button("キャンセル").clicked() {
                        cancelled = true;
                    }
                });
                ui.add_space(4.0);
            });

        if submitted {
            let p = PathBuf::from(self.path_input_text.trim());
            if p.is_dir() {
                self.path_input_open = false;
                self.navigate_to(p);
            } else if p.is_file() {
                self.path_input_open = false;
                let _ = open::that(&p);
            } else {
                self.path_input_error = true;
            }
        } else if cancelled {
            self.path_input_open = false;
        }
    }

    pub fn show_settings_panel(&mut self, ctx: &egui::Context) {
        if !self.settings_open { return; }

        let tok = self.tokens.clone();
        let mut close = false;
        let mut font_changed = false;
        let mut changed = false;

        egui::Window::new("設定")
            .collapsible(false)
            .resizable(false)
            .fixed_size([220.0, 340.0])
            .anchor(egui::Align2::RIGHT_TOP, [-12.0, 84.0])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(tok.bg)
                    .stroke(egui::Stroke::new(1.0, tok.border))
                    .rounding(egui::Rounding::same(10.0)),
            )
            .show(ctx, |ui| {
                // ── ヘッダー ────────────────────────────────
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("設定").strong().size(14.0).color(tok.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(egui::RichText::new("✕").color(tok.dim)).clicked() {
                            close = true;
                        }
                    });
                });
                ui.separator();
                ui.add_space(6.0);

                // ── テーマ ──────────────────────────────────
                ui.label(egui::RichText::new("テーマ").size(11.0).color(tok.faint));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let is_dark = self.theme == Theme::Dark;
                    if ui.selectable_label(is_dark, "ダーク").clicked() && !is_dark {
                        self.theme = Theme::Dark;
                        self.update_tokens();
                        changed = true;
                    }
                    if ui.selectable_label(!is_dark, "ライト").clicked() && is_dark {
                        self.theme = Theme::Light;
                        self.update_tokens();
                        changed = true;
                    }
                });

                ui.add_space(12.0);

                // ── 表示 ────────────────────────────────────
                ui.label(egui::RichText::new("表示").size(11.0).color(tok.faint));
                ui.add_space(4.0);
                if ui.checkbox(&mut self.show_hidden, "隠しファイルを表示").changed() {
                    self.main_pane.selection.clear();
                    changed = true;
                }

                ui.add_space(12.0);

                // ── フォント ────────────────────────────────
                ui.label(egui::RichText::new("フォント").size(11.0).color(tok.faint));
                ui.add_space(4.0);
                for choice in FontChoice::all() {
                    let selected = self.font_choice == choice;
                    if ui.selectable_label(selected, choice.label()).clicked() && !selected {
                        self.font_choice = choice;
                        font_changed = true;
                        changed = true;
                    }
                }

                ui.add_space(6.0);
            });

        if font_changed {
            fonts::setup(ctx, &self.font_choice);
        }
        if changed {
            settings::save(
                self.theme == Theme::Dark,
                self.show_hidden,
                &self.font_choice,
            );
        }
        if close {
            self.settings_open = false;
        }
    }
}

impl eframe::App for FerroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Ctrl+L でパス入力ダイアログを開く
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::L)) {
            self.open_path_input();
        }

        // マウスサイドボタン（戻る / 進む）
        if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra1)) {
            self.navigate_back();
        }
        if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra2)) {
            self.navigate_forward();
        }

        self.process_pending_action(ctx);
        self.show_delete_confirm(ctx);
        self.show_path_input(ctx);
        self.show_settings_panel(ctx);
        self.poll_results();

        // ファイルオープン中スピナー: 5秒後に自動消去、アニメーションのため再描画を要求
        if let Some((_, t)) = &self.file_open_status {
            if t.elapsed().as_secs_f32() > 5.0 {
                self.file_open_status = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
        }

        let tok = self.tokens.clone();

        let mut visuals = if self.theme == Theme::Dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.window_fill = tok.bg;
        visuals.panel_fill = tok.bg;
        visuals.extreme_bg_color = tok.list;
        visuals.widgets.inactive.bg_fill = tok.elev;
        visuals.widgets.hovered.bg_fill = tok.hover;
        visuals.selection.bg_fill = tok.accent_soft;
        visuals.selection.stroke = Stroke::new(1.0, tok.accent);
        visuals.override_text_color = Some(tok.text);
        ctx.set_visuals(visuals);

        // ── Titlebar ───────────────────────────────────────────────────────
        egui::TopBottomPanel::top("titlebar")
            .frame(
                egui::Frame::none()
                    .fill(tok.titlebar)
                    .inner_margin(egui::Margin::symmetric(12.0, 0.0)),
            )
            .exact_height(38.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let mark_size = Vec2::splat(18.0);
                    let (mark_rect, _) = ui.allocate_exact_size(mark_size, Sense::hover());
                    ui.painter().rect_filled(mark_rect, Rounding::same(5.0), tok.accent);
                    ui.painter().text(
                        mark_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "f",
                        FontId::new(11.0, egui::FontFamily::Monospace),
                        Color32::WHITE,
                    );

                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("filox")
                            .font(FontId::new(13.0, egui::FontFamily::Monospace))
                            .strong()
                            .color(tok.text),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("0.4.7")
                            .font(FontId::monospace(10.0))
                            .color(tok.faint),
                    );

                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                        ui.label(
                            egui::RichText::new(self.main_pane.path.to_string_lossy().as_ref())
                                .font(FontId::monospace(11.0))
                                .color(tok.faint),
                        );
                    });
                });
            });

        // ── Toolbar ────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::none()
                    .fill(tok.bg)
                    .stroke(Stroke::new(1.0, tok.border))
                    .inner_margin(egui::Margin::symmetric(12.0, 0.0)),
            )
            .exact_height(46.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let can_back    = !self.main_pane.back_stack.is_empty();
                    let can_forward = !self.main_pane.forward_stack.is_empty();
                    if ui::nav_button(ui, &tok, icons::CHEVRON_LEFT, can_back).clicked() {
                        self.navigate_back();
                    }
                    if ui::nav_button(ui, &tok, icons::CHEVRON_RIGHT, can_forward).clicked() {
                        self.navigate_forward();
                    }
                    if ui::nav_button(ui, &tok, icons::ARROW_UPWARD, true).clicked() {
                        self.navigate_up();
                    }
                    if ui::nav_button(ui, &tok, icons::REFRESH, true).clicked() {
                        self.refresh();
                    }
                    if ui::nav_button(ui, &tok, icons::DRIVE_FILE_MOVE, true)
                        .on_hover_text("パスを開く (Ctrl+L)")
                        .clicked()
                    {
                        self.open_path_input();
                    }
                    ui.add_space(8.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 設定ボタン（最右端）
                        let settings_active = self.settings_open;
                        if ui::nav_button_active(ui, &tok, icons::SETTINGS_ICON, settings_active)
                            .on_hover_text("設定")
                            .clicked()
                        {
                            self.settings_open = !self.settings_open;
                        }
                        ui.add_space(4.0);

                        // View segment switcher
                        egui::Frame::none()
                            .fill(tok.elev)
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0, tok.border))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    if ui::segment_button(ui, &tok, icons::REORDER,
                                        self.view_mode == ViewMode::Explorer).clicked() {
                                        self.view_mode = ViewMode::Explorer;
                                    }
                                    if ui::segment_button(ui, &tok, icons::GRID_VIEW,
                                        self.view_mode == ViewMode::Modern).clicked() {
                                        self.view_mode = ViewMode::Modern;
                                    }
                                });
                            });
                        ui.add_space(8.0);

                        // Search
                        egui::Frame::none()
                            .fill(tok.elev)
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0, tok.border))
                            .show(ui, |ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.label(egui::RichText::new(icons::SEARCH.to_string())
                                        .font(FontId::proportional(16.0)).color(tok.faint));
                                    ui.add(egui::TextEdit::singleline(&mut self.search_text)
                                        .desired_width(150.0).frame(false)
                                        .hint_text("Search…").text_color(tok.text));
                                });
                            });
                        ui.add_space(8.0);

                        // ── パンくず ──
                        let avail_w = ui.available_width().max(40.0);
                        ui.allocate_ui_with_layout(
                            Vec2::new(avail_w, 38.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let path = self.main_pane.path.clone();
                                let parts: Vec<_> = path.components().collect();
                                egui::Frame::none()
                                    .fill(tok.elev)
                                    .rounding(Rounding::same(8.0))
                                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.set_max_width(avail_w - 4.0);
                                        ui.horizontal_centered(|ui| {
                                            let skip = if parts.len() > 4 {
                                                ui.label(egui::RichText::new("…")
                                                    .font(FontId::monospace(12.0)).color(tok.faint));
                                                ui.label(egui::RichText::new(icons::CHEVRON_RIGHT.to_string())
                                                    .font(FontId::proportional(15.0)).color(tok.faint));
                                                parts.len() - 4
                                            } else { 0 };

                                            let mut cum = PathBuf::new();
                                            for (i, comp) in parts.iter().enumerate() {
                                                cum.push(comp);
                                                if i < skip { continue; }
                                                let label = match comp {
                                                    std::path::Component::RootDir =>
                                                        cum.to_string_lossy().to_string(),
                                                    std::path::Component::Normal(n) =>
                                                        n.to_string_lossy().to_string(),
                                                    _ => continue,
                                                };
                                                let dest = cum.clone();
                                                let resp = ui.add(egui::Label::new(
                                                    egui::RichText::new(&label)
                                                        .font(FontId::monospace(12.0))
                                                        .color(if i == parts.len()-1 { tok.text } else { tok.dim }),
                                                ).sense(Sense::click()));
                                                if resp.clicked() { self.navigate_to(dest); }
                                                if i < parts.len() - 1 {
                                                    ui.label(egui::RichText::new(icons::CHEVRON_RIGHT.to_string())
                                                        .font(FontId::proportional(15.0)).color(tok.faint));
                                                }
                                            }
                                        });
                                    });
                            },
                        );
                    });
                });
            });

        // ── Status bar ────────────────────────────────────────────────────
        {
            let tok2 = tok.clone();
            egui::TopBottomPanel::bottom("statusbar")
                .frame(
                    egui::Frame::none()
                        .fill(tok2.titlebar)
                        .stroke(Stroke::new(1.0, tok2.border))
                        .inner_margin(egui::Margin::symmetric(12.0, 0.0)),
                )
                .exact_height(27.0)
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            egui::RichText::new(self.main_pane.status_text())
                                .font(FontId::monospace(11.0))
                                .color(tok2.dim),
                        );

                        // ファイルオープン中スピナー
                        if let Some((name, _)) = &self.file_open_status {
                            ui.add_space(12.0);
                            ui.add(egui::Spinner::new().size(13.0).color(tok2.accent));
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!("「{}」を開いています...", name))
                                    .font(FontId::monospace(11.0))
                                    .color(tok2.dim),
                            );
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let speed = if let Some(ms) = self.main_pane.scan_time_ms {
                                let free = self.main_pane.free_bytes
                                    .map(|b| format!("{:.0} GB free", b as f64 / 1024.0_f64.powi(3)))
                                    .unwrap_or_default();
                                format!("{} {}indexed in {} ms", free, icons::BOLT, ms)
                            } else {
                                "Scanning…".to_owned()
                            };
                            ui.label(
                                egui::RichText::new(speed)
                                    .font(FontId::monospace(11.0))
                                    .color(tok2.dim),
                            );
                        });
                    });
                });
        }

        // ── Main content ──────────────────────────────────────────────────
        match self.view_mode {
            ViewMode::Explorer => ui::explorer::show(self, ctx),
            ViewMode::Modern   => ui::modern::show(self, ctx),
        }

        if self.main_pane.loading {
            ctx.request_repaint();
        }
    }
}

fn split_stem_ext(filename: &str) -> (String, String) {
    match filename.rfind('.') {
        Some(i) if i > 0 => (filename[..i].to_owned(), filename[i + 1..].to_owned()),
        _ => (filename.to_owned(), String::new()),
    }
}

fn unique_path(dir: &std::path::Path, stem: &str, ext: &str) -> PathBuf {
    let make = |suffix: &str| -> PathBuf {
        if ext.is_empty() {
            dir.join(format!("{}{}", stem, suffix))
        } else {
            dir.join(format!("{}{}.{}", stem, suffix, ext))
        }
    };
    let base = make("");
    if !base.exists() { return base; }
    for i in 2u32.. {
        let candidate = make(&format!(" ({})", i));
        if !candidate.exists() { return candidate; }
    }
    base
}
