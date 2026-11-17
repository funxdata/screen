//! 运行期路径解析。
//!
//! `run.pid`、日志与 `cyctron` 都相对**工作目录**解析，工作目录的优先级：
//! 命令行 `--work-dir` > 环境变量 `SCREEN_HOME` > 可执行文件所在目录 > 当前目录。
//! 这样从快捷方式、服务或任意 CWD 启动时都能得到稳定的路径。

use std::path::{Path, PathBuf};

/// `cyctron` 在各平台的发布命名。
const CYCTRON_CANDIDATES: [&str; 4] = ["cyctron", "cyctron.exe", "cyctron.bin", "cyctron.app"];

/// 解析工作目录。
pub fn resolve_work_dir(explicit: Option<&Path>) -> PathBuf {
    if let Some(dir) = explicit {
        return dir.to_path_buf();
    }

    if let Some(dir) = std::env::var_os("SCREEN_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.to_path_buf();
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 在工作目录（以及 macOS 的 bundle 邻接目录）中查找 `cyctron`。
pub fn find_cyctron(work_dir: &Path) -> Option<PathBuf> {
    for dir in search_dirs(work_dir) {
        for name in CYCTRON_CANDIDATES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// 参与查找的目录列表，按优先级去重。
pub fn search_dirs(work_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![work_dir.to_path_buf()];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.to_path_buf());

            // macOS: <App>.app/Contents/MacOS/screen，依赖可能被放在
            // Contents/Resources 或 bundle 外层。
            #[cfg(target_os = "macos")]
            {
                if let Some(contents) = parent.parent() {
                    dirs.push(contents.join("Resources"));
                    dirs.push(contents.to_path_buf());
                    if let Some(bundle_parent) = contents.parent() {
                        dirs.push(bundle_parent.to_path_buf());
                    }
                }
            }
        }
    }

    let mut unique = Vec::new();
    for dir in dirs {
        if !unique.contains(&dir) {
            unique.push(dir);
        }
    }
    unique
}

/// PID 文件路径。
pub fn pid_path(work_dir: &Path) -> PathBuf {
    work_dir.join("run.pid")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "screen-paths-{tag}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn explicit_work_dir_wins() {
        let dir = temp_dir("explicit");
        assert_eq!(resolve_work_dir(Some(&dir)), dir);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pid_path_is_inside_work_dir() {
        assert_eq!(pid_path(Path::new("/tmp/x")), Path::new("/tmp/x/run.pid"));
    }

    #[test]
    fn finds_cyctron_candidate() {
        let dir = temp_dir("cyctron");
        let exe = dir.join("cyctron.exe");
        std::fs::write(&exe, b"").unwrap();
        assert_eq!(find_cyctron(&dir), Some(exe));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn returns_none_without_cyctron() {
        let dir = temp_dir("empty");
        assert_eq!(find_cyctron(&dir), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
