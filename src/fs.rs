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
                    let entries = scan_dir(&path);
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

fn scan_dir(path: &Path) -> Vec<Entry> {
    let Ok(rd) = std::fs::read_dir(path) else {
        return Vec::new();
    };

    let mut entries: Vec<Entry> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let entry_path = e.path();
            let meta = e.metadata().ok()?;
            let kind = if meta.is_dir() {
                EntryKind::Dir
            } else if meta.is_symlink() {
                EntryKind::Symlink
            } else {
                EntryKind::File
            };
            let size = if meta.is_file() { Some(meta.len()) } else { None };
            let modified = meta
                .modified()
                .ok()
                .map(|st| st.into());
            let is_hidden = name.starts_with('.');
            Some(Entry {
                name,
                path: entry_path,
                kind,
                size,
                modified,
                is_hidden,
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
