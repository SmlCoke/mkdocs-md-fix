//! Fenced code blocks and code indentation in lists (specification 1.4 and III).

use crate::model::Issue;
use crate::syntax::{self, FenceMarker, FenceState, Line};

pub(super) fn normalize_content(
    line: &mut Line,
    idx: usize,
    state: &FenceState,
    issues: &mut Vec<Issue>,
) {
    if let Some(required) = state.list_indent {
        if !line.text.trim().is_empty() && syntax::count_leading_spaces(&line.text) < required {
            line.text = format!("{}{}", " ".repeat(required), line.text.trim_start());
            issues.push(Issue::new(
                line.issue_line(idx + 1),
                "code block inside list item must be indented by four spaces",
            ));
        }
    }
}

pub(super) fn normalize_opening(
    lines: &mut [Line],
    idx: usize,
    marker: FenceMarker,
    issues: &mut Vec<Issue>,
) -> FenceState {
    let list_indent = if syntax::is_probably_in_list(lines, idx) {
        Some(syntax::normalized_list_child_indent(marker.indent))
    } else {
        None
    };
    if let Some(required) = list_indent {
        if marker.indent != required {
            lines[idx].text = syntax::replace_indent(&lines[idx].text, required);
            issues.push(Issue::new(
                lines[idx].issue_line(idx + 1),
                "code block inside list item must be indented by four spaces",
            ));
        }
    }
    FenceState::new(marker, list_indent)
}

pub(super) fn append_block(
    lines: &[Line],
    start: usize,
    marker: FenceMarker,
    output: &mut Vec<Line>,
    issues: &mut Vec<Issue>,
) -> usize {
    const KIND: &str = "fenced code block";
    super::blank_before(output, issues, lines[start].issue_line(start + 1), KIND);
    output.push(lines[start].clone());
    let mut state = FenceState::new(marker, None);
    let mut idx = start + 1;
    while idx < lines.len() {
        output.push(lines[idx].clone());
        let closes = syntax::advances_or_closes_fence(&mut state, &lines[idx].text);
        idx += 1;
        if closes {
            super::blank_after(
                lines,
                idx - 1,
                output,
                issues,
                lines[idx - 1].issue_line(idx),
                KIND,
            );
            break;
        }
    }
    idx
}
