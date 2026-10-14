//! Abnormal prose spaces (specification 5.1).

use crate::model::Issue;
use crate::syntax::{protected_mask, Line};

pub(crate) fn normalize_spaces(lines: &mut [Line], issues: &mut Vec<Issue>) {
    let text = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let protected = protected_mask(&text);
    let mut offset = 0;

    for (idx, line) in lines.iter_mut().enumerate() {
        let original_len = line.text.len();
        let mut output = String::with_capacity(original_len);
        let mut chars = line.text.char_indices().peekable();
        let mut chinese_gap = false;
        let mut repeated = false;
        while let Some((start, ch)) = chars.next() {
            if ch != ' ' || protected[offset + start] {
                output.push(ch);
                continue;
            }
            let mut count = 1;
            while chars
                .peek()
                .is_some_and(|(pos, ch)| *ch == ' ' && !protected[offset + pos])
            {
                chars.next();
                count += 1;
            }
            let left = output.chars().next_back();
            let right = chars.peek().map(|(_, ch)| *ch);
            let remove = left.is_some_and(is_chinese) && right.is_some_and(is_chinese);
            repeated |= count > 1;
            chinese_gap |= remove;
            if !remove {
                output.push(' ');
            }
        }
        if chinese_gap {
            issues.push(Issue::new(
                line.issue_line(idx + 1),
                "spaces between Chinese characters or punctuation must be removed",
            ));
        }
        if repeated {
            issues.push(Issue::new(
                line.issue_line(idx + 1),
                "consecutive spaces in prose must be collapsed to one space",
            ));
        }
        line.text = output;
        offset += original_len + 1;
    }
}

fn is_chinese(ch: char) -> bool {
    // Han and compatibility ideograph blocks, including extensions A–J:
    // https://www.unicode.org/Public/17.0.0/ucd/Blocks.txt
    matches!(ch,
        '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' |
        '\u{f900}'..='\u{faff}' | '\u{20000}'..='\u{2a6df}' |
        '\u{2a700}'..='\u{2ee5f}' | '\u{2f800}'..='\u{2fa1f}' |
        '\u{30000}'..='\u{3347f}' |
        '〇' | '，' | '。' | '、' | '；' | '：' | '！' | '？' |
        '（' | '）' | '【' | '】' | '《' | '》' | '〈' | '〉' |
        '「' | '」' | '『' | '』' | '〔' | '〕' | '〖' | '〗' |
        '〘' | '〙' | '〚' | '〛' | '“' | '”' | '‘' | '’' |
        '…' | '—' | '～' | '·'
    )
}

#[cfg(test)]
mod tests {
    use crate::{check_markdown, fix_markdown};

    fn assert_fix(input: &str, expected: &str) {
        let fixed = fix_markdown(input);
        assert_eq!(fixed.content, expected);
        assert!(fixed
            .issues
            .iter()
            .any(|issue| issue.message.contains("spaces")));
        let again = fix_markdown(expected);
        assert_eq!(again.content, expected, "fixes must be idempotent");
        assert!(again.issues.is_empty(), "fixed text should pass check");
    }

    #[test]
    fn fixes_specification_examples() {
        let correct = "CXL.mem 允许主机处理器将连接的 CXL 设备内存视为其内存地址空间的一部分。\n";
        for input in [
            "CXL.mem 允许主机处理器将连接的 CXL 设备内 存 视 为 其 内 存 地 址 空 间 的 一 部 分。\n",
            "CXL.mem 允许  主机处理器将连接的 CXL 设备内存视为其内存地址空间的一部分。\n",
            "CXL.mem  允许主机处理器将连接的 CXL 设备内存视为其内存地址空间的一部分。\n",
        ] {
            assert_fix(input, correct);
        }
    }

    #[test]
    fn removes_gaps_between_chinese_punctuation_and_characters() {
        assert_fix("中  文 ， （ 内 容 ） 。\n", "中文，（内容）。\n");
    }

    #[test]
    fn collapses_runs_between_english_numbers_and_symbols() {
        assert_fix(
            "hello   world 中文  1  个 A  +  B\n",
            "hello world 中文 1 个 A + B\n",
        );
    }

    #[test]
    fn preserves_valid_specification_spacing() {
        let input = "初始时设备 1 的 **HMC（host memory cache）** 中没有该主机数据的副本（处于无效态（I）），而设备 2 缓存中存在该数据的独占态副本（E）。\n";
        assert_eq!(fix_markdown(input).content, input);
        assert!(check_markdown(input).is_empty());
    }

    #[test]
    fn preserves_ascii_punctuation_and_special_character_boundaries() {
        let input = "中 @ 文 # 字 $ 符 % 号 ^ 文 & 字 * 文 ( 字 ) 文 _ 字 + 文 - 字 = 文 ~ 字 / 文 \\ 字 | 文 [ 字 ] 文 { 字 } 文 , 字 . 文 ! 字 ? 文 : 字 ; 文\n";
        assert!(check_markdown(input).is_empty());
        assert_eq!(fix_markdown(input).content, input);
    }

    #[test]
    fn handles_extended_han_without_touching_other_scripts() {
        assert_fix("𠀀  文 龘 〇\n", "𠀀文龘〇\n");
        let input = "α β 한 글 あ い 😀 文\n";
        assert!(check_markdown(input).is_empty());
    }

    #[test]
    fn preserves_inline_code_with_different_delimiter_lengths() {
        assert_fix(
            "中 文 `中  文` 和 ``中 `  文``  后 文\n",
            "中文 `中  文` 和 ``中 `  文`` 后文\n",
        );
    }

    #[test]
    fn preserves_multiline_inline_code_and_math() {
        assert_fix(
            "中 文 `中  文\n中  文` 后 文 $a  +\nb  = c$ 正 文\n",
            "中文 `中  文\n中  文` 后文 $a  +\nb  = c$ 正文\n",
        );
    }

    #[test]
    fn preserves_inline_math_and_latex_delimiters() {
        assert_fix(
            "正 文 $中  文 + x$ 和 \\(a  + b\\)  后 文\n",
            "正文 $中  文 + x$ 和 \\(a  + b\\) 后文\n",
        );
    }

    #[test]
    fn preserves_display_math_contents() {
        let input = "中 文\n\n$$\n中  文 + a  b\n$$\n\n后 文\n\n$$中  文 + x$$\n";
        let expected = "中文\n\n$$\n中  文 + a  b\n$$\n\n后文\n\n$$中  文 + x$$\n";
        assert_fix(input, expected);
    }

    #[test]
    fn preserves_latex_display_math_and_unclosed_blocks() {
        assert_fix(
            "正 文\n\\[\n中  文 + a  b\n\\]\n后 文\n",
            "正文\n\\[\n中  文 + a  b\n\\]\n后文\n",
        );
        for input in ["$$\n中  文\n", "```python\n中  文\n"] {
            assert_eq!(fix_markdown(input).content, input);
        }
    }

    #[test]
    fn preserves_fenced_and_indented_code() {
        for input in [
            "~~~python\n中  文 = 'a  b'\n~~~\n",
            "````markdown\n中  文\n```python\nx  =  1\n```\n````\n",
            "    中  文 = 'a  b'\n",
            "> ```python\n> 中  文 = 'a  b'\n> ```\n",
            "- ```python\n    中  文 = 'a  b'\n    ```\n",
            "    ```python\n中  文 = 'a  b'\n    ```\n",
        ] {
            let fixed = fix_markdown(input);
            assert!(!fixed
                .issues
                .iter()
                .any(|issue| issue.message.contains("spaces in prose")
                    || issue.message.contains("Chinese")));
            assert!(fixed.content.contains("中  文"));
        }
    }

    #[test]
    fn preserves_structure_while_fixing_list_quote_and_table_prose() {
        assert_fix("-   中 文\n    - 子  项\n", "-   中文\n    - 子项\n");
        assert_fix(
            "- 中 文\n\n    > 引 用  text\n\n- 末 项\n",
            "- 中文\n\n    > 引用 text\n\n- 末项\n",
        );
        assert_fix(
            "| 中 文 | A  B |\n| --- | --- |\n",
            "| 中文 | A B |\n| --- | --- |\n",
        );
        assert_fix("正 文  \n下 一 行\n", "正文  \n下一行\n");
    }

    #[test]
    fn retains_original_line_numbers_after_other_rules_insert_blanks() {
        let fixed = fix_markdown("Intro:\n- 中 文\n\n后  文\n");
        let lines: Vec<_> = fixed
            .issues
            .iter()
            .filter(|issue| issue.message.contains("Chinese"))
            .map(|issue| issue.line)
            .collect();
        assert_eq!(lines, vec![2, 4]);
        assert_eq!(check_markdown(&fixed.content), vec![]);
    }

    #[test]
    fn escaped_and_unmatched_delimiters_do_not_hide_prose() {
        assert_fix(
            "\\` 中 文 \\$ 正 文 unmatched ` 后 文\n",
            "\\` 中文 \\$ 正文 unmatched ` 后文\n",
        );
        assert_fix("中 文 $5 and  more\n", "中文 $5 and more\n");
    }

    #[test]
    fn handles_crlf_empty_input_and_missing_final_newline() {
        assert_eq!(fix_markdown("").content, "");
        assert_fix("中  文\r\n", "中文\n");
        assert_fix("中  文", "中文");
    }

    #[test]
    fn protects_new_spans_after_a_multiline_span_closes() {
        assert_fix(
            "正 文 `a  b\nc  d` $中  文$ 后 文\n",
            "正文 `a  b\nc  d` $中  文$ 后文\n",
        );
        assert_fix(
            "正 文 $a  b\nc  d$ `中  文` 后 文\n",
            "正文 $a  b\nc  d$ `中  文` 后文\n",
        );
    }

    #[test]
    fn currency_amounts_do_not_hide_repeated_prose_spaces() {
        assert_fix("价格 $5  与 $10  不 同\n", "价格 $5 与 $10 不同\n");
    }

    #[test]
    fn unmatched_inline_delimiters_do_not_swallow_later_blocks() {
        assert_fix(
            "未 闭 `code\n\n```text\n中  文 `a`\n```\n\n后 文\n",
            "未闭 `code\n\n```text\n中  文 `a`\n```\n\n后文\n",
        );
        assert_fix(
            "未 闭 `code\n   \n后 文 ` tail\n",
            "未闭 `code\n   \n后文 ` tail\n",
        );
    }
}
