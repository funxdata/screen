//! 轻量日志：同时输出到 stderr 与 `<work-dir>/logs/screen.log`。
//!
//! 不引入第三方日志库，避免为桌面外壳增加依赖。日志文件只追加、不轮转，
//! 需要清理时由运维删除 `logs/` 目录。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

/// 初始化日志文件。失败时只保留 stderr 输出，不阻塞启动。
pub fn init(work_dir: &Path) -> std::io::Result<()> {
    let dir = work_dir.join("logs");
    std::fs::create_dir_all(&dir)?;
    let _ = LOG_FILE.set(dir.join("screen.log"));
    Ok(())
}

pub fn info(message: impl AsRef<str>) {
    write(Level::Info, message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    write(Level::Warn, message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    write(Level::Error, message.as_ref());
}

/// 把 panic 信息同样写入日志文件。
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        error(format!("panic: {info}"));
        default_hook(info);
    }));
}

fn write(level: Level, message: &str) {
    let Some(min) = min_level() else {
        return;
    };
    if level < min {
        return;
    }

    let line = format!("{} [{}] {}", timestamp(), level.as_str(), message);
    eprintln!("{line}");

    if let Some(path) = LOG_FILE.get() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{line}");
        }
    }
}

/// 通过 `SCREEN_LOG=info|warn|error|off` 控制级别，默认 `info`。
fn min_level() -> Option<Level> {
    match std::env::var("SCREEN_LOG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "off" | "none" => None,
        "error" => Some(Level::Error),
        "warn" | "warning" => Some(Level::Warn),
        _ => Some(Level::Info),
    }
}

fn timestamp() -> String {
    format_utc(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    )
}

/// 把 Unix 秒格式化为 UTC（RFC 3339 的精简形式）。
fn format_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Howard Hinnant 的 civil_from_days 算法。
fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_unix_epoch() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(86_400), "1970-01-02T00:00:00Z");
    }

    #[test]
    fn formats_known_timestamp() {
        // 2025-01-01T00:00:00Z
        assert_eq!(format_utc(1_735_689_600), "2025-01-01T00:00:00Z");
    }

    #[test]
    fn formats_after_midnight() {
        assert_eq!(format_utc(1_735_689_601), "2025-01-01T00:00:01Z");
    }
}
