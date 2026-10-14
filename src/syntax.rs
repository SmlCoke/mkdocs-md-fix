mod protected;

pub(crate) use protected::protected_mask;

#[derive(Clone, Debug)]
pub(crate) struct Line {
    pub(crate) text: String,
    pub(crate) original_line: Option<usize>,
}

impl Line {
    pub(crate) fn source(text: impl Into<String>, line: usize) -> Self {
        Self {
            text: text.into(),
            original_line: Some(line),
        }
    }

    pub(crate) fn blank() -> Self {
        Self {
            text: String::new(),
            original_line: None,
        }
    }

    pub(crate) fn issue_line(&self, fallback: usize) -> usize {
        self.original_line.unwrap_or(fallback)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum MarkerKind {
    Dash,
    Star,
    Plus,
    Ordered,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ListMarker {
    pub(crate) indent: usize,
    pub(crate) marker_end: usize,
    pub(crate) kind: MarkerKind,
}

#[derive(Clone, Debug)]
pub(crate) struct FenceState {
    pub(crate) marker: char,
    pub(crate) len: usize,
    pub(crate) list_indent: Option<usize>,
    pub(crate) info: String,
    pub(crate) nested: Vec<NestedFence>,
}

#[derive(Clone, Debug)]
pub(crate) struct NestedFence {
    pub(crate) marker: char,
    pub(crate) len: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct FenceMarker {
    pub(crate) marker: char,
    pub(crate) len: usize,
    pub(crate) indent: usize,
    pub(crate) info: String,
}

impl FenceState {
    pub(crate) fn new(marker: FenceMarker, list_indent: Option<usize>) -> Self {
        Self {
            marker: marker.marker,
            len: marker.len,
            list_indent,
            info: marker.info,
            nested: Vec::new(),
        }
    }
}

pub(crate) fn count_leading_spaces(text: &str) -> usize {
    text.as_bytes()
        .iter()
        .take_while(|byte| **byte == b' ')
        .count()
}

pub(crate) fn replace_indent(text: &str, indent: usize) -> String {
    let trimmed = text.trim_start_matches(' ');
    format!("{}{}", " ".repeat(indent), trimmed)
}

pub(crate) fn normalized_list_child_indent(indent: usize) -> usize {
    if indent == 0 {
        4
    } else {
        ((indent + 3) / 4) * 4
    }
}

pub(crate) fn list_marker(text: &str) -> Option<ListMarker> {
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

pub(crate) fn indented_blockquote(text: &str) -> Option<(usize, usize)> {
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

pub(crate) fn is_indented_blockquote(text: &str) -> bool {
    indented_blockquote(text)
        .map(|(indent, _)| indent >= 4 && indent % 4 == 0)
        .unwrap_or(false)
}

pub(crate) fn fence_marker(text: &str) -> Option<FenceMarker> {
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

pub(crate) fn advances_or_closes_fence(state: &mut FenceState, text: &str) -> bool {
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

pub(crate) fn is_closing_fence(text: &str, marker: char, required_len: usize) -> bool {
    let rest = text.trim_start();
    let len = rest.chars().take_while(|ch| *ch == marker).count();
    if len < required_len {
        return false;
    }

    rest.chars().skip(len).all(char::is_whitespace)
}

pub(crate) fn is_markdown_fence_info(info: &str) -> bool {
    info.split_whitespace()
        .next()
        .map(|lang| lang.eq_ignore_ascii_case("markdown"))
        .unwrap_or(false)
}

pub(crate) fn is_math_delimiter(text: &str) -> bool {
    text.trim() == "$$"
}

pub(crate) fn is_one_line_display_math(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.len() > 4 && trimmed.starts_with("$$") && trimmed.ends_with("$$")
}

pub(crate) fn is_table_row(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2
}

pub(crate) fn prev_output_not_blank(output: &[Line]) -> bool {
    output
        .last()
        .map(|line| !line.text.trim().is_empty())
        .unwrap_or(false)
}

pub(crate) fn prev_output_is_list(output: &[Line]) -> bool {
    output
        .last()
        .and_then(|line| list_marker(&line.text))
        .is_some()
}

pub(crate) fn next_exists_and_not_blank(lines: &[Line], idx: usize) -> bool {
    lines
        .get(idx + 1)
        .map(|line| !line.text.trim().is_empty())
        .unwrap_or(false)
}

pub(crate) fn list_block_ends_before(lines: &[Line], idx: usize, marker: ListMarker) -> bool {
    let Some(next) = lines.get(idx + 1) else {
        return false;
    };

    if next.text.trim().is_empty() || list_marker(&next.text).is_some() {
        return false;
    }

    count_leading_spaces(&next.text) <= marker.indent
}

pub(crate) fn is_probably_in_list(lines: &[Line], idx: usize) -> bool {
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
