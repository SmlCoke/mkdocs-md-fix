//! List indentation and block spacing (specification 1.1–1.2).

use crate::model::Issue;
use crate::syntax::{self, Line, ListMarker};

pub(super) fn normalize_indent(
    line: &mut Line,
    idx: usize,
    marker: ListMarker,
    issues: &mut Vec<Issue>,
) {
    if marker.indent > 0 && marker.indent % 4 != 0 {
        let required = syntax::normalized_list_child_indent(marker.indent);
        line.text = syntax::replace_indent(&line.text, required);
        issues.push(Issue::new(
            line.issue_line(idx + 1),
            "nested list indentation must use multiples of four spaces",
        ));
    }
}

pub(super) fn append_item(
    lines: &[Line],
    idx: usize,
    marker: ListMarker,
    output: &mut Vec<Line>,
    issues: &mut Vec<Issue>,
) {
    let line = lines[idx].issue_line(idx + 1);
    if marker.indent == 0 && !syntax::prev_output_is_list(output) {
        super::blank_before(output, issues, line, "list block");
    }
    output.push(lines[idx].clone());
    if syntax::list_block_ends_before(lines, idx, marker) {
        super::blank_after(lines, idx, output, issues, line, "list block");
    }
}
