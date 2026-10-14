//! Block quotes in list items (specification 1.3).

use crate::model::Issue;
use crate::syntax::{self, Line};

pub(super) fn normalize_indent(
    lines: &mut [Line],
    idx: usize,
    indent: usize,
    issues: &mut Vec<Issue>,
) {
    if indent % 4 != 0 && syntax::is_probably_in_list(lines, idx) {
        let required = syntax::normalized_list_child_indent(indent);
        lines[idx].text = syntax::replace_indent(&lines[idx].text, required);
        issues.push(Issue::new(
            lines[idx].issue_line(idx + 1),
            "blockquote inside list item must be indented by four spaces",
        ));
    }
}

pub(super) fn append_block(
    lines: &[Line],
    start: usize,
    output: &mut Vec<Line>,
    issues: &mut Vec<Issue>,
) -> usize {
    super::append_run(
        lines,
        start,
        output,
        issues,
        "blockquote inside list item",
        syntax::is_indented_blockquote,
    )
}
