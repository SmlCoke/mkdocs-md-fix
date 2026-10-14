//! Mark code, math and Markdown structural whitespace before editing prose.

use super::{advances_or_closes_fence, fence_marker, list_marker, FenceState};

pub(crate) fn protected_mask(text: &str) -> Vec<bool> {
    let mut mask = vec![false; text.len()];
    let mut offset = 0;
    let mut fence: Option<FenceState> = None;
    let mut list_indent: Option<usize> = None;
    let mut indented_code: Option<usize> = None;
    let mut previous_blank = true;

    for raw in text.split_inclusive('\n') {
        let line = raw.trim_end_matches('\n');
        let end = offset + line.len();
        let quoted = quote_content_start(line);
        let payload = &line[quoted..];
        let indent = payload.bytes().take_while(|b| *b == b' ').count();
        let marker = list_marker(payload);
        let prefix = structural_prefix_end(line, quoted);

        // Prefix indentation, list padding and hard line breaks are syntax, not prose.
        mask[offset..offset + prefix].fill(true);
        let trailing = line.len() - line.trim_end_matches([' ', '\t']).len();
        mask[end - trailing..end].fill(true);

        if let Some(state) = &mut fence {
            mask[offset..offset + raw.len()].fill(true);
            if advances_or_closes_fence(state, payload) {
                fence = None;
            }
        } else if !mask.get(offset + prefix).copied().unwrap_or(false) {
            let required = list_indent.map_or(4, |base| base + 8);
            // Preserve the same fence recognition priority as the existing rule engine.
            if let Some(opening) = fence_marker(&line[prefix..]) {
                fence = Some(FenceState::new(opening, None));
                indented_code = None;
                mask[offset..offset + raw.len()].fill(true);
            } else if marker.is_none()
                && indent >= required
                && (previous_blank || indented_code.is_some())
            {
                indented_code = Some(required);
                mask[offset..offset + raw.len()].fill(true);
            } else if payload.trim().is_empty() && indented_code.is_some() {
                mask[offset..offset + raw.len()].fill(true);
            } else {
                indented_code = None;
                mark_inline(text, offset + prefix, end, &mut mask);
            }
        } else {
            // A multiline inline span may end here, followed by another protected span.
            mark_inline(text, offset + prefix, end, &mut mask);
        }

        if let Some(marker) = marker {
            list_indent = Some(marker.indent);
        } else if !payload.trim().is_empty() && indent == 0 && fence.is_none() {
            list_indent = None;
        }
        previous_blank = payload.trim().is_empty();
        offset += raw.len();
    }
    mask
}

fn quote_content_start(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut pos = 0;
    loop {
        let mut candidate = pos;
        while bytes
            .get(candidate)
            .is_some_and(|b| matches!(b, b' ' | b'\t'))
        {
            candidate += 1;
        }
        if bytes.get(candidate) != Some(&b'>') {
            return pos;
        }
        pos = candidate + 1;
        if bytes.get(pos) == Some(&b' ') {
            pos += 1;
        }
    }
}

fn structural_prefix_end(line: &str, quoted: usize) -> usize {
    let mut pos = quoted;
    if let Some(marker) = list_marker(&line[quoted..]) {
        pos += marker.marker_end;
    }
    while line
        .as_bytes()
        .get(pos)
        .is_some_and(|b| matches!(b, b' ' | b'\t'))
    {
        pos += 1;
    }
    pos
}

fn escaped(bytes: &[u8], pos: usize) -> bool {
    let backslashes = bytes[..pos]
        .iter()
        .rev()
        .take_while(|b| **b == b'\\')
        .count();
    backslashes % 2 == 1
}

fn run_len(bytes: &[u8], pos: usize) -> usize {
    bytes[pos..]
        .iter()
        .take_while(|b| **b == bytes[pos])
        .count()
}

fn paragraph_boundary(bytes: &[u8], pos: usize) -> bool {
    if bytes[pos] != b'\n' {
        return false;
    }
    let end = bytes[pos + 1..]
        .iter()
        .position(|b| *b == b'\n')
        .map_or(bytes.len(), |len| pos + 1 + len);
    let next = std::str::from_utf8(&bytes[pos + 1..end]).expect("input is UTF-8");
    let quoted = quote_content_start(next);
    let prefix = structural_prefix_end(next, quoted);
    next[quoted..].trim().is_empty() || fence_marker(&next[prefix..]).is_some()
}

fn matching_run(bytes: &[u8], start: usize, len: usize, code: bool) -> Option<usize> {
    let marker = bytes[start];
    let mut pos = start + len;
    while pos < bytes.len() {
        // Inline spans cannot span paragraph boundaries; display math can.
        if (code || len == 1) && paragraph_boundary(bytes, pos) {
            return None;
        }
        if bytes[pos] == marker {
            let found = run_len(bytes, pos);
            let currency = len == 1 && bytes.get(pos + 1).is_some_and(u8::is_ascii_digit);
            if found == len && (code || (!escaped(bytes, pos) && !currency)) {
                return Some(pos + len);
            }
            pos += found;
        } else {
            pos += 1;
        }
    }
    None
}

fn mark_inline(text: &str, start: usize, line_end: usize, mask: &mut [bool]) {
    let bytes = text.as_bytes();
    let mut pos = start;
    while pos < line_end {
        if mask[pos] {
            pos += 1;
            continue;
        }
        let mut close = None;
        if matches!(bytes[pos], b'`' | b'$') && !escaped(bytes, pos) {
            let len = run_len(bytes, pos);
            let code = bytes[pos] == b'`';
            if code || len <= 2 {
                close = matching_run(bytes, pos, len, code);
                // An unfinished display math block still protects its contents.
                if close.is_none() && !code && len == 2 {
                    close = Some(bytes.len());
                }
            }
            if close.is_none() {
                pos += len;
                continue;
            }
        } else if bytes[pos] == b'\\' && !escaped(bytes, pos) {
            let pair = match bytes.get(pos + 1) {
                Some(b'(') => Some((b"\\)".as_slice(), false)),
                Some(b'[') => Some((b"\\]".as_slice(), true)),
                _ => None,
            };
            if let Some((delimiter, display)) = pair {
                let mut cursor = pos + 2;
                while cursor + 1 < bytes.len() {
                    if !display && paragraph_boundary(bytes, cursor) {
                        break;
                    }
                    if bytes[cursor..].starts_with(delimiter) && !escaped(bytes, cursor) {
                        close = Some(cursor + 2);
                        break;
                    }
                    cursor += 1;
                }
                if close.is_none() && display {
                    close = Some(bytes.len());
                }
            }
        }
        if let Some(end) = close {
            mask[pos..end].fill(true);
            pos = end;
        } else {
            pos += 1;
        }
    }
}
