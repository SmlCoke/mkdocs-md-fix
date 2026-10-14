# Agent 工作准则

本文件适用于整个仓库，作为后续补充要求的基础模板。开始工作前阅读本文件和相关规格；用户在当前会话中的明确要求优先。

## 基础规则

1. 先理解需求、仓库状态和现有实现，再修改。使用 `git status` 检查已有改动，不覆盖用户尚未提交的工作。
2. 聚焦当前任务，保持改动可审查；不要顺带改写无关代码、文档、系统 PATH 或工具配置。
3. 持续完成已获授权的工作；需求不清楚时先查仓库资料，必要时说明具体歧义并请求澄清。
4. 对用户用中文汇报修改内容、验证结果和实际限制。失败检查必须如实说明，不能将未验证的行为写成已通过。
5. 后续新增规则可写入对应章节；遇到冲突时明确说明，不自行删除已有要求。

## 文档规则

1. `docs/spec.md` 是 mmf 功能规格书。所有算法、检查和修复行为必须严格与该文档对齐；禁止在未获授权时改写规格以迎合实现。
2. `README.md` 与 `README_EN.md` 分别为中英文项目介绍，两者的功能范围、版本、目录结构、安装和使用示例必须保持一致。
3. `docs/usage.md` 与 `docs/usage_EN.md` 分别为中英文使用手册，两者的命令、参数、退出码、路径规则和限制必须保持一致。
4. 修改功能或命令时同步更新两种语言的相关说明，检查相对链接、文件名和示例是否可用。
5. README 保留项目介绍与快速入门；详细命令解释放入使用手册。不要在多个文档中维护互相矛盾的命令说明。

## 环境规则

1. 本机使用 Windows PowerShell。Rust/Cargo 不在系统 PATH，需要在 **MSYS2 CLANG64** 环境中执行。
2. Bash 路径为 `D:\MSYS\msys64\usr\bin\bash.exe`。本机 Cargo 位于 `D:\MSYS\msys64\clang64\.cargo\bin`，rustup 数据位于 `D:\MSYS\msys64\clang64\.rustup`。
3. 从 PowerShell 执行 Rust 命令的示例：

    ```powershell
    $env:MSYSTEM = 'CLANG64'
    & 'D:\MSYS\msys64\usr\bin\bash.exe' --login -i -c 'cd /d/Project/mkdocs-md-fix && cargo test --locked'
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    ```

4. 如果登录环境未加载 Rust 配置，仅为当前命令设置 `RUSTUP_HOME=/clang64/.rustup`、`CARGO_HOME=/clang64/.cargo`，并将 `/clang64/.cargo/bin` 加入该进程 PATH；不要修改用户或系统的全局设置。
5. `git` 和 `gh` 已加入系统 PATH，可直接调用。本机已确认路径为 `D:\Git\Git\cmd\git.exe` 与 `C:\Program Files\GitHub CLI\gh.exe`；跨机器工作时先用 `Get-Command git,gh` 验证，不假定绝对路径相同。
6. 用户日常使用的稳定程序应与开发目录分离；未经要求不要改动已安装程序或用户 PATH。

## 代码规则

1. 保持源码拆分和小模块结构，**禁止将所有算法重新集中到 `src/lib.rs`**。该文件只负责模块声明和公开接口导出。
2. `markdown.rs` 组织处理流程，`model.rs` 定义结果类型，`syntax.rs` 提供共享语法识别；命令、文件操作、差异和交互输入解析分别放在 `cli.rs`、`files.rs`、`diff.rs`、`shell.rs`。
3. 规则实现放在 `src/rules/`，按列表、引用、代码块、公式、表格、正文空格等职责分组；公共调度只负责顺序与共享上下文。
4. 保持公开接口、原始诊断行号、退出码及非写入命令的行为稳定。新增规则必须保护代码和公式内容，不破坏 Markdown 结构性缩进。
5. 新增依赖前先判断标准库是否足够。不要为了拆分而引入复杂的抽象或循环依赖。
6. 功能修改需要有针对性的测试，覆盖正常输入、异常输入、保护区域和重复修复的稳定性；现有回归测试必须继续通过。
7. 交付前运行 `cargo fmt --check`、`cargo test --locked` 和适当的构建检查。若检查失败，先修复并说明原因；不要在没有新改动或未解决问题的情况下重复扩大测试范围。

## 发布规则

1. 使用语义化版本号，例如 `0.1.0`、`0.1.1`、`0.2.0`、`1.0.0`。涉及版本变更时同步 `Cargo.toml`、`Cargo.lock`、双语 README 和使用手册；欢迎页自动读取 Cargo 包版本。
2. **除非用户明确要求，否则不得自行 commit、push、创建 tag 或发布 Release。** 当前任务包含明确发布要求时，按其版本和范围执行，不重复询问已获授权的操作。
3. 发布前先完成代码与文档验证。通过 `.github/workflows/release.yml` 在 Windows、Linux、macOS Intel 和 Apple Silicon 上进行原生测试、构建及解压包运行验证。
4. tag 采用 `v<版本号>`，必须指向已通过构建验证的提交。禁止移动或覆盖已有已发布 tag；也不要覆盖历史 Release 附件。
5. Release 至少包含源码 ZIP/TAR.GZ、独立规格书、各平台程序包、许可证和 `SHA256SUMS`。程序包同时包含 `mmf`、`mmf_shell` 和双语说明。
6. 源码包使用受 Git 管理的文件生成，排除 `target/`、`dist/`、本地 prompts 等材料；保留规格书内容和原始换行。
7. 上传后核对远端附件名称、大小和 SHA-256，确认正式发布成功，再向用户汇报 Release 链接、提交与测试结果。

## 后续补充

- 基础规则补充：待填写。
- 文档规则补充：待填写。
- 环境规则补充：待填写。
- 代码规则补充：待填写。
- 发布规则补充：待填写。
