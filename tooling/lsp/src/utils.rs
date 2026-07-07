// These functions are copied from the codespan_lsp crate, except that they never panic
// (the library will sometimes panic, so functions returning Result are not always accurate)

use fm::codespan_files::Files;
use fm::{FileId, FileMap};

pub(crate) fn range_to_byte_span(
    files: &FileMap,
    file_id: FileId,
    range: &async_lsp::lsp_types::Range,
) -> Option<std::ops::Range<usize>> {
    Some(
        position_to_byte_index(files, file_id, &range.start)?
            ..position_to_byte_index(files, file_id, &range.end)?,
    )
}

pub(crate) fn position_to_byte_index(
    files: &FileMap,
    file_id: FileId,
    position: &async_lsp::lsp_types::Position,
) -> Option<usize> {
    let Ok(source) = files.source(file_id) else {
        return None;
    };

    let Ok(line_span) = files.line_range(file_id, position.line as usize) else {
        return None;
    };
    let line_str = source.get(line_span.clone())?;

    let byte_offset = character_to_line_offset(line_str, position.character)?;

    Some(line_span.start + byte_offset)
}

pub(crate) fn character_to_line_offset(line: &str, character: u32) -> Option<usize> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();

            return Some(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u32;
    }

    // Handle positions after the last character on the line
    if character_offset == character { Some(line_len) } else { None }
}

/// Given a string with a single ">|<" (cursor) in it, returns:
/// 1. The line where the cursor is (zero-based)
/// 2. The column where the cursor is (zero-based)
/// 3. that string with ">|<" removed
#[cfg(test)]
pub(crate) fn get_cursor_line_and_column(src: &str) -> (usize, usize, String) {
    let (line, column) = src
        .lines()
        .enumerate()
        .find_map(|(line_index, line)| line.find(">|<").map(|char_index| (line_index, char_index)))
        .expect("Expected to find one >|< in the source code");

    let src = src.replace(">|<", "");
    (line, column, src)
}
