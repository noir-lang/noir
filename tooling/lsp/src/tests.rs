#![cfg(test)]

use lsp_types::TextEdit;

pub(crate) fn apply_text_edit(src: &str, text_edit: &TextEdit) -> String {
    let mut lines: Vec<_> = src.lines().collect();
    assert_eq!(text_edit.range.start.line, text_edit.range.end.line);

    let mut line = lines[text_edit.range.start.line as usize].to_string();
    line.replace_range(
        text_edit.range.start.character as usize..text_edit.range.end.character as usize,
        &text_edit.new_text,
    );
    lines[text_edit.range.start.line as usize] = &line;
    lines.join("\n")
}
