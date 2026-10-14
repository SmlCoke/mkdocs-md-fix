use std::path::Path;

use crate::diff::colored_diff;
use crate::{fix_markdown, split_command_line};

#[test]
fn inserts_blank_before_top_level_list() {
    let input = "Intro:\n- item\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, "Intro:\n\n- item\n");
    assert!(fixed
        .issues
        .iter()
        .any(|issue| issue.message.contains("list block")));
}

#[test]
fn inserts_blank_after_top_level_list() {
    let input = "Intro:\n\n- item\nEnd\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, "Intro:\n\n- item\n\nEnd\n");
    assert!(fixed
        .issues
        .iter()
        .any(|issue| issue.message.contains("followed by a blank line")));
}

#[test]
fn does_not_split_nested_list_items() {
    let input = "- item\n    - child\n- next\nEnd\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, "- item\n    - child\n- next\n\nEnd\n");
}

#[test]
fn normalizes_nested_list_indent() {
    let input = "- item\n  - child\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, "- item\n    - child\n");
    assert!(fixed.issues.iter().any(
        |issue| issue.message.contains("Nested list") || issue.message.contains("nested list")
    ));
}

#[test]
fn fixes_blockquote_inside_list() {
    let input = "1. item\n   > quote\n2. item\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, "1. item\n\n    > quote\n\n2. item\n");
}

#[test]
fn surrounds_fenced_code_block() {
    let input = "Before:\n```python\nprint('hi')\n```\nAfter\n";
    let fixed = fix_markdown(input);

    assert_eq!(
        fixed.content,
        "Before:\n\n```python\nprint('hi')\n```\n\nAfter\n"
    );
}

#[test]
fn does_not_insert_blank_lines_inside_fenced_code_block() {
    let input = "```python\nprint(1)\n```\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, input);
    assert!(fixed.issues.is_empty());
}

#[test]
fn fixes_list_code_block_indent_and_spacing() {
    let input = "- item\n   ```python\nprint('hi')\n   ```\n- next\n";
    let fixed = fix_markdown(input);

    assert_eq!(
        fixed.content,
        "- item\n\n    ```python\n    print('hi')\n    ```\n\n- next\n"
    );
}

#[test]
fn surrounds_display_math() {
    let input = "Formula:\n$$E = mc^2$$\nDone\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, "Formula:\n\n$$E = mc^2$$\n\nDone\n");
}

#[test]
fn surrounds_table() {
    let input = "Data:\n| Name | Age |\n| --- | --- |\n| Alice | 30 |\nDone\n";
    let fixed = fix_markdown(input);

    assert_eq!(
        fixed.content,
        "Data:\n\n| Name | Age |\n| --- | --- |\n| Alice | 30 |\n\nDone\n"
    );
}

#[test]
fn ignores_markdown_inside_fenced_code_block() {
    let input = "````markdown\nText:\n- item\n````\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, input);
    assert!(fixed.issues.is_empty());
}

#[test]
fn ignores_nested_fences_inside_markdown_demo_block() {
    let input = "```markdown\nText:\n```python\nprint(1)\n```\n- not checked\n```\n";
    let fixed = fix_markdown(input);

    assert_eq!(fixed.content, input);
    assert!(fixed.issues.is_empty());
}

#[test]
fn colored_diff_prints_only_changed_lines() {
    let diff = colored_diff(
        Path::new("sample.md"),
        "- item\n  - child\n",
        "- item\n    - child\n",
    );

    assert!(diff.contains("\x1b[31m-  - child\x1b[0m"));
    assert!(diff.contains("\x1b[32m+    - child\x1b[0m"));
    assert!(!diff.contains("\n- item\n"));
}

#[test]
fn colored_diff_shows_inserted_blank_line() {
    let diff = colored_diff(
        Path::new("sample.md"),
        "Intro:\n- item\n",
        "Intro:\n\n- item\n",
    );

    assert!(diff.contains("@@ -2,0 +2,1 @@"));
    assert!(diff.contains("\x1b[32m+\x1b[0m"));
    assert!(!diff.contains("Intro:"));
}

#[test]
fn shell_parser_handles_quoted_paths() {
    let args = split_command_line("check \"docs/my file.md\" -r").unwrap();
    assert_eq!(args, vec!["check", "docs/my file.md", "-r"]);
}
