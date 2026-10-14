//! Rule dispatch and shared block-boundary helpers.

mod code;
mod lists;
mod math;
mod quotes;
pub(crate) mod spacing;
mod tables;

use crate::model::Issue;
use crate::syntax::{self, FenceState, Line};

pub(crate) fn normalize_indentation(lines: &mut [Line], issues: &mut Vec<Issue>) {
    let mut fence: Option<FenceState> = None;
    for idx in 0..lines.len() {
        if let Some(state) = &mut fence {
            code::normalize_content(&mut lines[idx], idx, state, issues);
            if syntax::advances_or_closes_fence(state, &lines[idx].text) {
                fence = None;
            }
        } else if let Some(marker) = syntax::fence_marker(&lines[idx].text) {
            fence = Some(code::normalize_opening(lines, idx, marker, issues));
        } else if let Some(marker) = syntax::list_marker(&lines[idx].text) {
            lists::normalize_indent(&mut lines[idx], idx, marker, issues);
        } else if let Some((indent, _)) = syntax::indented_blockquote(&lines[idx].text) {
            quotes::normalize_indent(lines, idx, indent, issues);
        }
    }
}

pub(crate) fn ensure_blank_lines(lines: &mut Vec<Line>, issues: &mut Vec<Issue>) {
    let mut output = Vec::with_capacity(lines.len());
    let mut idx = 0;
    while idx < lines.len() {
        let text = &lines[idx].text;
        idx = if let Some(marker) = syntax::fence_marker(text) {
            code::append_block(lines, idx, marker, &mut output, issues)
        } else if syntax::is_one_line_display_math(text) || syntax::is_math_delimiter(text) {
            math::append_block(lines, idx, &mut output, issues)
        } else if syntax::is_table_row(text) {
            tables::append_block(lines, idx, &mut output, issues)
        } else if syntax::is_indented_blockquote(text) {
            quotes::append_block(lines, idx, &mut output, issues)
        } else if let Some(marker) = syntax::list_marker(text) {
            lists::append_item(lines, idx, marker, &mut output, issues);
            idx + 1
        } else {
            output.push(lines[idx].clone());
            idx + 1
        };
    }
    *lines = output;
}

fn blank_before(output: &mut Vec<Line>, issues: &mut Vec<Issue>, line: usize, kind: &str) {
    if syntax::prev_output_not_blank(output) {
        output.push(Line::blank());
        issues.push(Issue::new(
            line,
            format!("{kind} must be preceded by a blank line"),
        ));
    }
}

fn blank_after(
    lines: &[Line],
    end: usize,
    output: &mut Vec<Line>,
    issues: &mut Vec<Issue>,
    line: usize,
    kind: &str,
) {
    if syntax::next_exists_and_not_blank(lines, end) {
        output.push(Line::blank());
        issues.push(Issue::new(
            line,
            format!("{kind} must be followed by a blank line"),
        ));
    }
}

fn append_run(
    lines: &[Line],
    start: usize,
    output: &mut Vec<Line>,
    issues: &mut Vec<Issue>,
    kind: &str,
    matches: fn(&str) -> bool,
) -> usize {
    let line = lines[start].issue_line(start + 1);
    blank_before(output, issues, line, kind);
    let mut end = start;
    while end < lines.len() && matches(&lines[end].text) {
        output.push(lines[end].clone());
        end += 1;
    }
    blank_after(lines, end - 1, output, issues, line, kind);
    end
}
