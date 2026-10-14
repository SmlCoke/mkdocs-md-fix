use std::path::Path;

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RESET: &str = "\x1b[0m";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffOp<'old, 'new> {
    Equal,
    Delete(&'old str),
    Insert(&'new str),
}

pub(crate) fn colored_diff(path: &Path, old: &str, new: &str) -> String {
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
