use clap::Parser;
use crossbeam_channel::unbounded;
use ignore::overrides::OverrideBuilder;
use ignore::{DirEntry, WalkBuilder};
use std::collections::HashMap;
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

    /// Output the snapshot content to a specified file.
    /// 将快照内容输出到指定文件。
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// Output the snapshot content as plain text to standard output.
    /// This is the default behavior if --out is not specified.
    /// 将快照内容作为纯文本输出到标准输出。
    /// 如果未指定 --out，则此为默认行为。
    #[arg(short, long, default_value_t = true)]
    text: bool,
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
    let mut builder = WalkBuilder::new(path);
    let mut override_builder = OverrideBuilder::new(path);
    override_builder.add("!.git/").unwrap();
    builder.overrides(override_builder.build().unwrap());
    builder
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

/// Represents a node in the project directory tree structure.
///
/// 表示项目目录树结构中的一个节点。
///
/// This enum is used to build a hierarchical representation of the filesystem,
/// where each node can either be a file or a directory containing other nodes.
/// 此枚举用于构建文件系统的层级表示，
/// 其中每个节点可以是文件或包含其他节点的目录。
///
/// # Variants
///
/// * `File` - Represents a file in the tree.
///   - `File` - 表示树中的一个文件。
/// * `Directory(HashMap<String, TreeNode>)` - Represents a directory containing child nodes.
///   - `Directory(HashMap<String, TreeNode>)` - 表示包含子节点的目录。
enum TreeNode {
    File,
    Directory(HashMap<String, TreeNode>),
}

/// Inserts a path into the tree structure recursively.
///
/// 递归地将路径插入到树结构中。
///
/// # Arguments
///
/// * `node` - A mutable reference to the current TreeNode.
///   - `node` - 对当前 TreeNode 的可变引用。
/// * `components` - A slice of path components as strings.
///   - `components` - 路径组件的字符串切片。
fn insert_path(node: &mut TreeNode, components: &[String]) {
    if components.is_empty() {
        return;
    }
    if let TreeNode::Directory(children) = node {
        if components.len() == 1 {
            children.insert(components[0].clone(), TreeNode::File);
        } else {
            let child = children
                .entry(components[0].clone())
                .or_insert(TreeNode::Directory(HashMap::new()));
            insert_path(child, &components[1..]);
        }
    }
}

/// Generates a string representation of the project's directory tree.
///
/// This function converts a flat list of file paths into a hierarchical,
/// text-based tree structure, similar to the output of the `tree` command.
///
/// 生成项目目录树的字符串表示。
///
/// 此函数将一个扁平的文件路径列表转换为一个层级化的、基于文本的
/// 树形结构，类似于 `tree` 命令的输出。
///
/// # Arguments
///
/// * `base_path` - The root path of the project, used to determine relative paths.
///   - `base_path` - 项目的根路径，用于计算相对路径。
/// * `paths` - A slice of `PathBuf` containing the files to be included.
///   - `paths` - 一个 `PathBuf` 的切片，包含所有需要被包含的文件。
///
/// # Returns
///
/// A `String` containing the formatted directory tree.
/// 一个 `String`，其中包含格式化后的目录树。
fn generate_project_tree(base_path: &Path, paths: &[PathBuf]) -> String {
    let mut root = TreeNode::Directory(HashMap::new());
    for path in paths {
        let relative_path = match path.strip_prefix(base_path) {
            Ok(rel) => rel,
            Err(_) => path.as_path(),
        };
        let components: Vec<String> = relative_path
            .components()
            .map(|c| c.as_os_str().to_str().unwrap().to_string())
            .collect();
        insert_path(&mut root, &components);
    }

    fn build_tree_string(node: &TreeNode, prefix: &str) -> String {
        if let TreeNode::Directory(children) = node {
            let mut entries: Vec<_> = children.keys().collect();
            entries.sort();

            let mut result = String::new();
            for (i, name) in entries.iter().enumerate() {
                let is_last = i == entries.len() - 1;
                let connector = if is_last { "└── " } else { "├── " };
                let child_prefix = if is_last { "    " } else { "│   " };

                result.push_str(&format!("{}{}{}\n", prefix, connector, name));

                if let Some(child_node) = children.get(*name) {
                    if let TreeNode::Directory(_) = child_node {
                        result.push_str(&build_tree_string(
                            child_node,
                            &format!("{}{}", prefix, child_prefix),
                        ));
                    }
                }
            }
            return result;
        }
        String::new()
    }

    ".\n".to_owned() + &build_tree_string(&root, "")
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
fn generate_snapshot_content(project_name: &str, base_path: &Path, paths: &[PathBuf]) -> String {
    let header = format!("# Project Snapshot: {}\n\n", project_name);
    let summary = format!(
        "This file contains a snapshot of the project structure and source code, formatted for AI consumption.\n"
    );
    let total_files = format!("Total files included: {}\n\n", paths.len());

    let project_tree_str = generate_project_tree(base_path, paths);
    let project_tree = format!("```\n{}\n```\n\n", project_tree_str);

    let file_contents_header = "## File Contents\n\n";
    let file_contents: String = paths
        .iter()
        .map(|path| {
            let relative_path = path.strip_prefix(base_path).unwrap_or(path);
            let content = match std::fs::read_to_string(path) {
                Ok(text) => text,
                Err(e) => format!("Error reading file: {}", e),
            };
            let lang = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            format!(
                "```{}\n{}\n```\n\n",
                format_args!("{}:{}", lang, relative_path.display()),
                content
            )
        })
        .collect();

    [
        header,
        summary,
        total_files,
        project_tree,
        file_contents_header.to_owned(),
        file_contents,
    ]
    .concat()
}

fn main() {
    let cli = Cli::parse();
    let project_path = &cli.path;
    let project_name = project_path
        .file_name()
        .unwrap_or_else(|| project_path.as_os_str())
        .to_string_lossy();

    let file_paths = run_scanner(project_path);

    if file_paths.is_empty() {
        // Use stderr for error messages or status updates
        eprintln!("No files to include in the snapshot. Exiting.");
        return;
    }

    let snapshot_content: String =
        generate_snapshot_content(&project_name, project_path, &file_paths);

    if let Some(output_path) = cli.out {
        std::fs::write(&output_path, &snapshot_content).expect("Failed to write to output file");
        eprintln!(
            "Snapshot successfully written to: {}",
            output_path.display()
        );
    } else {
        // Default behavior: print to standard output
        println!("{}", snapshot_content);
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

/// Tests the project tree generation logic.
///
/// This test provides a predefined set of paths and a base path, then
/// verifies that the generated tree string matches the expected hierarchical
/// structure, including correct connectors and sorting.
///
/// 测试项目树生成逻辑。
///
/// 本测试提供一组预定义的路径和一个基础路径，然后验证生成的树字符串
/// 是否与预期的层级结构匹配，包括正确的连接符和排序。
#[test]
fn test_generate_project_tree() {
    // 1. Setup: Define a base path and a list of file paths
    let base_path = Path::new("/tmp/test-project");
    let paths = vec![
        base_path.join("src/main.rs"),
        base_path.join("Cargo.toml"),
        base_path.join("src/module/api.rs"),
        base_path.join(".gitignore"),
    ];

    // 2. Execution: Generate the project tree
    let tree = generate_project_tree(base_path, &paths);

    // 3. Assertion: Check against the expected string output
    let expected_tree = r#".
├── .gitignore
├── Cargo.toml
└── src
    ├── main.rs
    └── module
        └── api.rs
"#;
    assert_eq!(tree.trim(), expected_tree.trim());
}
