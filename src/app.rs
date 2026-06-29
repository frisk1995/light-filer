use std::path::PathBuf;
use crossbeam_channel::{Receiver, Sender};
use egui::{Color32, FontId, Rounding, Sense, Stroke, Vec2};

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

use crate::{
    fonts,
    fs::{DriveInfo, FsMsg, ScanResult, list_drives, list_onedrive_paths, spawn_worker},
    icons,
    state::{PaneState, ViewMode},
    theme::{Accent, Theme, Tokens},
    ui,
};

pub enum ContextAction {
    Open(PathBuf),
    CopyPath(PathBuf),
    OpenTerminal(PathBuf),
    NewFolder,
    NewFile(String),  // default filename with extension
    Delete(Vec<PathBuf>),
    Rename(PathBuf),
    AddQuickAccess(PathBuf),
    RemoveQuickAccess(usize),
    Refresh,
}

pub struct FerroApp {
    // State
    pub main_pane: PaneState,
    pub view_mode: ViewMode,
    pub theme: Theme,
    pub accent: Accent,
    pub tokens: Tokens,
    pub search_text: String,
    pub preview_idx: Option<usize>,
    pub quick_access: Vec<(String, PathBuf)>,
    pub pending_action: Option<ContextAction>,
    pub rename_state: Option<(PathBuf, String)>,
    pub show_hidden: bool,
    pub drives: Vec<DriveInfo>,
    pub onedrive_paths: Vec<(String, PathBuf)>,

    // FS worker
    req_tx: Sender<FsMsg>,
    res_rx: Receiver<ScanResult>,
}

impl FerroApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::setup(&cc.egui_ctx);

        let start = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("C:\\"));

        let (req_tx, res_rx) = spawn_worker();

        let _ = req_tx.send(FsMsg::Request(start.clone()));

        let theme = Theme::Dark;
        let accent = Accent::Rust;

        let quick_access = {
            let saved = load_quick_access();
            if saved.is_empty() {
                vec![
                    ("Home".to_owned(), dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"))),
                    ("Downloads".to_owned(), dirs::download_dir().unwrap_or_else(|| PathBuf::from("C:\\"))),
                    ("Desktop".to_owned(), dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("C:\\"))),
                ]
            } else {
                saved
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
            rename_state: None,
            show_hidden: false,
            drives: list_drives(),
            onedrive_paths: list_onedrive_paths(),
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
            Some(ContextAction::Open(path)) => { let _ = open::that(&path); }
            Some(ContextAction::CopyPath(path)) => {
                ctx.output_mut(|o| o.copied_text = path.to_string_lossy().to_string());
            }
            Some(ContextAction::OpenTerminal(path)) => {
                let dir = if path.is_dir() { path.clone() } else {
                    path.parent().unwrap_or(&path).to_path_buf()
                };
                // Try Windows Terminal, fall back to PowerShell
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
                for p in &paths {
                    if p.is_dir() { let _ = std::fs::remove_dir_all(p); }
                    else { let _ = std::fs::remove_file(p); }
                }
                self.main_pane.selection.clear();
                self.refresh();
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
}

impl eframe::App for FerroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_pending_action(ctx);
        self.poll_results();

        let tok = self.tokens.clone();

        // Apply global visuals
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

        // ── Titlebar ──────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("titlebar")
            .frame(
                egui::Frame::none()
                    .fill(tok.titlebar)
                    .inner_margin(egui::Margin::symmetric(12.0, 0.0)),
            )
            .exact_height(38.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    // App mark
                    let mark_size = Vec2::splat(18.0);
                    let (mark_rect, _) = ui.allocate_exact_size(mark_size, Sense::hover());
                    ui.painter().rect_filled(
                        mark_rect,
                        Rounding::same(5.0),
                        tok.accent,
                    );
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
                        egui::RichText::new("0.4.2")
                            .font(FontId::monospace(10.0))
                            .color(tok.faint),
                    );


                    // Center: current path
                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                        ui.label(
                            egui::RichText::new(self.main_pane.path.to_string_lossy().as_ref())
                                .font(FontId::monospace(11.0))
                                .color(tok.faint),
                        );
                    });
                });
            });

        // ── Toolbar ───────────────────────────────────────────────────────────
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
                    // ── ナビゲーションボタン（左固定）──
                    let can_back = !self.main_pane.back_stack.is_empty();
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
                    ui.add_space(8.0);

                    // ── RTL で右側を埋め、パンくずは残り幅へ ──
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // View segment switcher（最右端）
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

                        // Theme toggle
                        let theme_icon = if self.theme == Theme::Dark { icons::LIGHT_MODE } else { icons::DARK_MODE };
                        if ui::nav_button(ui, &tok, theme_icon, true).clicked() {
                            self.theme = if self.theme == Theme::Dark { Theme::Light } else { Theme::Dark };
                            self.update_tokens();
                        }

                        // Hidden files toggle
                        let vis_icon = if self.show_hidden { icons::VISIBILITY } else { icons::VISIBILITY_OFF };
                        let vis_tip = if self.show_hidden { "隠しファイルを非表示" } else { "隠しファイルを表示" };
                        if ui::nav_button(ui, &tok, vis_icon, true).on_hover_text(vis_tip).clicked() {
                            self.show_hidden = !self.show_hidden;
                            self.main_pane.selection.clear();
                        }
                        ui.add_space(8.0);

                        // ── パンくず（RTL 内で残り幅を左から埋める）──
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

        // ── Status bar ────────────────────────────────────────────────────────
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

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let speed = if let Some(ms) = self.main_pane.scan_time_ms {
                                let free = self
                                    .main_pane
                                    .free_bytes
                                    .map(|b| format!("{:.0} GB free", b as f64 / 1024.0_f64.powi(3)))
                                    .unwrap_or_default();
                                format!(
                                    "{} {}indexed in {} ms",
                                    free,
                                    icons::BOLT,
                                    ms
                                )
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

        // ── Main content area ─────────────────────────────────────────────────
        match self.view_mode {
            ViewMode::Explorer => ui::explorer::show(self, ctx),
            ViewMode::Modern => ui::modern::show(self, ctx),
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
    if !base.exists() {
        return base;
    }
    for i in 2u32.. {
        let candidate = make(&format!(" ({})", i));
        if !candidate.exists() {
            return candidate;
        }
    }
    base
}
