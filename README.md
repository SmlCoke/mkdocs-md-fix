# mkdocs-md-fix

中文 | [English](README_EN.md)

**Mkdocs Markdown Fixer（mmf）** 是用 Rust 编写的 Markdown 检查与修复工具，针对编辑器预览正常、在 MkDocs 中却可能渲染异常的空行和缩进写法，帮助统一文档源文件。

当前版本：**0.1.0**。提供单行命令工具 `mmf` 和交互工具 `mmf_shell`，采用 MIT 许可证。运行时无需 MkDocs；实际网页渲染仍取决于项目的 Markdown 扩展与配置。

## 处理哪些问题

规则以 [docs/spec.md](docs/spec.md) 为准，规格书含正确与错误示例。

| 语法 | 检查与修复内容 |
| --- | --- |
| 列表 | 列表块前后补空行 |
| 嵌套列表 | 缩进按每级四个空格规范化；非四的倍数向上补齐 |
| 列表内引用 | 采用四空格缩进、引用块前后空行的推荐写法 |
| 列表内围栏代码块 | 补齐所需的四空格层级缩进，块前后补空行 |
| 行间公式 | `$$...$$` 单行形式及独立 `$$` 包围的多行形式前后补空行 |
| 围栏代码块 | 代码块整体前后补空行 |
| Markdown 表格 | 表格块前后补空行 |

行内公式不增加额外规则。围栏代码块中的 Markdown 示例不会按正文规则修复；列表内代码块的必要缩进会调整。表格识别针对以 `|` 开始和结束的连续行。本工具采用文本规则，没有完整 Markdown 语法树，不承担通用 Markdown 校验或 MkDocs 构建验证。

## 安装与稳定版本

### 下载 GitHub Release（无需 Rust）

从 [v0.1.0 Release](https://github.com/SmlCoke/mkdocs-md-fix/releases/tag/v0.1.0) 下载对应系统和 CPU 架构的压缩包：

| 系统 / 架构 | 文件 |
| --- | --- |
| Windows x64 | `mmf-v0.1.0-x86_64-pc-windows-msvc.zip` |
| Linux x64 | `mmf-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `mmf-v0.1.0-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `mmf-v0.1.0-aarch64-apple-darwin.tar.gz` |

每个包包含 `mmf`、`mmf_shell`（Windows 下带 `.exe`）、中英文说明、命令手册、规格书和许可证。Linux 包在 Ubuntu 22.04 上构建和测试，适用于兼容的 glibc 系统；macOS 包以 macOS 11.0 为最低部署目标。其他系统、架构可从源码编译。

Windows 示例（在下载文件所在目录执行）：

```powershell
Expand-Archive .\mmf-v0.1.0-x86_64-pc-windows-msvc.zip -DestinationPath D:\Tools
& D:\Tools\mmf-v0.1.0-x86_64-pc-windows-msvc\mmf.exe help
```

将解压后的版本目录加入用户 PATH，例如 `D:\Tools\mmf-v0.1.0-x86_64-pc-windows-msvc`，重新打开终端后即可直接使用。若 PATH 中还有仓库的 `target\debug`，应移除该项或确保稳定版本目录优先；可用 `Get-Command mmf` 确认实际程序路径。

Linux 示例（macOS 替换为对应压缩包和目录名）：

```sh
mkdir -p "$HOME/.local/opt"
tar -xzf mmf-v0.1.0-x86_64-unknown-linux-gnu.tar.gz -C "$HOME/.local/opt"
export PATH="$HOME/.local/opt/mmf-v0.1.0-x86_64-unknown-linux-gnu:$PATH"
mmf help
```

可将 PATH 设置加入自己的 shell 配置以持久生效。Release 还提供源码 `.zip` / `.tar.gz`、单独的 `spec.md` 和 `SHA256SUMS`。校验示例：

```powershell
Get-FileHash .\mmf-v0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

```sh
sha256sum mmf-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
# macOS: shasum -a 256 mmf-v0.1.0-aarch64-apple-darwin.tar.gz
```

将结果与 `SHA256SUMS` 中对应文件的值比较。

### 从指定 tag 安装（需要 Rust）

先安装 [Rust 工具链](https://www.rust-lang.org/tools/install)，然后运行：

```sh
cargo install --git https://github.com/SmlCoke/mkdocs-md-fix --tag v0.1.0 --locked --bins
```

两个程序安装到 Cargo 安装根目录的 `bin` 子目录，默认通常为 `$HOME/.cargo/bin`，设置了 `CARGO_HOME` 时则使用该目录。也可以指定独立目录：

```sh
cargo install --git https://github.com/SmlCoke/mkdocs-md-fix --tag v0.1.0 --locked --bins --root ./mmf-0.1.0
```

将其 `bin` 子目录加入 PATH。指定 tag 的安装副本不受开发仓库后续编译影响；只有主动重新安装或替换副本才会更新。参数见 [Cargo 官方安装文档](https://doc.rust-lang.org/cargo/commands/cargo-install.html)。

### 从源码编译

```sh
git clone https://github.com/SmlCoke/mkdocs-md-fix.git
cd mkdocs-md-fix
git checkout v0.1.0
cargo test --locked
cargo build --release --locked --bins
```

可执行文件位于 `target/release/`。Linux/macOS 可运行 `./target/release/mmf help`；Windows PowerShell 可运行 `.\target\release\mmf.exe help`。`target/release` 同样会随编译变化，长期使用请将两个程序复制到独立版本目录，或采用 tag 安装方式。

本仓库维护者使用 MSYS2 CLANG64 环境。进入该环境后执行相同的 Cargo 命令；本地 Cargo 位于 `/clang64/.cargo`，rustup 位于 `/clang64/.rustup`。下载 Release 的用户无需 MSYS2。

## 快速使用

### 单行命令

```sh
mmf check notes.md
mmf check docs notes.md -r
mmf fix notes.md
mmf write docs -r
mmf help
```

`check` 报告问题及原始行号；`fix` 仅打印一个文件的彩色差异；`write` 打印差异并原地保存。目录默认只检查当前层的 `.md` 文件，`-r` 包含子目录。建议先 `check`，再 `fix` 预览，确认后 `write`；原地修改没有自动备份，可以通过 Git 管理更改。

例如，`notes.md` 中的内容：

```markdown
介绍：
- 第一项
  - 子项
结束。
```

执行 `mmf write notes.md` 后变为：

```markdown
介绍：

- 第一项
    - 子项

结束。
```

### 交互 shell

启动 `mmf_shell`，然后输入不带 `mmf` 前缀的命令：

```text
mmf>check docs/spec.md
mmf>fix "notes/my file.md"
mmf>write docs -r
mmf>help
mmf>exit
```

完整命令、路径规则、退出码、Windows 示例与注意事项见 [中文命令手册](docs/usage.md) / [English usage guide](docs/usage_EN.md)。

## 仓库目录

```text
mkdocs-md-fix/
├── .github/
│   ├── workflows/release.yml      # 各平台原生测试、构建与制品上传
│   └── scripts/package_release.py # 打包程序、文档并验证发布包
├── src/
│   ├── lib.rs                     # 核心规则、命令处理、差异输出、单元测试
│   ├── main.rs                    # mmf 单行命令入口
│   └── bin/mmf_shell.rs           # 交互 shell 入口与欢迎页
├── docs/
│   ├── spec.md                    # Markdown 写法规格书
│   ├── usage.md                   # 中文命令手册
│   └── usage_EN.md                # 英文命令手册
├── Cargo.toml                    # 包版本、库与两个可执行目标
├── Cargo.lock                    # 锁定包版本信息
├── README.md                     # 中文项目说明
├── README_EN.md                  # 英文项目说明
├── LICENSE                       # MIT 许可证
├── .gitignore                    # 忽略构建目录和本地开发材料
├── target/                       # 本地编译产物，不入库
└── dist/                         # 本地发布制品，不入库
```

`docs/prompts/` 是被忽略的本地任务材料，不随源码发布。测试目前位于 `src/lib.rs` 的 `#[cfg(test)]` 模块，没有独立的受版本管理的 `tests/` 目录。

## 开发与许可证

```sh
cargo test --locked
cargo build --release --locked --bins
```

[Release build 工作流](.github/workflows/release.yml) 可通过 GitHub Actions 手动运行，在 Windows、Linux 和两种 macOS 架构上测试、编译并上传压缩包。工作流只生成制品，Release 由维护者检查制品后发布。

规则行为的变更应与 [规格书](docs/spec.md) 对齐。项目采用 [MIT License](LICENSE)。
