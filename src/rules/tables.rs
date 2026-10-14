//! Table block spacing (specification IV).

use crate::model::Issue;
use crate::syntax::{self, Line};

pub(super) fn append_block(
    lines: &[Line],
    start: usize,
    output: &mut Vec<Line>,
    issues: &mut Vec<Issue>,
) -> usize {
    super::append_run(lines, start, output, issues, "table", syntax::is_table_row)
}
