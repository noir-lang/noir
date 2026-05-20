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
/// `format_code` is invoked for each fenced code block.
/// It receives the joined fence body and the language tag (`None` for untagged fences).
/// Returning `Some(formatted)` replaces the inner lines of the fence; returning `None`
/// leaves the original content untouched.
///
/// When `reflow_paragraphs` is `false` the parser does not merge consecutive plain lines
/// into a single paragraph, list-item continuations don't fold into their parent, and
/// blockquotes don't span lines. Each input line is wrapped independently, preserving
/// the pre-reflow per-line wrap behavior.
pub(crate) fn reflow_comment_with_code_formatter<F>(
    lines: &[&str],
    first_budget: usize,
    cont_budget: usize,
    reflow_paragraphs: bool,
    format_code: F,
) -> Vec<String>
where
    F: Fn(&str, Option<&str>) -> Option<String>,
{
    let mut blocks = parse_blocks(lines, reflow_paragraphs);
    apply_code_formatter(&mut blocks, &format_code);
    emit_blocks(&blocks, first_budget, cont_budget)
}

fn apply_code_formatter<F>(blocks: &mut [Block], format_code: &F)
where
    F: Fn(&str, Option<&str>) -> Option<String>,
{
    for block in blocks {
        if let Block::CodeFence { lang, inner_lines, .. } = block {
            let lang_opt = if lang.is_empty() { None } else { Some(lang.as_str()) };
            let source = inner_lines.join("\n");
            if let Some(formatted) = format_code(&source, lang_opt) {
                *inner_lines = formatted
                    .strip_suffix('\n')
                    .unwrap_or(&formatted)
                    .lines()
                    .map(String::from)
                    .collect();
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum Block {
    Paragraph {
        words: Vec<String>,
    },
    ListItem {
        marker: String,
        hanging_indent: usize,
        words: Vec<String>,
    },
    BlockQuote {
        words: Vec<String>,
    },
    Passthrough {
        raw_lines: Vec<String>,
    },
    /// A fenced code block. `lang` is the info string after the opening fence (empty for
    /// untagged fences). `fence_open` / `fence_close` are the raw delimiter lines as
    /// they appeared in the source so we can re-emit them verbatim. `inner_lines` is the
    /// fence body, which may be replaced by a code-formatter callback before emit.
    CodeFence {
        lang: String,
        fence_open: String,
        fence_close: String,
        inner_lines: Vec<String>,
    },
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

fn parse_blocks(lines: &[&str], reflow_paragraphs: bool) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut in_fence = false;
    let mut fence_open: String = String::new();
    let mut fence_lang: String = String::new();
    let mut fence_lines: Vec<String> = Vec::new();

    for &line in lines {
        if in_fence {
            if matches!(classify(line), LineKind::FenceToggle) {
                in_fence = false;
                blocks.push(Block::CodeFence {
                    lang: std::mem::take(&mut fence_lang),
                    fence_open: std::mem::take(&mut fence_open),
                    fence_close: line.to_string(),
                    inner_lines: std::mem::take(&mut fence_lines),
                });
            } else {
                fence_lines.push(line.to_string());
            }
            continue;
        }

        let kind = classify(line);
        match kind {
            LineKind::FenceToggle => {
                in_fence = true;
                fence_open = line.to_string();
                fence_lang = extract_fence_lang(line);
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
                if reflow_paragraphs
                    && let Some(Block::BlockQuote { words: existing }) = blocks.last_mut()
                {
                    existing.extend(words);
                } else {
                    blocks.push(Block::BlockQuote { words });
                }
            }
            LineKind::Plain { indent, content } => {
                let new_words = collect_words(content);
                let mut merged = false;
                if reflow_paragraphs && let Some(last) = blocks.last_mut() {
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

    if in_fence {
        let mut raw_lines = Vec::new();
        if !fence_open.is_empty() {
            raw_lines.push(fence_open);
        }
        raw_lines.extend(fence_lines);
        if !raw_lines.is_empty() {
            blocks.push(Block::Passthrough { raw_lines });
        }
    }

    blocks
}

/// Returns the language tag of a fence opener (the info string after the leading ` ``` `
/// or `~~~`). Empty string for untagged fences.
fn extract_fence_lang(line: &str) -> String {
    let trimmed = line.trim_start();
    let marker = if trimmed.starts_with("```") { "```" } else { "~~~" };
    let after = trimmed.strip_prefix(marker).unwrap_or("");
    let mut after = after;
    while let Some(rest) = after.strip_prefix('`').or_else(|| after.strip_prefix('~')) {
        after = rest;
    }
    after.trim().to_string()
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
                let first_word_budget = fb.saturating_sub(*hanging_indent);
                let cont_word_budget = cont_budget.saturating_sub(*hanging_indent);
                let wrapped = wrap_words(words, first_word_budget, cont_word_budget);
                let marker_len = marker.chars().count();
                let leading_indent = hanging_indent.saturating_sub(marker_len);
                let leading_pad: String = " ".repeat(leading_indent);
                let mut iter = wrapped.into_iter();
                if let Some(first) = iter.next() {
                    output.push(format!("{leading_pad}{marker}{first}"));
                } else {
                    output.push(format!("{leading_pad}{}", marker.trim_end()));
                }
                let cont_pad: String = " ".repeat(*hanging_indent);
                for cont in iter {
                    output.push(format!("{cont_pad}{cont}"));
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
            Block::CodeFence { fence_open, fence_close, inner_lines, .. } => {
                output.push(fence_open.clone());
                for line in inner_lines {
                    output.push(line.clone());
                }
                output.push(fence_close.clone());
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
        reflow_comment_with_code_formatter(lines, first, cont, true, |_, _| None)
    }

    fn reflow_no_merge(lines: &[&str], first: usize, cont: usize) -> Vec<String> {
        reflow_comment_with_code_formatter(lines, first, cont, false, |_, _| None)
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
    fn nested_list_item_preserves_leading_indent() {
        let out = reflow(&["- Parent", "  - Child"], 80, 80);
        assert_eq!(out, vec!["- Parent".to_string(), "  - Child".to_string()]);
    }

    #[test]
    fn nested_list_item_with_wrap() {
        let out = reflow(&["- Parent", "  - Child item that is long enough to wrap"], 20, 20);
        assert_eq!(
            out,
            vec![
                "- Parent".to_string(),
                "  - Child item that".to_string(),
                "    is long enough".to_string(),
                "    to wrap".to_string(),
            ]
        );
    }

    #[test]
    fn reflow_false_does_not_merge_consecutive_plain_lines() {
        let out = reflow_no_merge(&["foo bar", "baz qux"], 80, 80);
        assert_eq!(out, vec!["foo bar".to_string(), "baz qux".to_string()]);
    }

    #[test]
    fn reflow_false_still_wraps_individual_long_lines() {
        let out = reflow_no_merge(&["one two three four five six seven"], 12, 12);
        assert_eq!(
            out,
            vec![
                "one two".to_string(),
                "three four".to_string(),
                "five six".to_string(),
                "seven".to_string()
            ]
        );
    }

    #[test]
    fn reflow_false_does_not_merge_list_item_continuations() {
        let out = reflow_no_merge(&["- first item", "  continuation text"], 80, 80);
        // The continuation does not fold into the list item; it becomes its own block.
        assert_eq!(out, vec!["- first item".to_string(), "  continuation text".to_string()]);
    }

    #[test]
    fn idempotent_nested_list() {
        let once = reflow(&["- Parent", "  - Child item that is long enough to wrap"], 20, 20);
        let refs: Vec<&str> = once.iter().map(String::as_str).collect();
        let twice = reflow(&refs, 20, 20);
        assert_eq!(once, twice);
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
