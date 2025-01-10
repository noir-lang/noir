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

pub(crate) fn apply_text_edits(src: &str, text_edits: &[TextEdit]) -> String {
    let mut text = src.to_string();

    // Text edits must be applied from last to first, otherwise if we apply a text edit
    // that comes before another one, that other one becomes invalid (it will edit the wrong
    // piece of code).
    let mut text_edits = text_edits.to_vec();
    text_edits.sort_by_key(|edit| (edit.range.start.line, edit.range.start.character));
    text_edits.reverse();

    for text_edit in text_edits {
        text = apply_text_edit(&text, &text_edit);
    }

    text
}
