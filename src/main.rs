use clap::Parser;
use crossbeam_channel::unbounded;
use ignore::{DirEntry, WalkBuilder};
use std::path::{Path, PathBuf};

/// A blazing fast project scanner that intelligently ignores files
/// based on .gitignore rules, creating snapshots for AI consumption.
/// 一个极速的项目扫描器，它根据 .gitignore 规则智能地忽略文件，为 AI 分析创建快照。
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the project directory to scan.
    /// 需要扫描的项目目录路径。
    #[arg(default_value = ".")]
    path: PathBuf,
}

/// Scans a directory and returns a vector of valid file paths.
///
/// This function is the core file collection engine. It performs a parallel
/// traversal of the filesystem, respecting all standard .gitignore rules.
/// Results from multiple threads are collected into a single vector.
///
/// 扫描一个目录并返回一个包含有效文件路径的向量。
///
/// 此函数是核心的文件收集引擎。它执行并行的文件系统遍历，
/// 并遵循所有标准的 .gitignore 规则。来自多个线程的结果会被收集到
/// 一个单一的向量中。
///
/// # Arguments
///
/// * `path` - A reference to the Path representing the root directory to scan.
///   - `path` - 一个指向 Path 的引用，代表需要扫描的根目录。
///
/// # Returns
///
/// A `Vec<PathBuf>` containing the paths of all files that were not ignored.
/// The paths are sorted alphabetically for consistent output.
/// 一个 `Vec<PathBuf>`，其中包含所有未被忽略的文件的路径。
/// 路径会按字母顺序排序，以确保输出的一致性。
fn run_scanner(path: &Path) -> Vec<PathBuf> {
    let (tx, rx) = unbounded::<PathBuf>();

    WalkBuilder::new(path)
        .standard_filters(true)
        .hidden(false)
        .parents(true)
        .git_global(true)
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false)
        .build_parallel()
        .run(|| {
            let tx_clone = tx.clone();
            Box::new(move |result: Result<DirEntry, ignore::Error>| {
                if let Ok(entry) = result {
                    match entry.file_type() {
                        Some(ft) => ft.is_file(),
                        None => false,
                    }
                    .then(|| {
                        tx_clone.send(entry.path().to_path_buf()).unwrap();
                    });
                }
                ignore::WalkState::Continue
            })
        });

    drop(tx);

    let mut collected_paths: Vec<PathBuf> = rx.iter().collect();
    collected_paths.sort();
    collected_paths
}

fn main() {
    let cli = Cli::parse();
    let file_paths = run_scanner(&cli.path);

    println!("Found {} files to include:", file_paths.len());
    for path in file_paths {
        println!("{}", path.display());
    }
}

/// Unit tests for the scanner functionality.
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Tests the basic functionality of run_scanner to find files and respect .gitignore.
    ///
    /// This test creates a temporary directory with a nested structure and a .gitignore
    /// file. It verifies that the scanner correctly identifies the files that should
    /// be included while ignoring the ones specified in the .gitignore.
    ///
    /// 测试 run_scanner 查找文件和遵循 .gitignore 的基本功能。
    ///
    /// 本测试创建一个包含嵌套结构和 .gitignore 文件的临时目录。它验证扫描器
    /// 能正确识别应包含的文件，同时忽略 .gitignore 中指定的文件。
    #[test]
    fn test_scanner_with_gitignore() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("data/logs")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("README.md"), "# Test").unwrap();
        fs::write(root.join("data/logs/error.log"), "error!").unwrap();
        fs::write(root.join("config.toml"), "[config]").unwrap();
        let gitignore_content = "data/\n*.toml";
        fs::write(root.join(".gitignore"), gitignore_content).unwrap();
        let result_paths = run_scanner(root);
        let mut expected_paths: Vec<PathBuf> = vec![
            root.join("src/main.rs"),
            root.join("README.md"),
            root.join(".gitignore"),
        ];
        expected_paths.sort();

        assert_eq!(result_paths, expected_paths);
    }
}
