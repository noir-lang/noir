//! Paragraph-aware reflow for comments.
//!
//! Given a sequence of comment content lines (with the per-line prefix such as
//! `//`, `///`, or ` * ` already stripped), this module merges adjacent "plain"
//! lines into paragraphs, then greedily wraps each paragraph to the configured
//! width. Certain line shapes (markdown headers, list items, blockquotes,
//! fenced code, tables, URL lines, Javadoc-style `@tag` lines, blank lines)
//! are recognized and never merged into prose.
//!
//! Output lines do not carry the prefix or any caller-side indentation; the
//! caller is responsible for adding them. The engine is a pure function with
//! no formatter state.

/// Wrap a sequence of comment content lines into paragraph-aware reflowed output.
///
/// `lines` is the comment content split by physical source line, with each line's
/// per-line prefix already stripped. Leading whitespace within each line is
/// preserved so that markdown structure can be detected.
///
/// `first_budget` is the maximum characters allowed for content on the first
/// output line. `cont_budget` is the maximum characters for subsequent lines.
/// Budgets exclude the per-line prefix the caller will add.
///
/// Returns one String per output line. The caller adds the prefix and any
/// indent to each line.
pub(crate) fn reflow_comment(
    lines: &[&str],
    first_budget: usize,
    cont_budget: usize,
) -> Vec<String> {
    let blocks = parse_blocks(lines);
    emit_blocks(&blocks, first_budget, cont_budget)
}

#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum Block {
    Paragraph { words: Vec<String> },
    ListItem { marker: String, hanging_indent: usize, words: Vec<String> },
    BlockQuote { words: Vec<String> },
    Passthrough { raw_lines: Vec<String> },
}

#[derive(Debug, PartialEq, Eq)]
enum LineKind<'a> {
    Blank,
    Header,
    JavadocTag,
    BulletItem { marker_len: usize },
    OrderedItem { marker_len: usize },
    BlockQuote,
    UrlLine,
    TableLine,
    FenceToggle,
    Plain { indent: usize, content: &'a str },
}

fn classify<'a>(line: &'a str) -> LineKind<'a> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return LineKind::Blank;
    }
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return LineKind::FenceToggle;
    }
    if trimmed.starts_with('#') {
        return LineKind::Header;
    }
    if is_javadoc_tag(trimmed) {
        return LineKind::JavadocTag;
    }
    if let Some(len) = bullet_marker_len(trimmed) {
        return LineKind::BulletItem { marker_len: len };
    }
    if let Some(len) = ordered_marker_len(trimmed) {
        return LineKind::OrderedItem { marker_len: len };
    }
    if trimmed.starts_with("> ") || trimmed == ">" {
        return LineKind::BlockQuote;
    }
    if is_table_line(trimmed) {
        return LineKind::TableLine;
    }
    if contains_url_or_reference(line) {
        return LineKind::UrlLine;
    }
    let indent = line.len() - trimmed.len();
    LineKind::Plain { indent, content: trimmed }
}

fn is_javadoc_tag(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix('@') else {
        return false;
    };
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        || rest.split_once(|c: char| c.is_ascii_whitespace()).is_some_and(|(name, _)| {
            !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

fn bullet_marker_len(trimmed: &str) -> Option<usize> {
    for marker in ["* ", "- ", "+ "] {
        if trimmed.starts_with(marker) {
            return Some(marker.len());
        }
    }
    None
}

fn ordered_marker_len(trimmed: &str) -> Option<usize> {
    let mut chars = trimmed.char_indices();
    let mut digits = 0;
    let mut digit_end = 0;
    while let Some((idx, c)) = chars.clone().next() {
        if c.is_ascii_digit() {
            digits += 1;
            digit_end = idx + c.len_utf8();
            chars.next();
            if digits > 2 {
                return None;
            }
        } else {
            break;
        }
    }
    if digits == 0 {
        return None;
    }
    let rest = &trimmed[digit_end..];
    if rest.starts_with(". ") || rest.starts_with(") ") { Some(digit_end + 2) } else { None }
}

fn is_table_line(trimmed: &str) -> bool {
    if !trimmed.starts_with('|') {
        return false;
    }
    trimmed[1..].contains('|')
}

fn contains_url_or_reference(line: &str) -> bool {
    for scheme in ["https://", "http://", "ftp://", "file://"] {
        if line.contains(scheme) {
            return true;
        }
    }
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix('[')
        && let Some(close_idx) = rest.find(']')
    {
        let after = &rest[close_idx + 1..];
        if after.starts_with(':') {
            return true;
        }
    }
    false
}

fn parse_blocks(lines: &[&str]) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut in_fence = false;
    let mut fence_lines: Vec<String> = Vec::new();

    for &line in lines {
        if in_fence {
            fence_lines.push(line.to_string());
            if matches!(classify(line), LineKind::FenceToggle) {
                in_fence = false;
                blocks.push(Block::Passthrough { raw_lines: std::mem::take(&mut fence_lines) });
            }
            continue;
        }

        let kind = classify(line);
        match kind {
            LineKind::FenceToggle => {
                in_fence = true;
                fence_lines.push(line.to_string());
            }
            LineKind::Blank => {
                blocks.push(Block::Passthrough { raw_lines: vec![String::new()] });
            }
            LineKind::Header | LineKind::TableLine | LineKind::UrlLine => {
                blocks.push(Block::Passthrough { raw_lines: vec![line.to_string()] });
            }
            LineKind::JavadocTag => {
                let words = collect_words(line);
                blocks.push(Block::Paragraph { words });
            }
            LineKind::BulletItem { marker_len } | LineKind::OrderedItem { marker_len } => {
                let trimmed = line.trim_start();
                let marker = trimmed[..marker_len].to_string();
                let leading_indent = line.len() - trimmed.len();
                let rest = &trimmed[marker_len..];
                let words = collect_words(rest);
                let hanging = leading_indent + marker_len;
                blocks.push(Block::ListItem { marker, hanging_indent: hanging, words });
            }
            LineKind::BlockQuote => {
                let trimmed = line.trim_start();
                let content = trimmed
                    .strip_prefix("> ")
                    .or_else(|| trimmed.strip_prefix('>'))
                    .unwrap_or(trimmed);
                let words = collect_words(content);
                if let Some(Block::BlockQuote { words: existing }) = blocks.last_mut() {
                    existing.extend(words);
                } else {
                    blocks.push(Block::BlockQuote { words });
                }
            }
            LineKind::Plain { indent, content } => {
                let new_words = collect_words(content);
                let mut merged = false;
                if let Some(last) = blocks.last_mut() {
                    match last {
                        Block::ListItem { hanging_indent, words, .. } => {
                            if indent >= *hanging_indent {
                                words.extend(new_words.clone());
                                merged = true;
                            }
                        }
                        Block::Paragraph { words } => {
                            if indent == 0 {
                                words.extend(new_words.clone());
                                merged = true;
                            }
                        }
                        _ => {}
                    }
                }
                if !merged {
                    if indent == 0 {
                        blocks.push(Block::Paragraph { words: new_words });
                    } else {
                        blocks.push(Block::Passthrough { raw_lines: vec![line.to_string()] });
                    }
                }
            }
        }
    }

    if in_fence && !fence_lines.is_empty() {
        blocks.push(Block::Passthrough { raw_lines: fence_lines });
    }

    blocks
}

fn collect_words(content: &str) -> Vec<String> {
    content.split_whitespace().map(|s| s.to_string()).collect()
}

fn emit_blocks(blocks: &[Block], first_budget: usize, cont_budget: usize) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();

    for block in blocks {
        match block {
            Block::Passthrough { raw_lines } => {
                for line in raw_lines {
                    output.push(line.clone());
                }
            }
            Block::Paragraph { words } => {
                let fb = if output.is_empty() { first_budget } else { cont_budget };
                let wrapped = wrap_words(words, fb, cont_budget);
                output.extend(wrapped);
            }
            Block::ListItem { marker, hanging_indent, words } => {
                let fb = if output.is_empty() { first_budget } else { cont_budget };
                let first_word_budget = fb.saturating_sub(marker.chars().count());
                let cont_word_budget = cont_budget.saturating_sub(*hanging_indent);
                let wrapped = wrap_words(words, first_word_budget, cont_word_budget);
                let mut iter = wrapped.into_iter();
                if let Some(first) = iter.next() {
                    output.push(format!("{marker}{first}"));
                } else {
                    output.push(marker.trim_end().to_string());
                }
                let pad: String = std::iter::repeat_n(' ', *hanging_indent).collect();
                for cont in iter {
                    output.push(format!("{pad}{cont}"));
                }
            }
            Block::BlockQuote { words } => {
                let fb = if output.is_empty() { first_budget } else { cont_budget };
                let first_word_budget = fb.saturating_sub(2);
                let cont_word_budget = cont_budget.saturating_sub(2);
                let wrapped = wrap_words(words, first_word_budget, cont_word_budget);
                for line in wrapped {
                    output.push(format!("> {line}"));
                }
            }
        }
    }

    output.into_iter().map(trim_trailing_spaces).collect()
}

fn trim_trailing_spaces(mut s: String) -> String {
    let trimmed_len = s.trim_end_matches([' ', '\t']).len();
    s.truncate(trimmed_len);
    s
}

fn wrap_words(words: &[String], first_budget: usize, cont_budget: usize) -> Vec<String> {
    if words.is_empty() {
        return Vec::new();
    }
    let mut output: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut on_first_line = true;
    for word in words {
        let budget = if on_first_line { first_budget } else { cont_budget };
        if current.is_empty() {
            current.push_str(word);
        } else {
            let needed = current.chars().count() + 1 + word.chars().count();
            if needed <= budget {
                current.push(' ');
                current.push_str(word);
            } else {
                output.push(std::mem::take(&mut current));
                on_first_line = false;
                current.push_str(word);
            }
        }
    }
    if !current.is_empty() {
        output.push(current);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reflow(lines: &[&str], first: usize, cont: usize) -> Vec<String> {
        reflow_comment(lines, first, cont)
    }

    #[test]
    fn single_short_paragraph_unchanged() {
        let out = reflow(&["Hello world"], 80, 80);
        assert_eq!(out, vec!["Hello world".to_string()]);
    }

    #[test]
    fn merges_two_lines_then_rewraps() {
        let out = reflow(&["Hello world, I just realized that this", "is a long comment"], 32, 32);
        assert_eq!(
            out,
            vec![
                "Hello world, I just realized".to_string(),
                "that this is a long comment".to_string(),
            ]
        );
    }

    #[test]
    fn user_example_reflow() {
        let out = reflow(&["Hello world, I just realized that this", "is a long comment"], 22, 22);
        // Words flow naturally across the boundary.
        assert_eq!(
            out,
            vec![
                "Hello world, I just".to_string(),
                "realized that this is".to_string(),
                "a long comment".to_string(),
            ]
        );
    }

    #[test]
    fn blank_line_breaks_paragraph() {
        let out = reflow(&["foo bar", "", "baz qux"], 80, 80);
        assert_eq!(out, vec!["foo bar".to_string(), String::new(), "baz qux".to_string()]);
    }

    #[test]
    fn markdown_header_passthrough() {
        let out = reflow(&["# Title", "body text"], 5, 5);
        assert_eq!(out, vec!["# Title".to_string(), "body".to_string(), "text".to_string()]);
    }

    #[test]
    fn fenced_code_passthrough() {
        let out = reflow(&["```rust", "fn  main()  {}", "```", "after"], 80, 80);
        assert_eq!(
            out,
            vec![
                "```rust".to_string(),
                "fn  main()  {}".to_string(),
                "```".to_string(),
                "after".to_string(),
            ]
        );
    }

    #[test]
    fn url_line_passthrough() {
        let out = reflow(&["see https://example.com/some/very/long/path for more"], 10, 10);
        assert_eq!(out, vec!["see https://example.com/some/very/long/path for more".to_string()]);
    }

    #[test]
    fn bullet_item_with_hanging_indent() {
        let out = reflow(&["- first item that wraps to next line"], 20, 20);
        assert_eq!(out, vec!["- first item that".to_string(), "  wraps to next line".to_string(),]);
    }

    #[test]
    fn ordered_list_marker() {
        let out = reflow(&["1. first ordered item that wraps"], 18, 18);
        assert_eq!(out, vec!["1. first ordered".to_string(), "   item that wraps".to_string()]);
    }

    #[test]
    fn javadoc_tag_breaks_paragraph() {
        let out = reflow(&["Build a Foo.", "@return a fresh Foo"], 80, 80);
        assert_eq!(out, vec!["Build a Foo.".to_string(), "@return a fresh Foo".to_string()]);
    }

    #[test]
    fn blockquote_reflows_with_prefix() {
        let out = reflow(&["> a quoted line that should wrap"], 12, 12);
        assert_eq!(
            out,
            vec![
                "> a quoted".to_string(),
                "> line that".to_string(),
                "> should".to_string(),
                "> wrap".to_string(),
            ]
        );
    }

    #[test]
    fn word_longer_than_budget_emitted_alone() {
        let out = reflow(&["short superduperlongword end"], 10, 10);
        assert_eq!(
            out,
            vec!["short".to_string(), "superduperlongword".to_string(), "end".to_string(),]
        );
    }

    #[test]
    fn first_budget_can_differ_from_cont_budget() {
        let out = reflow(&["alpha beta gamma delta"], 10, 20);
        assert_eq!(out, vec!["alpha beta".to_string(), "gamma delta".to_string()]);
    }

    #[test]
    fn idempotent_on_wrapped_paragraph() {
        let once = reflow(&["Hello world, I just realized that this", "is a long comment"], 22, 22);
        let refs: Vec<&str> = once.iter().map(String::as_str).collect();
        let twice = reflow(&refs, 22, 22);
        assert_eq!(once, twice);
    }

    #[test]
    fn idempotent_on_list_item() {
        let once = reflow(&["- first item that wraps to next line"], 20, 20);
        let refs: Vec<&str> = once.iter().map(String::as_str).collect();
        let twice = reflow(&refs, 20, 20);
        assert_eq!(once, twice);
    }
}
