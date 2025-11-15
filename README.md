# snapshot-cli

A blazing-fast, `.gitignore`-aware project snapshot generator for AI. Built with Rust.

一个为 AI 打造的、能感知 `.gitignore` 的极速项目快照生成器。由 Rust 驱动。

## ✨ Core Features / 核心特性

- **Blazing Fast**: Native performance powered by Rust and parallel processing, capable of handling massive codebases in milliseconds.
  - **极速性能**: 由 Rust 和并行处理驱动的原生性能，能在毫秒间处理大型代码库。
- **Intelligent Filtering**: Automatically respects `.gitignore`, global git config, and built-in best practices to exclude unnecessary files (`.git`, `node_modules`, `target/`, etc.).
  - **智能过滤**: 自动遵循 `.gitignore`、全局 git 配置和内置的最佳实践，排除无需的文件（如 `.git`, `node_modules`, `target/`）。
- **Unix Philosophy**: Does one thing and does it well—prints to standard output. Integrates seamlessly with your favorite shell tools (`>`, `|`, `&&`).
  - **Unix 哲学**: 只做一件事并把它做好——输出到标准输出。与你最爱的 shell 工具（`>`, `|`, `&&`）无缝集成。
- **Cross-Platform**: A single codebase compiles to a lightweight, dependency-free native executable for Windows, Linux, and macOS.
  - **跨平台**: 单一代码库可编译为轻量级、无依赖的原生可执行文件，适用于 Windows、Linux 和 macOS。

## 🚀 Installation / 安装

### Option 1: Using Cargo (Recommended) / 方式一：使用 Cargo (推荐)

```bash
cargo install snapshot-cli
```

### Option 2: From GitHub Releases / 方式二：从 GitHub Releases 下载

Download the pre-compiled binary for your operating system from the [Releases](https://github.com/cagedbird043/snapshot-cli/releases) page.
从 [Releases](https://github.com/cagedbird043/snapshot-cli/releases) 页面下载适用于您操作系统的预编译二进制文件。

## 🛠️ Usage & Workflows / 用法与工作流

The default behavior is to scan the current directory and print the snapshot to standard output.
默认行为是扫描当前目录，并将快照打印到标准输出。

```bash
snapshot-cli
```

### 1. Save to File / 保存到文件

```bash
snapshot-cli . > project-snapshot.md
```

### 2. Open in File Explorer / 在文件管理器中打开

This is a powerful workflow for drag-and-drop.
这是一个强大的拖放工作流。

**On Windows (with WSL):**
**在 Windows (含 WSL) 环境下：**

```bash
snapshot-cli . > project.md && explorer.exe .
```

**On Linux:**
**在 Linux 环境下：**

```bash
snapshot-cli . > project.md && dolphin . # Or your file manager of choice
```

### 3. Copy as Text (via pipe) / 作为文本复制 (通过管道)

Pipe the output to your favorite clipboard utility.
将输出通过管道传递给你最爱的剪贴板工具。

```bash
# For Linux/WSL with a pbcopy alias
snapshot-cli . | pbcopy

# For Windows PowerShell
snapshot-cli . | Set-Clipboard
```

### 4. Use the `--out` flag / 使用 `--out` 参数

For a more explicit way to save to a file.
一个更明确的保存文件方式。

```bash
snapshot-cli . --out project-snapshot.md
```

## 🏗️ Building from Source / 从源码构建

```bash
git clone https://github.com/cagedbird043/snapshot-cli.git
cd snapshot-cli
cargo build --release
# The executable will be in ./target/release/snapshot-cli
```
