# mmf 命令手册

中文 | [English](usage_EN.md) · [返回 README](../README.md)

适用版本：**0.1.1**。语法规范及示例见 [spec.md](spec.md)。以下命令与当前实现一致。

## 两种运行模式

单行模式在终端执行，每次运行结束后返回退出码：

```sh
mmf check docs -r
```

交互模式执行 `mmf_shell` 后，在 `mmf>` 提示符下输入同样的命令，但省略 `mmf` 前缀：

```text
mmf>check docs -r
mmf>fix notes.md
mmf>write notes.md
mmf>exit
```

相对路径基于启动程序时的工作目录。交互 shell 不支持 `cd`、管道、重定向或系统命令；这些应由外部终端执行。

## 命令总览

| 命令 | 参数 | 修改文件 | 输出 |
| --- | --- | --- | --- |
| `check` | 一个或多个文件/目录，可带 `-r` | 否 | 问题行号和说明，或 `路径: OK` |
| `fix` | 恰好一个文件 | 否 | 彩色差异；无差异时不输出 |
| `write` | 一个或多个文件/目录，可带 `-r` | 是 | 已保存修改的彩色差异；未改变时不输出 |
| `help` / `?` | 不需要参数 | 否 | 帮助文本 |
| `exit` | 不带参数，仅交互模式 | 否 | 退出交互 shell |

`mmf` 不带参数也显示帮助。当前没有 `--version`、`--help`、`-h`、`--recursive` 或 `--` 分隔符；使用 `mmf help` 和 `-r`。`mmf_shell` 欢迎页显示版本。

## check：检查问题

```text
mmf check <path>... [-r]
```

```sh
mmf check notes.md
mmf check README.md docs
mmf check docs -r
mmf check -r docs notes.md
```

有问题时，每条诊断采用 `路径:原始行号:说明` 格式。行号从 1 开始，说明目前为英文，例如：

```text
notes.md:2: list block must be preceded by a blank line
```

`check` 不写入文件。退出码 `0` 表示无问题，`1` 表示存在规则问题，`2` 表示参数、路径、读取或没有找到文件等错误；存在读取错误时 `2` 优先于 `1`。

## fix：预览修改

```text
mmf fix <file>
```

```sh
mmf fix notes.md
```

只接受一个文件，不接受目录或多个文件。成功时退出码始终为 `0`，包括存在待修复问题的情况；失败时为 `2`。使用 `check` 判断是否有规则问题。

差异带有文件头和 `@@ -旧行号,行数 +新行号,行数 @@` 标记，删除行用红色 `-`，新增行用绿色 `+`，不打印未变化的上下文。无变化时没有输出。它不是修复后的完整 Markdown；不要用 `mmf fix notes.md > notes.md` 保存修复结果。

`-r` 只对 `check` 和 `write` 的目录遍历有意义。当前解析器在 `fix` 中接受但忽略 `-r`，它不会让 `fix` 支持目录。

## write：原地修复

```text
mmf write <path>... [-r]
```

```sh
mmf write notes.md
mmf write docs README.md
mmf write docs -r
```

对内容变化的文件先保存，再输出与 `fix` 相同的差异。未变化的文件不写入且不输出。没有自动备份；建议使用 Git 查看 `git diff`，需要时恢复。

成功退出码为 `0`，失败为 `2`。批量处理遇到某个文件的读取或写入错误时会继续处理其他文件，因此一次运行可能已有部分文件保存成功。

输入要求为 UTF-8。处理结果采用 LF 换行；非空结果保留输入是否有结尾换行的状态。CRLF 文件可能仅因换行规范化而被写回，即使 `check` 没有报告规则问题；彩色差异按行内容比较，这种换行变化可能没有可见差异。

## 正文异常空格（spec 5.1）

`check`、`fix`、`write` 都会应用此规则，无需新增参数：

- 删除两个中文字符或中文标点之间的普通半角空格，例如 `内 存` → `内存`、`中文 ， 正文` → `中文，正文`。
- 连续正文空格先压成一个；若两侧都是中文字符或中文标点，再删除剩下的空格。
- 保留英文单词、中英文、中文和数字，以及 ASCII 标点或特殊字符两侧的单空格；不主动添加空格。
- 行内代码、围栏代码、缩进代码和公式中的空格保持不变。公式保护包括 `$...$`、`$$...$$`、`\(...\)` 和 `\[...\]`。
- Markdown 结构性缩进、列表标记后的填充和行尾用于换行的空格保留；此规则不将制表符或全角空格转换成半角空格。

例如：

```markdown
CXL.mem  允许主机处理器访问内 存。设备 1 使用 `a  b` 和 $x  + y$。
```

执行 `mmf write notes.md` 后：

```markdown
CXL.mem 允许主机处理器访问内存。设备 1 使用 `a  b` 和 $x  + y$。
```

`check` 对每个发生变化的原始行报告中文间隔或连续空格问题；一行可以有两类诊断。`fix` 仅预览，`write` 保存修复结果，退出码规则与前述命令一致。

## 路径和递归规则

- `check` / `write` 可同时接收多个文件和目录，收集结果按路径排序并按路径值去重。
- 目录默认只遍历当前层；`-r` 遍历子目录，仅收集扩展名为 `.md` 的文件，忽略扩展名大小写。
- 显式传入的文件不检查扩展名；请主动选择 Markdown 文件。
- 目录遍历不读取 `.gitignore`，也不自动跳过隐藏目录、`target` 等目录，应直接指定文档目录。
- 当前没有目录符号链接循环检测；递归路径中应避免指向祖先目录的链接。
- 名称以 `-` 开始的文件可以用 `./-notes.md`（PowerShell 下同样可用）传入。

### 外部终端中的空格路径

PowerShell 示例，引用内的反斜杠是普通字符：

```powershell
mmf check "D:\My Notes\guide.md"
mmf fix "D:\My Notes\guide.md"
mmf write "D:\My Notes" -r
$LASTEXITCODE
```

POSIX shell 示例：

```sh
mmf check "docs/my file.md"
echo $?
```

### mmf_shell 中的空格路径

交互解析器支持单引号和双引号。引号内的 `\` 会转义后一个字符，因此 Windows 路径优先使用正斜杠；也可写双反斜杠：

```text
mmf>check "D:/My Notes/guide.md"
mmf>check "D:\\My Notes\\guide.md"
```

此处的转义是 `mmf_shell` 自己的规则，与 PowerShell 引号规则不同。没有空格且不加引号的路径可以使用普通反斜杠。未闭合的引号会报错并返回提示符。

## help、? 和 exit

```sh
mmf
mmf help
mmf '?'
```

交互 shell 中输入 `help` 或 `?` 显示帮助；输入完整的 `exit` 或发送 EOF 退出。空行被忽略，命令区分大小写。检查/修复命令的退出码不会结束交互会话，也不会成为 shell 的进程退出码；自动化应使用单行模式。

## 在自动化中检查

PowerShell 示例，将规则问题和执行错误都视为失败：

```powershell
mmf check docs -r
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

POSIX shell 示例：

```sh
mmf check docs -r
status=$?
exit "$status"
```

普通输出写入 stdout，错误写入 stderr。`fix` / `write` 差异始终包含 ANSI 颜色序列，包括重定向时；当前没有禁用颜色的开关。
