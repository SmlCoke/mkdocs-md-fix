use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RESET: &str = "\x1b[0m";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub line: usize,
    pub message: String,
}

impl Issue {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixResult {
    pub content: String,
    pub issues: Vec<Issue>,
}

#[derive(Clone, Debug)]
struct Line {
    text: String,
    original_line: Option<usize>,
}

impl Line {
    fn source(text: impl Into<String>, line: usize) -> Self {
        Self {
            text: text.into(),
            original_line: Some(line),
        }
    }

    fn blank() -> Self {
        Self {
            text: String::new(),
            original_line: None,
        }
    }

    fn issue_line(&self, fallback: usize) -> usize {
        self.original_line.unwrap_or(fallback)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum MarkerKind {
    Dash,
    Star,
    Plus,
    Ordered,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct ListMarker {
    indent: usize,
    marker_end: usize,
    kind: MarkerKind,
}

#[derive(Clone, Debug)]
struct FenceState {
    marker: char,
    len: usize,
    list_indent: Option<usize>,
    info: String,
    nested: Vec<NestedFence>,
}

#[derive(Clone, Debug)]
struct NestedFence {
    marker: char,
    len: usize,
}

#[derive(Clone, Debug)]
struct FenceMarker {
    marker: char,
    len: usize,
    indent: usize,
    info: String,
}

impl FenceState {
    fn new(marker: FenceMarker, list_indent: Option<usize>) -> Self {
        Self {
            marker: marker.marker,
            len: marker.len,
            list_indent,
            info: marker.info,
            nested: Vec::new(),
        }
    }
}

pub fn fix_markdown(input: &str) -> FixResult {
    let trailing_newline = input.ends_with('\n') || input.ends_with('\r');
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<Line> = normalized
        .lines()
        .enumerate()
        .map(|(idx, text)| Line::source(text.to_string(), idx + 1))
        .collect();

    let mut issues = Vec::new();
    normalize_indentation(&mut lines, &mut issues);
    ensure_blank_lines(&mut lines, &mut issues);

    let mut content = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    if trailing_newline && !content.is_empty() {
        content.push('\n');
    }

    FixResult { content, issues }
}

pub fn check_markdown(input: &str) -> Vec<Issue> {
    fix_markdown(input).issues
}

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

pub fn split_command_line(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), '\\') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push('\\');
                }
            }
            (Some(_), c) => current.push(c),
            (None, '"' | '\'') => quote = Some(ch),
            (None, c) if c.is_whitespace() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            (None, c) => current.push(c),
        }
    }

    if let Some(q) = quote {
        return Err(format!("unclosed quote: {q}"));
    }

    if !current.is_empty() {
        args.push(current);
    }

    Ok(args)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffOp<'old, 'new> {
    Equal,
    Delete(&'old str),
    Insert(&'new str),
}

fn colored_diff(path: &Path, old: &str, new: &str) -> String {
    if old == new {
        return String::new();
    }

    let old_lines = split_diff_lines(old);
    let new_lines = split_diff_lines(new);
    let ops = diff_ops(&old_lines, &new_lines);
    let mut body = String::new();

    let mut old_line = 1usize;
    let mut new_line = 1usize;
    let mut idx = 0usize;

    while idx < ops.len() {
        if ops[idx] == DiffOp::Equal {
            old_line += 1;
            new_line += 1;
            idx += 1;
            continue;
        }

        let old_start = old_line;
        let new_start = new_line;
        let mut old_len = 0usize;
        let mut new_len = 0usize;
        let mut hunk = Vec::new();

        while idx < ops.len() {
            match ops[idx] {
                DiffOp::Equal => break,
                DiffOp::Delete(text) => {
                    old_len += 1;
                    old_line += 1;
                    hunk.push(DiffOp::Delete(text));
                }
                DiffOp::Insert(text) => {
                    new_len += 1;
                    new_line += 1;
                    hunk.push(DiffOp::Insert(text));
                }
            }
            idx += 1;
        }

        body.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_start, old_len, new_start, new_len
        ));

        for op in hunk {
            match op {
                DiffOp::Delete(text) => {
                    body.push_str(ANSI_RED);
                    body.push('-');
                    body.push_str(text);
                    body.push_str(ANSI_RESET);
                    body.push('\n');
                }
                DiffOp::Insert(text) => {
                    body.push_str(ANSI_GREEN);
                    body.push('+');
                    body.push_str(text);
                    body.push_str(ANSI_RESET);
                    body.push('\n');
                }
                DiffOp::Equal => {}
            }
        }
    }

    if body.is_empty() {
        return String::new();
    }

    let mut diff = String::new();
    diff.push_str(&format!("diff -- mmf {}\n", path.display()));
    diff.push_str(&format!("--- {}\n", path.display()));
    diff.push_str(&format!("+++ {} (fixed)\n", path.display()));
    diff.push_str(&body);
    diff
}

fn split_diff_lines(content: &str) -> Vec<&str> {
    content.lines().collect()
}

fn diff_ops<'old, 'new>(
    old_lines: &[&'old str],
    new_lines: &[&'new str],
) -> Vec<DiffOp<'old, 'new>> {
    let old_len = old_lines.len();
    let new_len = new_lines.len();
    let stride = new_len + 1;
    let mut lcs = vec![0usize; (old_len + 1) * (new_len + 1)];

    for old_idx in (0..old_len).rev() {
        for new_idx in (0..new_len).rev() {
            let index = old_idx * stride + new_idx;
            if old_lines[old_idx] == new_lines[new_idx] {
                lcs[index] = lcs[(old_idx + 1) * stride + new_idx + 1] + 1;
            } else {
                lcs[index] =
                    lcs[(old_idx + 1) * stride + new_idx].max(lcs[old_idx * stride + new_idx + 1]);
            }
        }
    }

    let mut ops = Vec::new();
    let mut old_idx = 0usize;
    let mut new_idx = 0usize;

    while old_idx < old_len && new_idx < new_len {
        if old_lines[old_idx] == new_lines[new_idx] {
            ops.push(DiffOp::Equal);
            old_idx += 1;
            new_idx += 1;
        } else if lcs[(old_idx + 1) * stride + new_idx] >= lcs[old_idx * stride + new_idx + 1] {
            ops.push(DiffOp::Delete(old_lines[old_idx]));
            old_idx += 1;
        } else {
            ops.push(DiffOp::Insert(new_lines[new_idx]));
            new_idx += 1;
        }
    }

    while old_idx < old_len {
        ops.push(DiffOp::Delete(old_lines[old_idx]));
        old_idx += 1;
    }

    while new_idx < new_len {
        ops.push(DiffOp::Insert(new_lines[new_idx]));
        new_idx += 1;
    }

    ops
}

fn collect_inputs(paths: &[PathBuf], recursive: bool) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            collect_markdown_files(path, recursive, &mut files)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("path does not exist: {}", path.display()),
            ));
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_markdown_files(dir: &Path, recursive: bool, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if recursive {
                collect_markdown_files(&path, recursive, files)?;
            }
        } else if is_markdown_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

fn normalize_indentation(lines: &mut [Line], issues: &mut Vec<Issue>) {
    let mut fence: Option<FenceState> = None;
    let mut last_list_indent: Option<usize> = None;

    for idx in 0..lines.len() {
        let line_number = lines[idx].issue_line(idx + 1);
        let current_text = lines[idx].text.clone();

        if let Some(state) = &mut fence {
            if let Some(required_indent) = state.list_indent {
                let leading = count_leading_spaces(&current_text);
                if !current_text.trim().is_empty() && leading < required_indent {
                    let trimmed = current_text.trim_start();
                    lines[idx].text = format!("{}{}", " ".repeat(required_indent), trimmed);
                    issues.push(Issue::new(
                        line_number,
                        "code block inside list item must be indented by four spaces",
                    ));
                }
            }

            if advances_or_closes_fence(state, &lines[idx].text) {
                fence = None;
            }
            continue;
        }

        if let Some(marker) = fence_marker(&current_text) {
            let list_indent = if is_probably_in_list(lines, idx) {
                Some(normalized_list_child_indent(marker.indent))
            } else {
                None
            };

            if let Some(required) = list_indent {
                if marker.indent != required {
                    lines[idx].text = replace_indent(&current_text, required);
                    issues.push(Issue::new(
                        line_number,
                        "code block inside list item must be indented by four spaces",
                    ));
                }
            }

            fence = Some(FenceState::new(marker, list_indent));
            continue;
        }

        if let Some(marker) = list_marker(&current_text) {
            if marker.indent > 0 && marker.indent % 4 != 0 {
                let required = normalized_list_child_indent(marker.indent);
                lines[idx].text = replace_indent(&current_text, required);
                issues.push(Issue::new(
                    line_number,
                    "nested list indentation must use multiples of four spaces",
                ));
                last_list_indent = Some(required);
            } else {
                last_list_indent = Some(marker.indent);
            }
            continue;
        }

        if let Some((indent, _)) = indented_blockquote(&current_text) {
            if indent % 4 != 0 && is_probably_in_list(lines, idx) {
                let required = normalized_list_child_indent(indent);
                lines[idx].text = replace_indent(&current_text, required);
                issues.push(Issue::new(
                    line_number,
                    "blockquote inside list item must be indented by four spaces",
                ));
            }
            continue;
        }

        if !current_text.trim().is_empty() && count_leading_spaces(&current_text) == 0 {
            last_list_indent = None;
        } else if current_text.trim().is_empty() {
            let _ = last_list_indent;
        }
    }
}

fn ensure_blank_lines(lines: &mut Vec<Line>, issues: &mut Vec<Issue>) {
    let mut output = Vec::with_capacity(lines.len());
    let mut idx = 0;
    let mut fence: Option<FenceState> = None;
    let mut math_open = false;

    while idx < lines.len() {
        let line = lines[idx].clone();
        let line_number = line.issue_line(idx + 1);

        if let Some(state) = &mut fence {
            output.push(line.clone());
            if advances_or_closes_fence(state, &line.text) {
                fence = None;
                if next_exists_and_not_blank(lines, idx) {
                    output.push(Line::blank());
                    issues.push(Issue::new(
                        line_number,
                        "fenced code block must be followed by a blank line",
                    ));
                }
            }
            idx += 1;
            continue;
        }

        if math_open {
            output.push(line.clone());
            if is_math_delimiter(&line.text) {
                math_open = false;
                if next_exists_and_not_blank(lines, idx) {
                    output.push(Line::blank());
                    issues.push(Issue::new(
                        line_number,
                        "display math block must be followed by a blank line",
                    ));
                }
            }
            idx += 1;
            continue;
        }

        if let Some(marker) = fence_marker(&line.text) {
            if prev_output_not_blank(&output) {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "fenced code block must be preceded by a blank line",
                ));
            }
            output.push(line.clone());
            fence = Some(FenceState::new(marker, None));
            idx += 1;
            continue;
        }

        if is_one_line_display_math(&line.text) {
            if prev_output_not_blank(&output) {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "display math block must be preceded by a blank line",
                ));
            }
            output.push(line.clone());
            if next_exists_and_not_blank(lines, idx) {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "display math block must be followed by a blank line",
                ));
            }
            idx += 1;
            continue;
        }

        if is_math_delimiter(&line.text) {
            if prev_output_not_blank(&output) {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "display math block must be preceded by a blank line",
                ));
            }
            output.push(line.clone());
            math_open = true;
            idx += 1;
            continue;
        }

        if is_table_row(&line.text) {
            if prev_output_not_blank(&output) {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "table must be preceded by a blank line",
                ));
            }

            let start = idx;
            while idx < lines.len() && is_table_row(&lines[idx].text) {
                output.push(lines[idx].clone());
                idx += 1;
            }

            let table_line = lines[start].issue_line(start + 1);
            if idx < lines.len() && !lines[idx].text.trim().is_empty() {
                output.push(Line::blank());
                issues.push(Issue::new(
                    table_line,
                    "table must be followed by a blank line",
                ));
            }
            continue;
        }

        if is_indented_blockquote(&line.text) {
            if prev_output_not_blank(&output) {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "blockquote inside list item must be preceded by a blank line",
                ));
            }

            let start = idx;
            while idx < lines.len() && is_indented_blockquote(&lines[idx].text) {
                output.push(lines[idx].clone());
                idx += 1;
            }

            let quote_line = lines[start].issue_line(start + 1);
            if idx < lines.len() && !lines[idx].text.trim().is_empty() {
                output.push(Line::blank());
                issues.push(Issue::new(
                    quote_line,
                    "blockquote inside list item must be followed by a blank line",
                ));
            }
            continue;
        }

        if let Some(marker) = list_marker(&line.text) {
            if marker.indent == 0 && prev_output_not_blank(&output) && !prev_output_is_list(&output)
            {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "list block must be preceded by a blank line",
                ));
            }

            output.push(line);

            if list_block_ends_before(lines, idx, marker) {
                output.push(Line::blank());
                issues.push(Issue::new(
                    line_number,
                    "list block must be followed by a blank line",
                ));
            }

            idx += 1;
            continue;
        }

        output.push(line);
        idx += 1;
    }

    *lines = output;
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

fn count_leading_spaces(text: &str) -> usize {
    text.as_bytes()
        .iter()
        .take_while(|byte| **byte == b' ')
        .count()
}

fn replace_indent(text: &str, indent: usize) -> String {
    let trimmed = text.trim_start_matches(' ');
    format!("{}{}", " ".repeat(indent), trimmed)
}

fn normalized_list_child_indent(indent: usize) -> usize {
    if indent == 0 {
        4
    } else {
        ((indent + 3) / 4) * 4
    }
}

fn list_marker(text: &str) -> Option<ListMarker> {
    let indent = count_leading_spaces(text);
    let rest = &text[indent..];
    let bytes = rest.as_bytes();
    let first = *bytes.first()?;

    match first {
        b'-' | b'*' | b'+' => {
            if bytes
                .get(1)
                .map(|b| b.is_ascii_whitespace())
                .unwrap_or(false)
            {
                let kind = match first {
                    b'-' => MarkerKind::Dash,
                    b'*' => MarkerKind::Star,
                    _ => MarkerKind::Plus,
                };
                Some(ListMarker {
                    indent,
                    marker_end: indent + 1,
                    kind,
                })
            } else {
                None
            }
        }
        b'0'..=b'9' => {
            let digit_count = bytes
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count();
            let marker = bytes.get(digit_count)?;
            let after = bytes.get(digit_count + 1)?;
            if matches!(marker, b'.' | b')') && after.is_ascii_whitespace() {
                Some(ListMarker {
                    indent,
                    marker_end: indent + digit_count + 1,
                    kind: MarkerKind::Ordered,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn indented_blockquote(text: &str) -> Option<(usize, usize)> {
    let indent = count_leading_spaces(text);
    if indent == 0 {
        return None;
    }

    let rest = &text[indent..];
    if rest.starts_with('>') {
        Some((indent, indent + 1))
    } else {
        None
    }
}

fn is_indented_blockquote(text: &str) -> bool {
    indented_blockquote(text)
        .map(|(indent, _)| indent >= 4 && indent % 4 == 0)
        .unwrap_or(false)
}

fn fence_marker(text: &str) -> Option<FenceMarker> {
    let indent = count_leading_spaces(text);
    let rest = text[indent..].trim_end();
    let marker = rest.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }

    let len = rest.chars().take_while(|ch| *ch == marker).count();
    if len < 3 {
        return None;
    }

    let info = rest
        .char_indices()
        .nth(len)
        .map(|(idx, _)| rest[idx..].trim().to_string())
        .unwrap_or_default();

    Some(FenceMarker {
        marker,
        len,
        indent,
        info,
    })
}

fn advances_or_closes_fence(state: &mut FenceState, text: &str) -> bool {
    if is_markdown_fence_info(&state.info) {
        if let Some(nested) = state.nested.last() {
            if is_closing_fence(text, nested.marker, nested.len) {
                state.nested.pop();
            }
            return false;
        }

        if is_closing_fence(text, state.marker, state.len) {
            return true;
        }

        if let Some(marker) = fence_marker(text) {
            if !marker.info.is_empty() {
                state.nested.push(NestedFence {
                    marker: marker.marker,
                    len: marker.len,
                });
            }
        }

        return false;
    }

    is_closing_fence(text, state.marker, state.len)
}

fn is_closing_fence(text: &str, marker: char, required_len: usize) -> bool {
    let rest = text.trim_start();
    let len = rest.chars().take_while(|ch| *ch == marker).count();
    if len < required_len {
        return false;
    }

    rest.chars().skip(len).all(char::is_whitespace)
}

fn is_markdown_fence_info(info: &str) -> bool {
    info.split_whitespace()
        .next()
        .map(|lang| lang.eq_ignore_ascii_case("markdown"))
        .unwrap_or(false)
}

fn is_math_delimiter(text: &str) -> bool {
    text.trim() == "$$"
}

fn is_one_line_display_math(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.len() > 4 && trimmed.starts_with("$$") && trimmed.ends_with("$$")
}

fn is_table_row(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2
}

fn prev_output_not_blank(output: &[Line]) -> bool {
    output
        .last()
        .map(|line| !line.text.trim().is_empty())
        .unwrap_or(false)
}

fn prev_output_is_list(output: &[Line]) -> bool {
    output
        .last()
        .and_then(|line| list_marker(&line.text))
        .is_some()
}

fn next_exists_and_not_blank(lines: &[Line], idx: usize) -> bool {
    lines
        .get(idx + 1)
        .map(|line| !line.text.trim().is_empty())
        .unwrap_or(false)
}

fn list_block_ends_before(lines: &[Line], idx: usize, marker: ListMarker) -> bool {
    let Some(next) = lines.get(idx + 1) else {
        return false;
    };

    if next.text.trim().is_empty() || list_marker(&next.text).is_some() {
        return false;
    }

    count_leading_spaces(&next.text) <= marker.indent
}

fn is_probably_in_list(lines: &[Line], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }

    let current_indent = count_leading_spaces(&lines[idx].text);
    let mut pos = idx;
    while pos > 0 {
        pos -= 1;
        let previous = &lines[pos].text;
        if previous.trim().is_empty() {
            continue;
        }

        if let Some(marker) = list_marker(previous) {
            return marker.indent < current_indent || marker.indent == 0;
        }

        let previous_indent = count_leading_spaces(previous);
        return previous_indent >= 4 && previous_indent <= current_indent;
    }

    false
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.line, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_blank_before_top_level_list() {
        let input = "Intro:\n- item\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, "Intro:\n\n- item\n");
        assert!(fixed
            .issues
            .iter()
            .any(|issue| issue.message.contains("list block")));
    }

    #[test]
    fn inserts_blank_after_top_level_list() {
        let input = "Intro:\n\n- item\nEnd\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, "Intro:\n\n- item\n\nEnd\n");
        assert!(fixed
            .issues
            .iter()
            .any(|issue| issue.message.contains("followed by a blank line")));
    }

    #[test]
    fn does_not_split_nested_list_items() {
        let input = "- item\n    - child\n- next\nEnd\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, "- item\n    - child\n- next\n\nEnd\n");
    }

    #[test]
    fn normalizes_nested_list_indent() {
        let input = "- item\n  - child\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, "- item\n    - child\n");
        assert!(fixed
            .issues
            .iter()
            .any(|issue| issue.message.contains("Nested list")
                || issue.message.contains("nested list")));
    }

    #[test]
    fn fixes_blockquote_inside_list() {
        let input = "1. item\n   > quote\n2. item\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, "1. item\n\n    > quote\n\n2. item\n");
    }

    #[test]
    fn surrounds_fenced_code_block() {
        let input = "Before:\n```python\nprint('hi')\n```\nAfter\n";
        let fixed = fix_markdown(input);

        assert_eq!(
            fixed.content,
            "Before:\n\n```python\nprint('hi')\n```\n\nAfter\n"
        );
    }

    #[test]
    fn does_not_insert_blank_lines_inside_fenced_code_block() {
        let input = "```python\nprint(1)\n```\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, input);
        assert!(fixed.issues.is_empty());
    }

    #[test]
    fn fixes_list_code_block_indent_and_spacing() {
        let input = "- item\n   ```python\nprint('hi')\n   ```\n- next\n";
        let fixed = fix_markdown(input);

        assert_eq!(
            fixed.content,
            "- item\n\n    ```python\n    print('hi')\n    ```\n\n- next\n"
        );
    }

    #[test]
    fn surrounds_display_math() {
        let input = "Formula:\n$$E = mc^2$$\nDone\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, "Formula:\n\n$$E = mc^2$$\n\nDone\n");
    }

    #[test]
    fn surrounds_table() {
        let input = "Data:\n| Name | Age |\n| --- | --- |\n| Alice | 30 |\nDone\n";
        let fixed = fix_markdown(input);

        assert_eq!(
            fixed.content,
            "Data:\n\n| Name | Age |\n| --- | --- |\n| Alice | 30 |\n\nDone\n"
        );
    }

    #[test]
    fn ignores_markdown_inside_fenced_code_block() {
        let input = "````markdown\nText:\n- item\n````\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, input);
        assert!(fixed.issues.is_empty());
    }

    #[test]
    fn ignores_nested_fences_inside_markdown_demo_block() {
        let input = "```markdown\nText:\n```python\nprint(1)\n```\n- not checked\n```\n";
        let fixed = fix_markdown(input);

        assert_eq!(fixed.content, input);
        assert!(fixed.issues.is_empty());
    }

    #[test]
    fn colored_diff_prints_only_changed_lines() {
        let diff = colored_diff(
            Path::new("sample.md"),
            "- item\n  - child\n",
            "- item\n    - child\n",
        );

        assert!(diff.contains("\x1b[31m-  - child\x1b[0m"));
        assert!(diff.contains("\x1b[32m+    - child\x1b[0m"));
        assert!(!diff.contains("\n- item\n"));
    }

    #[test]
    fn colored_diff_shows_inserted_blank_line() {
        let diff = colored_diff(
            Path::new("sample.md"),
            "Intro:\n- item\n",
            "Intro:\n\n- item\n",
        );

        assert!(diff.contains("@@ -2,0 +2,1 @@"));
        assert!(diff.contains("\x1b[32m+\x1b[0m"));
        assert!(!diff.contains("Intro:"));
    }

    #[test]
    fn shell_parser_handles_quoted_paths() {
        let args = split_command_line("check \"docs/my file.md\" -r").unwrap();
        assert_eq!(args, vec!["check", "docs/my file.md", "-r"]);
    }
}
