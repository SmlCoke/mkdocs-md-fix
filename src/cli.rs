use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::diff::colored_diff;
use crate::files::collect_inputs;
use crate::{check_markdown, fix_markdown};

pub fn run_command(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match parse_command(args) {
        Ok(Command::Help) => {
            let _ = writeln!(out, "{}", help_text());
            0
        }
        Ok(Command::Check { paths, recursive }) => match collect_inputs(&paths, recursive) {
            Ok(files) => run_check(&files, out, err),
            Err(error) => {
                let _ = writeln!(err, "error: {error}");
                2
            }
        },
        Ok(Command::FixStdout { path }) => run_fix_stdout(&path, out, err),
        Ok(Command::Write { paths, recursive }) => match collect_inputs(&paths, recursive) {
            Ok(files) => run_write(&files, out, err),
            Err(error) => {
                let _ = writeln!(err, "error: {error}");
                2
            }
        },
        Err(error) => {
            let _ = writeln!(err, "error: {error}");
            let _ = writeln!(err, "\n{}", help_text());
            2
        }
    }
}

#[derive(Clone, Debug)]
enum Command {
    Help,
    Check {
        paths: Vec<PathBuf>,
        recursive: bool,
    },
    FixStdout {
        path: PathBuf,
    },
    Write {
        paths: Vec<PathBuf>,
        recursive: bool,
    },
}

fn parse_command(args: &[String]) -> Result<Command, String> {
    if args.is_empty() {
        return Ok(Command::Help);
    }

    let command = args[0].as_str();
    if matches!(command, "help" | "?") {
        return Ok(Command::Help);
    }

    let mut recursive = false;
    let mut paths = Vec::new();

    for arg in &args[1..] {
        match arg.as_str() {
            "-r" => recursive = true,
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
            _ => paths.push(PathBuf::from(arg)),
        }
    }

    if paths.is_empty() {
        return Err(format!("missing path for command `{command}`"));
    }

    match command {
        "check" => Ok(Command::Check { paths, recursive }),
        "fix" => {
            if paths.len() == 1 {
                Ok(Command::FixStdout {
                    path: paths.remove(0),
                })
            } else {
                Err("fix accepts exactly one file path".to_string())
            }
        }
        "write" => Ok(Command::Write { paths, recursive }),
        other => Err(format!("unknown command: {other}")),
    }
}

fn run_check(files: &[PathBuf], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    if files.is_empty() {
        let _ = writeln!(err, "error: no markdown files found");
        return 2;
    }

    let mut has_issue = false;
    let mut has_error = false;

    for file in files {
        match fs::read_to_string(file) {
            Ok(content) => {
                let issues = check_markdown(&content);
                if issues.is_empty() {
                    let _ = writeln!(out, "{}: OK", file.display());
                } else {
                    has_issue = true;
                    for issue in issues {
                        let _ =
                            writeln!(out, "{}:{}: {}", file.display(), issue.line, issue.message);
                    }
                }
            }
            Err(error) => {
                has_error = true;
                let _ = writeln!(err, "error: failed to read {}: {error}", file.display());
            }
        }
    }

    if has_error {
        2
    } else if has_issue {
        1
    } else {
        0
    }
}

fn run_fix_stdout(path: &Path, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    if path.is_dir() {
        let _ = writeln!(
            err,
            "error: fix only supports one markdown file, not a directory"
        );
        return 2;
    }

    match fs::read_to_string(path) {
        Ok(content) => {
            let fixed = fix_markdown(&content);
            let diff = colored_diff(path, &content, &fixed.content);
            let _ = write!(out, "{diff}");
            0
        }
        Err(error) => {
            let _ = writeln!(err, "error: failed to read {}: {error}", path.display());
            2
        }
    }
}

fn run_write(files: &[PathBuf], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    if files.is_empty() {
        let _ = writeln!(err, "error: no markdown files found");
        return 2;
    }

    let mut has_error = false;

    for file in files {
        match fs::read_to_string(file) {
            Ok(content) => {
                let fixed = fix_markdown(&content);
                if fixed.content == content {
                    continue;
                }

                match fs::write(file, fixed.content.as_bytes()) {
                    Ok(()) => {
                        let diff = colored_diff(file, &content, &fixed.content);
                        let _ = write!(out, "{diff}");
                    }
                    Err(error) => {
                        has_error = true;
                        let _ = writeln!(err, "error: failed to write {}: {error}", file.display());
                    }
                }
            }
            Err(error) => {
                has_error = true;
                let _ = writeln!(err, "error: failed to read {}: {error}", file.display());
            }
        }
    }

    if has_error {
        2
    } else {
        0
    }
}

fn help_text() -> &'static str {
    r#"Mkdocs Markdown Fixer (mmf)

Usage:
  mmf check <path>... [-r]
  mmf fix <file>
  mmf write <path>... [-r]
  mmf help

Commands:
  check    Report line numbers and descriptions for non-compliant Markdown.
  fix      Print a colored diff for one file without modifying it.
  write    Print the same diff and fix files in place. Directories process *.md files.
  help     Print this help text.

Interactive shell:
  Run mmf_shell, then use commands such as `check docs`, `write docs -r`, or `exit`.
"#
}
