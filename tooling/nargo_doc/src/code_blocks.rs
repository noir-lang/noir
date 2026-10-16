//! Recognizing fenced code blocks in doc comments.

/// Returns true if `line` opens a code block that contains Noir code.
///
/// The language is the first word of the info string after the opening backticks, where words
/// are separated by spaces or commas, so an info string such as `noir ignore` or
/// `noir,should_fail` is Noir code with extra attributes.
/// Code blocks without a language are assumed to be Noir. Rust code blocks are treated as Noir
/// too, because the two languages are very similar and some stdlib docs use Rust code blocks.
pub fn is_noir_code_block_start(line: &str) -> bool {
    let Some(info) = line.strip_prefix("```") else {
        return false;
    };
    let language = info.split([' ', '\t', ',']).next().unwrap_or_default();
    matches!(language, "" | "noir" | "rust")
}
