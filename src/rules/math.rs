//! Display math block spacing (specification II).

use crate::model::Issue;
use crate::syntax::{self, Line};

pub(super) fn append_block(
    lines: &[Line],
    start: usize,
    output: &mut Vec<Line>,
    issues: &mut Vec<Issue>,
) -> usize {
    const KIND: &str = "display math block";
    let line = lines[start].issue_line(start + 1);
    super::blank_before(output, issues, line, KIND);
    output.push(lines[start].clone());
    if syntax::is_one_line_display_math(&lines[start].text) {
        super::blank_after(lines, start, output, issues, line, KIND);
        return start + 1;
    }
    let mut idx = start + 1;
    while idx < lines.len() {
        output.push(lines[idx].clone());
        let closes = syntax::is_math_delimiter(&lines[idx].text);
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
