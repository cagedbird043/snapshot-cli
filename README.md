# snapshot-cli

A blazing-fast, `.gitignore`-aware project snapshot generator for AI. Built with Rust.

一个为 AI 打造的、能感知 `.gitignore` 的极速项目快照生成器。由 Rust 驱动。

## ✨ Core Features / 核心特性

- **Blazing Fast**: Native performance powered by Rust and parallel processing, capable of handling massive codebases in milliseconds.
  - **极速性能**: 由 Rust 和并行处理驱动的原生性能，能在毫秒间处理大型代码库。
- **Intelligent Filtering**: Automatically respects `.gitignore` and built-in best practices to exclude unnecessary files (`.git`, `node_modules`, `target/`, etc.).
  - **智能过滤**: 自动遵循 `.gitignore` 和内置的最佳实践，排除无需的文件（如 `.git`, `node_modules`, `target/`）。
- **Unix Philosophy**: Does one thing and does it well—prints to standard output. Integrates seamlessly with your favorite shell tools (`>`, `|`, `&&`).
  - **Unix 哲学**: 只做一件事并把它做好——输出到标准输出。与你最爱的 shell 工具（`>`, `|`, `&&`）无缝集成。
- **Cross-Platform**: A single codebase compiles to a lightweight, dependency-free native executable for Windows, Linux, and macOS.
  - **跨平台**: 单一代码库可编译为轻量级、无依赖的原生可执行文件，适用于 Windows、Linux 和 macOS。

## 🛠️ Usage & Workflows / 用法与工作流

The default behavior is to scan the current directory and print the snapshot to standard output.
默认行为是扫描当前目录，并将快照打印到标准输出。

```bash
# Display snapshot in console
# 在控制台显示快照
snapshot-cli .

# Save snapshot to a file
# 保存快照到文件
snapshot-cli . > project-snapshot.md

# Open in file explorer after creation (Linux example)
# 创建后在文件管理器中打开 (Linux 示例)
snapshot-cli . > project.md && dolphin .
```

## 🏗️ Building from Source / 从源码构建

Ensure you have the Rust toolchain installed.
请确保已安装 Rust 工具链。

```bash
git clone https://github.com/your-username/snapshot-cli.git
cd snapshot-cli
cargo build --release
# The executable will be in ./target/release/
# 可执行文件将位于 ./target/release/ 目录下
# Move it to a directory in your $PATH.
# 将其移动到你的 $PATH 路径下的某个目录中。
```
