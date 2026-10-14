# mmf command reference

[中文](usage.md) | English · [Back to README](../README_EN.md)

Applies to **0.1.0**. See [spec.md](spec.md) for formatting rules and examples. This guide describes the current implementation.

## Two modes

Command-line mode runs in your terminal and returns an exit code:

```sh
mmf check docs -r
```

For interactive mode, run `mmf_shell`, then enter commands without the `mmf` prefix:

```text
mmf>check docs -r
mmf>fix notes.md
mmf>write notes.md
mmf>exit
```

Relative paths use the working directory from which the program was started. The interactive shell does not execute `cd`, pipelines, redirection or system commands. Use your terminal for those operations.

## Commands

| Command | Arguments | Changes files | Output |
| --- | --- | --- | --- |
| `check` | One or more files/directories, optional `-r` | No | Issue lines and descriptions, or `path: OK` |
| `fix` | Exactly one file | No | Colored diff; nothing if unchanged |
| `write` | One or more files/directories, optional `-r` | Yes | Colored diff for saved changes; nothing if unchanged |
| `help` / `?` | No arguments required | No | Help text |
| `exit` | No arguments, interactive mode only | No | Leave the interactive shell |

Running `mmf` with no arguments also displays help. There are no `--version`, `--help`, `-h`, `--recursive` or `--` separator options. Use `mmf help` and `-r`. The interactive welcome banner displays the version.

## check: report issues

```text
mmf check <path>... [-r]
```

```sh
mmf check notes.md
mmf check README.md docs
mmf check docs -r
mmf check -r docs notes.md
```

Diagnostics use `path:original-line-number:description`. Line numbers start at 1; descriptions are currently in English. For example:

```text
notes.md:2: list block must be preceded by a blank line
```

No files are written. Exit code `0` means no issues, `1` means formatting issues, and `2` means an argument, path, read or empty-input error. Read errors take precedence over formatting issues.

## fix: preview changes

```text
mmf fix <file>
```

```sh
mmf fix notes.md
```

Accepts one file, not a directory or multiple files. A successful preview returns `0` even if fixes are needed; errors return `2`. Use `check` to determine whether formatting issues exist.

The diff has file headers and `@@ -old-line,count +new-line,count @@` markers. Red `-` lines are deletions; green `+` lines are additions. Unchanged context is omitted. There is no output when unchanged. This is a diff, not the complete fixed Markdown; do not save changes with `mmf fix notes.md > notes.md`.

`-r` affects directory traversal only for `check` and `write`. The current parser accepts but ignores it for `fix`; it does not enable directory support.

## write: apply changes

```text
mmf write <path>... [-r]
```

```sh
mmf write notes.md
mmf write docs README.md
mmf write docs -r
```

Changed files are saved before their diffs are printed. Unchanged files are neither written nor printed. No automatic backup is created; use `git diff` to review and Git to recover changes.

Returns `0` on success or `2` on error. A read/write error for one file does not stop processing the remaining files, so some files may already have been saved when the command fails.

Input must be UTF-8. Output uses LF line endings; nonempty output preserves whether the input had a trailing newline. A CRLF file may be rewritten solely to normalize line endings even if `check` reports no formatting issues. Since the diff compares line contents, this change may produce no visible diff.

## Paths and recursion

- `check` / `write` accept multiple files and directories; collected paths are sorted and deduplicated by path value.
- Directories scan only the current level unless `-r` is supplied. They collect only `.md` files, case-insensitively.
- Explicit file arguments are not filtered by extension. Select Markdown files yourself.
- Traversal does not read `.gitignore` or skip hidden directories or build directories. Prefer targeting your document directory.
- Directory symlink cycles are not detected. Avoid links to ancestor directories in recursive inputs.
- For filenames starting with `-`, use a path such as `./-notes.md`, also supported in PowerShell.

### Paths with spaces in your terminal

PowerShell treats backslashes inside quotes as ordinary characters:

```powershell
mmf check "D:\My Notes\guide.md"
mmf fix "D:\My Notes\guide.md"
mmf write "D:\My Notes" -r
$LASTEXITCODE
```

POSIX shell:

```sh
mmf check "docs/my file.md"
echo $?
```

### Paths with spaces in mmf_shell

The interactive parser supports single and double quotes. Inside either kind of quote, a backslash escapes the next character. Prefer forward slashes in Windows paths, or double the backslashes:

```text
mmf>check "D:/My Notes/guide.md"
mmf>check "D:\\My Notes\\guide.md"
```

These are the interactive parser's rules, distinct from PowerShell quoting. Unquoted paths without spaces can use ordinary backslashes. An unclosed quote reports an error and returns to the prompt.

## help, ? and exit

```sh
mmf
mmf help
mmf '?'
```

Inside the interactive shell, `help` and `?` show help; the exact command `exit` or EOF ends the session. Blank input is ignored and commands are case-sensitive. Command exit codes do not end the interactive session or become its process exit code. Use command-line mode for automation.

## Automation

PowerShell, treating both formatting issues and execution errors as failures:

```powershell
mmf check docs -r
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

POSIX shell:

```sh
mmf check docs -r
status=$?
exit "$status"
```

Normal output goes to stdout and errors to stderr. `fix` / `write` always emit ANSI color sequences, including when redirected. There is no color-disable option.
