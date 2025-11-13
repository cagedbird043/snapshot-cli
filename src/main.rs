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

/// Generates the complete snapshot content as a Markdown string.
///
/// This function takes a list of file paths and orchestrates the creation of
/// the final snapshot. It will be responsible for building the project tree
/// visualization and appending the content of each file.
///
/// 生成完整的快照内容，格式为 Markdown 字符串。
///
/// 此函数接收一个文件路径列表，并主导最终快照的创建过程。它将负责
/// 构建项目树的可视化表示，并附加每个文件的内容。
///
/// # Arguments
///
/// * `project_name` - The name of the project, used in the snapshot header.
///   - `project_name` - 项目的名称，用于快照的标题。
/// * `paths` - A slice of `PathBuf` containing the files to be included.
///   - `paths` - 一个 `PathBuf` 的切片，包含所有需要被包含的文件。
///
/// # Returns
///
/// A `String` containing the full Markdown snapshot.
/// 一个 `String`，其中包含完整的 Markdown 快照。
fn generate_snapshot_content(project_name: &str, paths: &[PathBuf]) -> String {
    let header = format!("# Project Snapshot: {}\n\n", project_name);
    let summary = format!(
        "This file contains a snapshot of the project structure and source code, formatted for AI consumption.\n"
    );
    let total_files = format!("Total files included: {}\n\n", paths.len());

    // --- Placeholder for Project Tree ---
    let project_tree_header = "## Project Structure\n\n";
    let project_tree = "```\nTODO: Implement project tree here\n```\n\n";

    // --- Placeholder for File Contents ---
    let file_contents_header = "## File Contents\n\n";
    let file_contents = "```\nTODO: Implement file content appending here\n```\n\n";

    [
        header,
        summary,
        total_files,
        project_tree_header.to_owned(),
        project_tree.to_owned(),
        file_contents_header.to_owned(),
        file_contents.to_owned(),
    ]
    .concat()
}

fn main() {
    let cli = Cli::parse();
    let project_name = cli
        .path
        .file_name()
        .unwrap_or_else(|| cli.path.as_os_str())
        .to_string_lossy();

    let file_paths = run_scanner(&cli.path);

    if file_paths.is_empty() {
        println!("No files to include in the snapshot. Exiting.");
        return;
    }

    let snapshot = generate_snapshot_content(&project_name, &file_paths);
    println!("{}", snapshot);
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
