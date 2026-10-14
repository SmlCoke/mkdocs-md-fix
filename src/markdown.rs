use crate::model::{FixResult, Issue};
use crate::rules::{ensure_blank_lines, normalize_indentation};
use crate::syntax::Line;

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
    crate::rules::spacing::normalize_spaces(&mut lines, &mut issues);
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
