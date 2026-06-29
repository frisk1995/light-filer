use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn log_path() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("filox");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("filox.log"))
}

pub fn write(level: &str, msg: &str) {
    let Some(path) = log_path() else { return };
    let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) else { return };
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let _ = writeln!(f, "[{now}] [{level}] {msg}");
}

/// アプリ起動時に呼ぶ。ログファイルにヘッダーを書き、パニックフックを設定する。
pub fn init() {
    // ログファイルに起動行を記録
    write("INFO", &format!("=== filox {} started ===", env!("CARGO_PKG_VERSION")));

    // パニック時にバックトレースをログファイルに書く
    std::panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic payload".to_owned()
        };

        let location = info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_owned());

        let msg = format!("PANIC at {location}: {payload}");
        write("PANIC", &msg);

        // バックトレース取得（RUST_BACKTRACE=1 不要で取れる場合のみ）
        let bt = std::backtrace::Backtrace::capture();
        if bt.status() == std::backtrace::BacktraceStatus::Captured {
            write("PANIC", &format!("backtrace:\n{bt}"));
        }
    }));
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log::write("ERROR", &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log::write("INFO", &format!($($arg)*));
    };
}
