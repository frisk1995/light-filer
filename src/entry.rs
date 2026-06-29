use std::path::PathBuf;
use chrono::{DateTime, Local};
use egui::Color32;
use crate::icons;

#[derive(Clone, PartialEq)]
pub enum EntryKind {
    Dir,
    File,
    Symlink,
}

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size: Option<u64>,
    pub modified: Option<DateTime<Local>>,
    pub is_hidden: bool,
    pub is_cloud_only: bool,
}

impl Entry {
    pub fn icon_char(&self) -> char {
        match &self.kind {
            EntryKind::Dir => icons::FOLDER,
            EntryKind::File => self.file_icon(),
            EntryKind::Symlink => icons::DESCRIPTION,
        }
    }

    pub fn icon_color(&self, accent: Color32) -> Color32 {
        match &self.kind {
            EntryKind::Dir if self.is_hidden => Color32::from_rgb(0x6f, 0x76, 0x7b),
            EntryKind::Dir => Color32::from_rgb(0xcf, 0x9b, 0x53),
            EntryKind::File => self.file_icon_color(accent),
            EntryKind::Symlink => Color32::from_rgb(0x8a, 0x90, 0x95),
        }
    }

    fn file_icon(&self) -> char {
        let ext = self
            .name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "rs" => icons::CODE,
            "toml" => icons::SETTINGS_ICON,
            "lock" => icons::LOCK,
            "md" | "markdown" => icons::ARTICLE,
            _ if self.name.starts_with("LICENSE") => icons::GAVEL,
            _ => icons::DESCRIPTION,
        }
    }

    fn file_icon_color(&self, accent: Color32) -> Color32 {
        let ext = self
            .name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "rs" => accent,
            "toml" => Color32::from_rgb(0x8a, 0xa8, 0x61),
            "lock" => Color32::from_rgb(0x8a, 0x90, 0x95),
            "md" | "markdown" => Color32::from_rgb(0x5f, 0x9e, 0x95),
            _ => Color32::from_rgb(0x8a, 0x90, 0x95),
        }
    }

    pub fn type_label(&self) -> &'static str {
        match &self.kind {
            EntryKind::Dir => "Folder",
            EntryKind::File => {
                let ext = self.name.rsplit('.').next().unwrap_or("");
                match ext.to_lowercase().as_str() {
                    "rs" => "Rust Source",
                    "toml" => "TOML Config",
                    "lock" => "Lock File",
                    "md" | "markdown" => "Markdown",
                    "txt" => "Text File",
                    "json" => "JSON",
                    "yaml" | "yml" => "YAML",
                    _ => "File",
                }
            }
            EntryKind::Symlink => "Symlink",
        }
    }

    pub fn size_display(&self) -> String {
        match self.size {
            None => String::new(),
            Some(0) => "0 B".to_owned(),
            Some(b) if b < 1024 => format!("{} B", b),
            Some(b) if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
            Some(b) if b < 1024 * 1024 * 1024 => {
                format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
            }
            Some(b) => format!("{:.1} GB", b as f64 / (1024.0 * 1024.0 * 1024.0)),
        }
    }

    pub fn modified_display(&self) -> String {
        self.modified
            .map(|dt| dt.format("%Y/%m/%d %H:%M").to_string())
            .unwrap_or_default()
    }
}
