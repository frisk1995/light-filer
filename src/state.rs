use std::collections::HashSet;
use std::path::PathBuf;
use crate::entry::Entry;


#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    Explorer,
    Modern,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortColumn {
    Name,
    Size,
    Kind,
    Modified,
}

#[derive(Clone)]
pub struct Sort {
    pub column: SortColumn,
    pub ascending: bool,
}

impl Default for Sort {
    fn default() -> Self {
        Self { column: SortColumn::Name, ascending: true }
    }
}

#[derive(Clone)]
pub struct PaneState {
    pub path: PathBuf,
    pub back_stack: Vec<PathBuf>,
    pub forward_stack: Vec<PathBuf>,
    pub entries: Vec<Entry>,
    pub selection: HashSet<usize>,
    pub cursor: usize,
    pub tagged: HashSet<usize>,
    pub sort: Sort,
    pub scan_time_ms: Option<u64>,
    pub free_bytes: Option<u64>,
    pub loading: bool,
}

impl PaneState {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
            entries: Vec::new(),
            selection: HashSet::new(),
            cursor: 0,
            tagged: HashSet::new(),
            sort: Sort::default(),
            scan_time_ms: None,
            free_bytes: None,
            loading: true,
        }
    }

    pub fn total_size_selected(&self) -> u64 {
        self.selection
            .iter()
            .filter_map(|&i| self.entries.get(i)?.size)
            .sum()
    }

    pub fn status_text(&self) -> String {
        let n = self.entries.len();
        let sel = self.selection.len();
        let tagged = self.tagged.len();

        let mut parts = vec![format!("{n} items")];
        if sel > 0 {
            let sz = self.total_size_selected();
            parts.push(format!("{sel} selected · {}", fmt_size(sz)));
        }
        if tagged > 0 {
            let sz: u64 = self
                .tagged
                .iter()
                .filter_map(|&i| self.entries.get(i)?.size)
                .sum();
            parts.push(format!("{tagged} tagged · {}", fmt_size(sz)));
        }
        parts.join(" · ")
    }
}

fn fmt_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
