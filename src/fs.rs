use std::path::{Path, PathBuf};
use std::time::Instant;
use crossbeam_channel::{Receiver, Sender};
use crate::entry::{Entry, EntryKind};

#[derive(Clone)]
pub struct DriveInfo {
    pub letter: char,
    pub path: PathBuf,
    pub free_bytes: u64,
    pub total_bytes: u64,
}

/// OneDrive フォルダを環境変数から検出して返す
pub fn list_onedrive_paths() -> Vec<(String, std::path::PathBuf)> {
    let mut paths = Vec::new();
    // 個人用 OneDrive
    if let Ok(p) = std::env::var("OneDrive") {
        let path = std::path::PathBuf::from(&p);
        if path.exists() {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "OneDrive".to_owned());
            paths.push((name, path));
        }
    }
    // ビジネス用 OneDrive（個人と別パスの場合のみ追加）
    if let Ok(p) = std::env::var("OneDriveCommercial") {
        let path = std::path::PathBuf::from(&p);
        if path.exists() && !paths.iter().any(|(_, existing)| existing == &path) {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "OneDrive for Business".to_owned());
            paths.push((name, path));
        }
    }
    paths
}

pub fn list_drives() -> Vec<DriveInfo> {
    #[cfg(windows)]
    {
        let mask = unsafe { get_logical_drives() };
        let mut drives = Vec::new();
        for i in 0u32..26 {
            if mask & (1 << i) == 0 { continue; }
            let letter = (b'A' + i as u8) as char;
            let path = PathBuf::from(format!("{}:\\", letter));
            let (free_bytes, total_bytes) = drive_space(&path).unwrap_or((0, 0));
            drives.push(DriveInfo { letter, path, free_bytes, total_bytes });
        }
        drives
    }
    #[cfg(not(windows))]
    {
        vec![DriveInfo {
            letter: '/',
            path: PathBuf::from("/"),
            free_bytes: 0,
            total_bytes: 0,
        }]
    }
}

#[cfg(windows)]
unsafe fn get_logical_drives() -> u32 {
    extern "system" { fn GetLogicalDrives() -> u32; }
    GetLogicalDrives()
}

fn drive_space(path: &Path) -> Option<(u64, u64)> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut free: u64 = 0;
        let mut total: u64 = 0;
        let ok = unsafe { get_disk_free_space(wide.as_ptr(), &mut free, &mut total) };
        if ok { Some((free, total)) } else { None }
    }
    #[cfg(not(windows))]
    { let _ = path; None }
}

#[cfg(windows)]
unsafe fn get_disk_free_space(path: *const u16, free: &mut u64, total: &mut u64) -> bool {
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }
    let mut total_free: u64 = 0;
    GetDiskFreeSpaceExW(path, free, total, &mut total_free) != 0
}

pub enum FsMsg {
    Request(PathBuf),
}

pub struct ScanResult {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
    pub elapsed_ms: u64,
    pub free_bytes: Option<u64>,
}

pub fn spawn_worker() -> (Sender<FsMsg>, Receiver<ScanResult>) {
    let (req_tx, req_rx) = crossbeam_channel::unbounded::<FsMsg>();
    let (res_tx, res_rx) = crossbeam_channel::unbounded::<ScanResult>();

    std::thread::spawn(move || {
        for msg in req_rx {
            match msg {
                FsMsg::Request(path) => {
                    let t = Instant::now();
                    // panic を捕捉してスレッドを生かし続ける
                    let entries = std::panic::catch_unwind(|| scan_dir(&path))
                        .unwrap_or_default();
                    let elapsed_ms = t.elapsed().as_millis() as u64;
                    let free_bytes = free_space(&path);
                    let _ = res_tx.send(ScanResult {
                        path,
                        entries,
                        elapsed_ms,
                        free_bytes,
                    });
                }
            }
        }
    });

    (req_tx, res_rx)
}

// OneDrive オンライン専用ファイルの Windows ファイル属性フラグ
#[cfg(windows)]
const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x0004_0000;
#[cfg(windows)]
const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;

fn is_cloud_only_file(meta: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let attrs = meta.file_attributes();
        (attrs & FILE_ATTRIBUTE_RECALL_ON_OPEN) != 0
            || (attrs & FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS) != 0
    }
    #[cfg(not(windows))]
    { let _ = meta; false }
}

fn scan_dir(path: &Path) -> Vec<Entry> {
    let Ok(rd) = std::fs::read_dir(path) else {
        return Vec::new();
    };

    let mut entries: Vec<Entry> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let entry_path = e.path();
            // symlink_metadata はリパースポイントを辿らないため
            // OneDrive クラウドプレースホルダーを安全に読める
            let meta = std::fs::symlink_metadata(&entry_path).ok()?;
            let is_cloud_only = is_cloud_only_file(&meta);
            let kind = if meta.is_dir() {
                EntryKind::Dir
            } else if meta.is_symlink() {
                EntryKind::Symlink
            } else {
                EntryKind::File
            };
            // クラウド専用ファイルのサイズは実ファイルを開かず 0 扱い
            let size = if kind == EntryKind::File && !is_cloud_only {
                Some(meta.len())
            } else if kind == EntryKind::File {
                Some(0)
            } else {
                None
            };
            let modified = meta.modified().ok().map(|st| st.into());
            let is_hidden = name.starts_with('.');
            Some(Entry {
                name,
                path: entry_path,
                kind,
                size,
                modified,
                is_hidden,
                is_cloud_only,
            })
        })
        .collect();

    entries.sort_by(|a, b| {
        let rank_a = if a.kind == EntryKind::Dir { 0u8 } else { 1 };
        let rank_b = if b.kind == EntryKind::Dir { 0u8 } else { 1 };
        rank_a
            .cmp(&rank_b)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

fn free_space(path: &Path) -> Option<u64> {
    drive_space(path).map(|(free, _)| free)
}
