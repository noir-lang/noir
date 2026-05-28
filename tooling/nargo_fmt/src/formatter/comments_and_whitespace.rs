use noirc_frontend::{
    parser::block_comment_has_all_leading_stars,
    token::{DocStyle, Token},
};

use super::Formatter;

/// Whether a comment line must stay on its own line rather than being merged into a
/// reflowed prose paragraph with its neighbours. The `content` is the text of the line
/// after its comment prefix (`//`, `///`, `//!`, or a block comment's ` * `).
///
/// This keeps markdown structure intact when wrapping comments: blank lines, headings,
/// list items, code fences and table rows are never merged.
pub(crate) fn comment_line_is_standalone(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
        || trimmed.starts_with('|')
        || comment_line_is_list_item(trimmed)
        || trimmed == "noir-fmt:ignore"
}

/// Like [`comment_line_is_standalone`], but for line comments (`//`, `///`, `//!`) where a
/// run of four or more spaces (beyond the single conventional leading space) or a leading
/// tab marks an indented markdown code block that must not be reflowed.
pub(crate) fn line_comment_is_standalone(content: &str) -> bool {
    let body = content.strip_prefix(' ').unwrap_or(content);
    if body.starts_with("    ") || body.starts_with('\t') {
        return true;
    }

    comment_line_is_standalone(content)
}

/// Whether a trimmed line begins with a markdown list marker (`- `, `* `, `+ `, `1. `, `1) `).
fn comment_line_is_list_item(trimmed: &str) -> bool {
    if let Some(rest) = trimmed.strip_prefix(['-', '*', '+']) {
        return rest.starts_with(' ');
    }

    let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0
        && let Some(rest) = trimmed[digits..].strip_prefix(['.', ')'])
    {
        return rest.starts_with(' ');
    }

    false
}

/// Whether a code fence (` ``` ` or `~~~`) opens or closes on this trimmed line.
fn comment_line_is_code_fence(trimmed: &str) -> bool {
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// The classification content of a block-comment source line: the text after a leading
/// `*` (for `all_stars` comments) or after leading whitespace.
fn block_comment_line_content(line: &str, all_stars: bool) -> &str {
    if all_stars {
        let trimmed = line.trim_start();
        trimmed.strip_prefix('*').unwrap_or(trimmed)
    } else {
        line.trim_start()
    }
}

/// The text of a block-comment source line to append when merging it into a reflowed
/// paragraph: its content with leading whitespace and any leading `*` removed.
fn block_comment_line_text(line: &str, all_stars: bool) -> &str {
    block_comment_line_content(line, all_stars).trim_start()
}

#[cfg(windows)]
const NEWLINE: &str = "\r\n";
#[cfg(not(windows))]
const NEWLINE: &str = "\n";

impl Formatter<'_> {
    /// Writes a single space, skipping any whitespace and comments.
    /// That is, suppose the next token is a big whitespace, possibly with multiple lines.
    /// Those are skipped but only one space is written. In this way if we have
    /// "mod     foo" it's transformed to "mod foo".
    /// If there are comments in between `mod` and `foo` they are written, though!
    /// No comment is ever lost.
    ///
    /// A space is not appended to the buffer is it already ends with a space.
    pub(crate) fn write_space(&mut self) {
        self.skip_comments_and_whitespace();
        self.write_space_without_skipping_whitespace_and_comments();
    }

    /// Writes a single space, but doesn't skip whitespace and comments before doing that.
    ///
    /// A space is not appended to the buffer is it already ends with a space.
    pub(crate) fn write_space_without_skipping_whitespace_and_comments(&mut self) {
        if !self.buffer.ends_with_newline() && !self.buffer.ends_with_space() {
            self.write(" ");
        }
    }

    pub(crate) fn skip_whitespace(&mut self) {
        while let Token::Whitespace(..) = &self.token {
            self.bump();
        }
    }

    /// Only skips whitespace if it doesn't have newlines in it.
    /// Note that this doesn't write whitespace or comments at all.
    pub(crate) fn skip_whitespace_if_it_is_not_a_newline(&mut self) {
        while let Token::Whitespace(whitespace) = &self.token {
            if whitespace.contains('\n') {
                break;
            }
            self.bump();
        }
    }

    /// Skips comments and whitespace, writing newlines if there are any.
    /// If there are multiple consecutive newlines, only one is written.
    pub(crate) fn skip_comments_and_whitespace(&mut self) {
        self.skip_comments_and_whitespace_impl(
            false, // write multiple lines
            false, // at beginning
        );
    }

    /// Similar to skip_comments_and_whitespace, but will write two lines if
    /// multiple newlines are found (but at most two lines at a time).
    pub(crate) fn skip_comments_and_whitespace_writing_multiple_lines_if_found(&mut self) {
        self.skip_comments_and_whitespace_impl(
            true,  // write multiple lines
            false, // at beginning
        );
    }

    pub(crate) fn skip_comments_and_whitespace_impl(
        &mut self,
        write_multiple_lines: bool,
        at_beginning: bool,
    ) {
        // Number of newlines we just skipped.
        let mut number_of_newlines = 0;

        // Did we just passed some whitespace?
        let mut passed_whitespace = false;

        // Was the last token we processed a block comment?
        let mut last_was_block_comment = false;

        let mut ignore_next = self.ignore_next;

        loop {
            match &self.token {
                Token::Whitespace(whitespace) => {
                    number_of_newlines = whitespace.chars().filter(|char| *char == '\n').count();
                    passed_whitespace = whitespace.ends_with(' ');

                    if last_was_block_comment && number_of_newlines > 0 {
                        if number_of_newlines > 1 {
                            self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        } else {
                            self.write_line_without_skipping_whitespace_and_comments();
                        }

                        self.bump();

                        // Only indent for what's coming next if it's a comment
                        // (otherwise a closing brace must come and we wouldn't want to indent that)
                        if matches!(
                            &self.token,
                            Token::LineComment(_, None) | Token::BlockComment(_, None),
                        ) {
                            self.write_indentation();
                        }

                        number_of_newlines = 0;
                        passed_whitespace = false;
                    } else {
                        self.bump();
                    }

                    last_was_block_comment = false;
                }
                Token::LineComment(comment, None) => {
                    let comment = comment.clone();

                    if comment.trim() == "noir-fmt:ignore" {
                        ignore_next = true;
                        self.ignore_next = true;
                    }

                    // Here we check if we need to write one line, two lines or none after the
                    // end of the line comment.
                    if number_of_newlines > 1 && write_multiple_lines {
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if !(at_beginning && self.buffer.is_empty()) {
                        // We write a space before a line comment so if you have code like this:
                        // "1// comment" it's transformed to "1 // comment".
                        // What if there was already a space? It's all good, `write_space`
                        // will never write two consecutive spaces.
                        self.write_space_without_skipping_whitespace_and_comments();
                    }

                    if self.config.wrap_comments && !self.ignore_next && !self.in_chunk {
                        // Gather the whole run of consecutive line comments so prose can be
                        // reflowed as a paragraph rather than wrapped one source line at a
                        // time. The run consumes its own trailing newline (and comments), so
                        // the surrounding bookkeeping is restored from its result.
                        number_of_newlines = self.write_line_comment_run("//", None).unwrap_or(1);
                        if self.ignore_next {
                            ignore_next = true;
                        }
                    } else {
                        self.write_line_comment(&comment, "//");
                        self.write_line_without_skipping_whitespace_and_comments();
                        number_of_newlines = 1;
                        self.bump();
                        self.written_comments_count += 1;
                    }
                    passed_whitespace = false;
                    last_was_block_comment = false;
                }
                Token::BlockComment(comment, None) => {
                    let comment = comment.clone();

                    if comment.trim() == "noir-fmt:ignore" {
                        ignore_next = true;
                        self.ignore_next = true;
                    }

                    // Here we check if we need to write one line, two lines or none after the
                    // end of the block comment.
                    if number_of_newlines > 1 && write_multiple_lines {
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if passed_whitespace {
                        // We write a space before a line comment so if you have code like this:
                        // "1/* comment */" it's transformed to "1 /* comment */".
                        // What if there was already a space? It's all good, `write_space`
                        // will never write two consecutive spaces.
                        self.write_space_without_skipping_whitespace_and_comments();
                    }
                    self.write_block_comment(&comment, "/*");
                    self.bump();
                    passed_whitespace = false;
                    last_was_block_comment = true;
                    self.written_comments_count += 1;
                }
                _ => break,
            }
        }

        // Case when we passed some whitespace with newlines but no comments followed it.
        if number_of_newlines > 1 && write_multiple_lines {
            self.write_multiple_lines_without_skipping_whitespace_and_comments();
        }

        self.ignore_next = ignore_next;
    }

    pub(crate) fn write_line_comment(&mut self, comment: &str, prefix: &str) {
        // We don't wrap lines that start with '#' because these might be
        // markdown headers and wrapping those would actually break them.
        if self.ignore_next
            || !self.config.wrap_comments
            || self.in_chunk
            || comment.trim_start().starts_with('#')
            || self.current_line_width() + comment.chars().count() + prefix.len()
                < self.config.comment_width
        {
            self.write(prefix);
            self.write(comment.trim_end());
            return;
        }

        self.write_comment_with_prefix(comment, prefix);
    }

    pub(crate) fn write_comment_with_prefix(&mut self, comment: &str, prefix: &str) {
        self.write(prefix);
        for word in comment.split_inclusive([' ', '\n', '\t']) {
            if self.current_line_width() + word.trim().chars().count() > self.config.comment_width {
                self.start_new_line();
                if !prefix.is_empty() {
                    self.write(prefix);
                    self.write(" ");
                }
            }

            self.write(word);
        }
    }

    /// Gathers a maximal run of consecutive line comments of the given kind (`doc_style`
    /// distinguishes `//`, `///` and `//!`), separated only by single newlines, and writes
    /// it with paragraph reflow: prose lines that overflow `comment_width` are merged and
    /// re-wrapped together so an overflowing word is never stranded on its own line, while
    /// blank lines and markdown structure (headings, list items, code fences, ...) keep
    /// their own lines.
    ///
    /// `self.token` must be the first comment of the run, with the first line's indentation
    /// already written. A trailing newline is written after the run. Returns `Some(1)` when
    /// a single trailing newline was consumed and `self.token` now holds the token after the
    /// run; returns `None` when the run ended at a blank line or `EOF` left in `self.token`
    /// for the caller to reprocess.
    pub(crate) fn write_line_comment_run(
        &mut self,
        prefix: &str,
        doc_style: Option<DocStyle>,
    ) -> Option<usize> {
        let mut paragraph: Vec<String> = Vec::new();
        let mut wrote_segment = false;
        let mut in_code_fence = false;

        let result = loop {
            let Token::LineComment(content, _) = &self.token else {
                unreachable!("write_line_comment_run called on a non-line-comment token")
            };
            let content = content.clone();
            self.written_comments_count += 1;

            if content.trim() == "noir-fmt:ignore" {
                self.ignore_next = true;
            }

            let is_fence = comment_line_is_code_fence(content.trim_start());
            let standalone =
                in_code_fence || self.ignore_next || line_comment_is_standalone(&content);

            if standalone {
                self.flush_line_comment_paragraph(&mut paragraph, prefix, &mut wrote_segment);
                if wrote_segment {
                    self.start_new_line();
                }
                self.write(prefix);
                self.write(content.trim_end());
                wrote_segment = true;
                if is_fence {
                    in_code_fence = !in_code_fence;
                }
            } else {
                paragraph.push(content);
            }

            // Advance past this comment and decide whether the run continues.
            self.bump();
            match &self.token {
                Token::Whitespace(whitespace)
                    if whitespace.chars().filter(|c| *c == '\n').count() == 1 =>
                {
                    // Consume the single newline separating two comments and peek the next.
                    self.bump();
                    if matches!(&self.token, Token::LineComment(_, style) if *style == doc_style) {
                        continue;
                    }
                    break Some(1);
                }
                // A blank line (or `EOF`) ends the run; leave the token for the caller.
                _ => break None,
            }
        };

        self.flush_line_comment_paragraph(&mut paragraph, prefix, &mut wrote_segment);
        self.write_line_without_skipping_whitespace_and_comments();

        result
    }

    /// Writes the accumulated prose `paragraph` (if any) as a reflowed segment, preceded by
    /// a newline when an earlier segment was already written. Clears `paragraph`.
    fn flush_line_comment_paragraph(
        &mut self,
        paragraph: &mut Vec<String>,
        prefix: &str,
        wrote_segment: &mut bool,
    ) {
        if paragraph.is_empty() {
            return;
        }

        if *wrote_segment {
            self.start_new_line();
        }

        let lines = std::mem::take(paragraph);
        self.write_line_comment_paragraph(&lines, prefix);
        *wrote_segment = true;
    }

    /// Writes a single prose paragraph. If every line already fits within `comment_width`
    /// the original line breaks are preserved; otherwise the lines are joined and greedily
    /// re-wrapped, which is what pulls an overflowing trailing word down onto the next line.
    fn write_line_comment_paragraph(&mut self, lines: &[String], prefix: &str) {
        let base = self.current_line_width();
        let prefix_width = prefix.chars().count();
        let overflows = lines.iter().any(|line| {
            base + prefix_width + line.trim_end().chars().count() > self.config.comment_width
        });

        if overflows {
            let joined = lines.iter().map(|line| line.trim()).collect::<Vec<_>>().join(" ");
            self.write_comment_with_prefix(&format!(" {joined}"), prefix);
        } else {
            for (index, line) in lines.iter().enumerate() {
                if index > 0 {
                    self.start_new_line();
                }
                self.write(prefix);
                self.write(line.trim_end());
            }
        }
    }

    pub(crate) fn write_block_comment(&mut self, comment: &str, prefix: &str) {
        self.write(prefix);

        if self.ignore_next || !self.config.wrap_comments || self.in_chunk {
            self.write(comment);
            self.write("*/");
            return;
        }

        let all_stars = block_comment_has_all_leading_stars(comment);

        if comment.trim_start_matches([' ', '\t']).starts_with('\n') {
            self.start_new_line_no_indentation();
        }

        let lines: Vec<&str> = comment.lines().collect();
        let merge_with_previous = self.block_comment_merge_flags(&lines, all_stars);

        for (index, source_line) in lines.iter().enumerate() {
            // Consecutive prose lines that overflow `comment_width` are reflowed together
            // as one paragraph: instead of breaking onto a new source line, join the line's
            // text (stripped of its leading `*`/indentation) to the running word stream.
            let line: &str = if merge_with_previous[index] {
                if !self.buffer.ends_with_space() {
                    self.write(" ");
                }
                block_comment_line_text(source_line, all_stars)
            } else {
                // When moving to the next source line, only emit a newline. The line itself
                // carries the leading whitespace from the source, so re-emitting the
                // structural indent here would double it up.
                if index > 0 {
                    self.start_new_line_no_indentation();
                }
                source_line
            };

            for word in line.split_inclusive([' ', '\n', '\t']) {
                if self.current_line_width() + word.trim().chars().count()
                    > self.config.comment_width
                {
                    // Wrapping introduces a new line that has no source whitespace, so
                    // we re-emit the structural indent and the canonical `*` prefix.
                    self.start_new_line();
                    if all_stars {
                        self.write(" * ");
                    }
                }

                self.write(word);
            }
        }

        if comment.ends_with('\n') {
            self.start_new_line();
        }

        if self.current_line_width() + 2 > self.config.comment_width {
            self.start_new_line();
        }

        self.write("*/");
    }

    /// For each source line of a block comment, whether it should be merged onto the
    /// previous line when reflowing. A run of consecutive non-standalone (prose) lines is
    /// merged into one paragraph only when at least one of its lines overflows
    /// `comment_width`; otherwise the existing line breaks are preserved verbatim.
    fn block_comment_merge_flags(&self, lines: &[&str], all_stars: bool) -> Vec<bool> {
        let mut standalone = vec![false; lines.len()];
        let mut in_code_fence = false;
        for (index, line) in lines.iter().enumerate() {
            let content = block_comment_line_content(line, all_stars);
            standalone[index] = in_code_fence || comment_line_is_standalone(content);
            if comment_line_is_code_fence(content.trim_start()) {
                in_code_fence = !in_code_fence;
            }
        }

        let mut merge_with_previous = vec![false; lines.len()];
        let mut index = 0;
        while index < lines.len() {
            if standalone[index] {
                index += 1;
                continue;
            }

            let mut end = index + 1;
            while end < lines.len() && !standalone[end] {
                end += 1;
            }

            if lines[index..end]
                .iter()
                .any(|line| self.block_comment_line_overflows(line, all_stars))
            {
                for flag in &mut merge_with_previous[index + 1..end] {
                    *flag = true;
                }
            }

            index = end;
        }

        merge_with_previous
    }

    /// Whether a single block-comment source line, rendered at the comment's indentation,
    /// would exceed `comment_width`.
    fn block_comment_line_overflows(&self, line: &str, all_stars: bool) -> bool {
        let indentation = (self.indentation.max(0) as usize) * self.config.tab_spaces;
        let content_width = block_comment_line_text(line, all_stars).chars().count();
        // `all_stars` lines render with a leading `* ` prefix.
        let star_width = if all_stars { " * ".len() } else { 0 };
        indentation + star_width + content_width > self.config.comment_width
    }

    /// Returns the number of newlines that come next, if we are at a whitespace
    /// token (otherwise returns 0).
    pub(crate) fn following_newlines_count(&self) -> usize {
        let Token::Whitespace(whitespace) = &self.token else {
            return 0;
        };

        whitespace.chars().filter(|char| *char == '\n').count()
    }

    /// Writes a single newline, if the last thing we wrote wasn't also a newline
    /// (this prevents multiple consecutive newlines, though that's still possible to
    /// do if you call `write_multiple_lines_...`).
    ///
    /// Any whitespace or comments found right at and after the current token are "skipped"
    /// (whitespace is discarded, comments are written).
    pub(crate) fn write_line(&mut self) {
        self.skip_comments_and_whitespace_impl(
            true,  // writing newline
            false, // at beginning
        );
        self.write_line_without_skipping_whitespace_and_comments();
    }

    pub(crate) fn write_line_without_skipping_whitespace_and_comments(&mut self) -> bool {
        if !self.buffer.ends_with_newline() && !self.buffer.ends_with_space() {
            self.write(NEWLINE);
            true
        } else {
            false
        }
    }

    // Modifies the current buffer so that it will always have two newlines at the end.
    pub(crate) fn write_multiple_lines_without_skipping_whitespace_and_comments(&mut self) {
        if self.buffer.ends_with_double_newline() {
            // Nothing
        } else if self.buffer.ends_with_newline() {
            self.write(NEWLINE);
        } else {
            self.write(NEWLINE);
            self.write(NEWLINE);
        }
    }

    pub(crate) fn start_new_line(&mut self) {
        self.start_new_line_no_indentation();
        self.write_indentation();
    }

    pub(crate) fn start_new_line_no_indentation(&mut self) {
        self.trim_spaces();
        self.write_line_without_skipping_whitespace_and_comments();
    }

    /// Trim spaces from the end of the buffer.
    pub(crate) fn trim_spaces(&mut self) {
        self.buffer.trim_spaces();
    }

    /// Trim commas from the end of the buffer. Returns true if a comma was trimmed.
    pub(crate) fn trim_comma(&mut self) -> bool {
        self.buffer.trim_comma()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Config, assert_format, assert_format_with_config, assert_format_with_max_width,
        assert_formatter_changes_with_config,
    };
    use test_case::test_case;

    fn assert_format_wrapping_comments(src: &str, expected: &str, comment_width: usize) {
        let config = Config { wrap_comments: true, comment_width, ..Config::default() };
        assert_format_with_config(src, expected, config);
    }

    fn assert_formatter_changes_wrapping_comments(src: &str, comment_width: usize) {
        let config = Config { wrap_comments: true, comment_width, ..Config::default() };
        assert_formatter_changes_with_config(src, config);
    }

    #[test]
    fn format_array_in_global_with_line_comments() {
        let src = "global x = [ // hello
        1 , 2 ] ;";
        let expected = "global x = [
    // hello
    1, 2,
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_2() {
        let src = "global x = [ // hello
         [ 1 , 2 ]  ] ;";
        let expected = "global x = [
    // hello
    [1, 2],
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_3() {
        let src = "global x =
    [ 
        // hello
        [1, 2],  
    ];
";
        let expected = "global x = [
    // hello
    [1, 2],
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_4() {
        let src = "global x =
    [
        1, // world 
        2, 3,
    ];
";
        let expected = "global x = [
    1, // world
    2, 3,
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_block_comments() {
        let src = "global x = [ /* hello */
        1 , 2 ] ;";
        let expected = "global x = [
    /* hello */ 1,
    2,
];
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_if_with_comment_after_condition() {
        let src = "global x = if  123  // some comment  
        {   456   }  ;";
        let expected = "global x = if 123 // some comment
{
    456
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_with_comment_after_else() {
        let src = "global x = if  123  {   456   } else  // some comment 
        { 789 };";
        let expected = "global x = if 123 {
    456
} else // some comment
{
    789
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_when_some_args_are_multiline_because_of_line_comments() {
        let src = "fn  foo ( a: i32, // comment
         b: i32
         )  { }  ";
        let expected = "fn foo(
    a: i32, // comment
    b: i32,
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_when_some_args_are_multiline_because_of_line_comments_2() {
        let src = "fn  foo ( a: i32, // comment
        // another
         b: i32 // another comment
         )  { }  ";
        let expected = "fn foo(
    a: i32, // comment
    // another
    b: i32, // another comment
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_when_some_args_are_multiline_because_of_block_comments() {
        let src = "fn  foo ( a: i32 /*
        some
        comment */, b: i32
         )  { }  ";
        let expected = "fn foo(
    a: i32 /*
        some
        comment */,
    b: i32,
) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_comment_after_parameters() {
        let src = "fn main()
        // hello 
    {}";
        let expected = "fn main()
// hello
{}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_line_comment_in_parameters() {
        let src = "fn main(
        // hello
        )
    {}";
        let expected = "fn main(
    // hello
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_line_comment_on_top_of_parameter() {
        let src = "fn main(
// hello
unit: ()
) {}";
        let expected = "fn main(
    // hello
    unit: (),
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_block_comment_in_params() {
        let src = "fn main(/* test */) {}";
        let expected = "fn main(/* test */) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_body_and_block_comment() {
        let src = "fn main() { 
        /* foo */ 
        1 }";
        let expected = "fn main() {
    /* foo */
    1
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_body_one_expr_trailing_comment() {
        let src = "mod moo { fn main() { 1   // yes
        } }";
        let expected = "mod moo {
    fn main() {
        1 // yes
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_body_one_expr_semicolon_trailing_comment() {
        let src = "mod moo { fn main() { 1  ; // yes
        } }";
        let expected = "mod moo {
    fn main() {
        1; // yes
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_many_exprs_trailing_comments() {
        let src = "mod moo { fn main() { 1  ; // yes
        2 ; // no
        3 // maybe
        } }";
        let expected = "mod moo {
    fn main() {
        1; // yes
        2; // no
        3 // maybe
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_block_comment_after_two_newlines() {
        let src = "fn foo() {
    1;

    /* world */
    2
}
";
        let expected = "fn foo() {
    1;

    /* world */
    2
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_on_top_of_let_followed_by_statement() {
        let src = "fn foo() {
    1;

    // Comment
    let x = 1;
}
";
        let expected = "fn foo() {
    1;

    // Comment
    let x = 1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_block_comments() {
        let src = "  mod/*a*/ foo /*b*/ ; ";
        let expected = "mod/*a*/ foo /*b*/;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_inline_comments() {
        let src = "  mod // a  
 foo // b 
  ; ";
        let expected = "mod // a
foo // b
;
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_line_comments_in_separate_line() {
        let src = " #[foo] pub  mod foo { 
// one
#[hello]
mod bar; 
// two
}";
        let expected = "#[foo]
pub mod foo {
    // one
    #[hello]
    mod bar;
    // two
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_line_comment_in_same_line() {
        let src = " #[foo] pub  mod foo {  // one
mod bar; 
}";
        let expected = "#[foo]
pub mod foo { // one
    mod bar;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_block_comment() {
        let src = " #[foo] pub  mod foo {  /* one */
/* two */
mod bar; 
}";
        let expected = "#[foo]
pub mod foo { /* one */
    /* two */
    mod bar;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_block_comment_2() {
        let src = "mod foo {
        /* one */
}";
        let expected = "mod foo {
    /* one */
}
";
        assert_format(src, expected);
    }

    #[test]
    fn keeps_spaces_between_comments() {
        let src = "  mod  foo { 

// hello

// world

} ";
        let expected = "mod foo {

    // hello

    // world

}
";
        assert_format(src, expected);
    }

    #[test]
    fn comment_with_leading_space() {
        let src = "    // comment
        // hello
mod  foo ; ";
        let expected = "// comment
// hello
mod foo;
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_block_statement_with_inline_block_comment() {
        let src = " fn foo() { { /* hello */ } } ";
        let expected = "fn foo() {
    { /* hello */ }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_struct_with_block_comments() {
        let src = " struct Foo {
        /* hello */
    }
        ";
        let expected = "struct Foo {
    /* hello */
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_with_just_comments() {
        let src = " mod foo { struct Foo {
// hello
    } }
        ";
        let expected = "mod foo {
    struct Foo {
        // hello
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_no_whitespace_in_block_single_line() {
        let src = "global x = {/*foo*/};";
        let expected = "global x = { /*foo*/ };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_no_whitespace_but_newline_in_block_single_line() {
        let src = "global x = {/*foo*/
        };";
        let expected = "global x = { /*foo*/ };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_in_block_same_line() {
        let src = "global x = {       // foo
        };";
        let expected = "global x = { // foo
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_in_block_separate_line() {
        let src = "global x = {
        // foo
        };";
        let expected = "global x = {
    // foo
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_in_parenthesized_expression() {
        let src = "global x = ( /* foo */ 1 );";
        let expected = "global x = ( /* foo */ 1);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_in_parenthesized() {
        let src = "global x = ( // hello 
        1 );";
        let expected = "global x = (
    // hello
    1
);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_index_with_comment() {
        let src = "global x = foo[// hello
        1];";
        let expected = "global x = foo[
    // hello
    1
];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_infix_between_lhs_and_operator() {
        let src = "global x = 1/* comment */+ 2 ;";
        let expected = "global x = 1 /* comment */ + 2;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_constructor_inside_function() {
        let src = "fn foo() { MyStruct {/*test*/}; } ";
        let expected = "fn foo() {
    MyStruct { /*test*/ };
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_before_constructor_field() {
        let src = "global x = Foo {/*comment*/field}; ";
        let expected = "global x = Foo { /*comment*/ field };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_before_constructor_field() {
        let src = "global x = Foo { // foo
        field}; ";
        let expected = "global x = Foo {
    // foo
    field,
};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_empty_constructor() {
        let src = "global x = Foo { // comment
        }; ";
        let expected = "global x = Foo { // comment
};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_after_parenthesized() {
        let src = "global x = (
            1
            // hello
        )
        ; ";
        let expected = "global x = (
    1
    // hello
);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_single_element_tuple() {
        let src = "global x = ( 1 /* hello */ , );";
        let expected = "global x = (1 /* hello */,);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_after_impl_function() {
        let src = "impl Foo { fn foo() {} 
        // bar 
        }";
        let expected = "impl Foo {
    fn foo() {}
    // bar
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_after_trait_impl_function() {
        let src = "impl Foo for Bar { fn foo() {} 
        // bar 
        }";
        let expected = "impl Foo for Bar {
    fn foo() {}
    // bar
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_after_trait_function() {
        let src = "trait Foo { fn foo() {} 
        // bar 
        }";
        let expected = "trait Foo {
    fn foo() {}
    // bar
}
";
        assert_format(src, expected);
    }

    #[test]
    fn keeps_newlines_after_comment_at_the_beginning() {
        let src = "// foo

global x = 1;
";
        let expected = src;
        assert_format(src, expected);
    }

    #[test]
    fn keeps_newlines_after_comment_at_the_beginning_2() {
        let src = "
        
        // foo

global x = 1;
";
        let expected = "// foo

global x = 1;
";
        assert_format(src, expected);
    }

    #[test]
    fn wraps_line_comments() {
        let src = "
        // This is a long comment that's going to be wrapped.
        global x: Field = 1;
        ";
        let expected = "// This is a long comment
// that's going to be
// wrapped.
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_line_comments_with_indentation() {
        let src = "
        mod moo {
            // This is a long comment that's going to be wrapped.
            global x: Field = 1;
        }
        ";
        let expected = "mod moo {
    // This is a long comment
    // that's going to be
    // wrapped.
    global x: Field = 1;
}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn does_not_wrap_line_comment_if_it_starts_with_pound() {
        let src = "
        // # This is a long comment that's not going to be wrapped.
        global x: Field = 1;
        ";
        let expected = "// # This is a long comment that's not going to be wrapped.
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_line_comments_in_statement() {
        let src = "fn foo() {
        // This is a long comment that's going to be wrapped.
        let x = 1;
    }
        ";
        let expected = "fn foo() {
    // This is a long comment
    // that's going to be
    // wrapped.
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_line_comments_in_statement_trailing_position() {
        let src = "fn foo() {
        let x = 1; // This is a long comment that's going to be wrapped.
    }
        ";
        let expected = "fn foo() {
    let x = 1; // This is a
    // long comment that's
    // going to be wrapped.
}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_line_comments_in_argument_trailing_position() {
        let src = "fn foo() {
        bar(
        1, // This is a long comment that's going to be wrapped.
    )}
        ";
        let expected = "fn foo() {
    bar(
        1, // This is a long
        // comment that's
        // going to be
        // wrapped.
    )
}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_comments_without_newlines_1() {
        let src = "
/* This is a long comment that's going to be wrapped. */
global x: Field = 1;
        ";
        let expected = "/* This is a long comment
 * that's going to be
 * wrapped. */
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_comments_without_newlines_2() {
        let src = "
/* This is a long comment that's wrapped. */
global x: Field = 1;
        ";
        let expected = "/* This is a long comment
 * that's wrapped. */
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_comments_without_newlines_3() {
        let src = "
/* This is a long comment that will be wrapped. */
global x: Field = 1;
        ";
        let expected = "/* This is a long comment
 * that will be wrapped. */
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_comments_with_newlines() {
        let src = "
/*  
This is a long comment that's going to be wrapped.
*/
global x: Field = 1;
        ";
        let expected = "/*
This is a long comment that's
going to be wrapped.
*/
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_comments_multiple_lines() {
        let src = "
/*  
This is a long comment that's wrapped.
This is a long comment that's wrapped.
*/
global x: Field = 1;
        ";
        let expected = "/*
This is a long comment that's
wrapped. This is a long
comment that's wrapped.
*/
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_comments_multiple_lines_with_all_stars() {
        let src = "
/*  
 * This is a long comment that's wrapped.
 * This is a long comment that's wrapped.
 */
global x: Field = 1;
        ";
        let expected = "/*
 * This is a long comment
 * that's wrapped. This is a
 * long comment that's
 * wrapped.
 */
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_comments_in_statement() {
        let src = "fn foo() {
        /* This is a long comment that's going to be wrapped. */
        let x = 1;
    }
        ";
        let expected = "fn foo() {
    /* This is a long comment
    that's going to be
    wrapped. */
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_mixed_comments_in_statement() {
        let src = "fn foo() {
        /* 
        This is a long comment that's going to be wrapped. 
        This is a long comment that's going to be wrapped. 
        */
        // This is a long comment that's going to be wrapped.
        /* This is a long comment that's going to be wrapped. */
        let x = 1;
    }
        ";
        let expected = "fn foo() {
    /* This is a long comment
    that's going to be
    wrapped.
    This is a long comment
    that's going to be
    wrapped.
    */
    // This is a long comment
    // that's going to be
    // wrapped.
    /* This is a long comment
    that's going to be
    wrapped. */
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_line_comments_at_block_end() {
        let src = "fn foo() {
        let x = 1;
        // This is a very long comment
    }
        ";
        let expected = "fn foo() {
    let x = 1;
    // This is a very long
    // comment
}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn does_not_wrap_line_comment_when_at_max() {
        let src = "// One two three
fn foo() {}
";
        assert_format_wrapping_comments(src, src, 16);
    }

    #[test]
    fn trims_newlines_from_the_end_of_the_file() {
        let src = "global x: Field = 1;\n\n\n";
        let expected = "global x: Field = 1;\n";
        assert_format(src, expected);
    }

    #[test]
    fn block_comment_trailing_spaces_bug() {
        // Note: there are three trailing spaces after "one"
        let src = "fn foo() {
    /*
    * one   
    * two
    */
}
";
        // Here the trailing spaces are gone
        let expected = "fn foo() {
    /*
    * one
    * two
    */
}
";
        assert_format(src, expected);
    }

    #[test_case("//", "" ; "line comment")]
    #[test_case("/*", " */" ; "block comment")]
    #[test_case("///", "" ; "outer doc line comment")]
    #[test_case("/**", " */" ; "outer doc block comment")]
    fn does_not_wrap_outer_comment_if_directed_to_ignore(prefix: &str, suffix: &str) {
        let comment = format!(
            r#"{prefix} This is a long comment that's going to be wrapped.{suffix}
{prefix} This is a long comment that's going to be wrapped.{suffix}
global x: Field = 1;
"#
        );
        assert_formatter_changes_wrapping_comments(&comment, 29);
        let ignored_comment = format!("// noir-fmt:ignore\n{comment}");
        assert_format_wrapping_comments(&ignored_comment, &ignored_comment, 29);
    }

    #[test_case("//!", "" ; "inner doc line comment")]
    #[test_case("/*!", " */" ; "inner doc block comment")]
    fn does_not_wrap_inner_comment_if_directed_to_ignore(prefix: &str, suffix: &str) {
        let comment = format!(
            r#"{prefix} This is a long comment that's going to be wrapped.{suffix}
{prefix} This is a long comment that's going to be wrapped.{suffix}
"#
        );
        assert_formatter_changes_wrapping_comments(&comment, 29);
        let ignored_comment = format!("// noir-fmt:ignore\n{comment}");
        assert_format_wrapping_comments(&ignored_comment, &ignored_comment, 29);
    }

    #[test]
    fn does_not_over_indent_block_comment_when_wrapping_is_enabled_all_stars() {
        let src = "pub struct Foo {}

impl Foo {
    /**
     * Build a Foo.
     * @return a fresh Foo
     */
    fn build() {}
}
";
        assert_format_wrapping_comments(src, src, 120);
    }

    #[test]
    fn does_not_over_indent_block_comment_when_wrapping_is_enabled_not_all_stars() {
        let src = "pub struct Foo {}

impl Foo {
    /**
       Build a Foo.
       @return a fresh Foo
     */
    fn build() {}
}
";
        assert_format_wrapping_comments(src, src, 120);
    }

    #[test]
    fn preserves_utf8_in_line_comment() {
        // cSpell:disable-next-line
        let src = "// schön — héllo 🙂
fn main() {}
";
        assert_format(src, src);
    }

    #[test]
    fn preserves_utf8_in_block_comment() {
        let src = "/* 日本語 in a block comment */
fn main() {}
";
        assert_format(src, src);
    }

    #[test]
    fn preserves_utf8_in_doc_comment() {
        let src = "/// 日本語 doc on a function
fn main() {}
";
        assert_format(src, src);
    }

    #[test]
    fn preserves_utf8_in_trailing_line_comment() {
        // cSpell:disable-next-line
        let src = "fn main() {
    let x = 1; // héllo 🙂
}
";
        assert_format(src, src);
    }

    #[test]
    fn reflows_line_comment_paragraph_without_stranding_a_word() {
        let src = "// This comment is sized so it sits just barely over one hundred and twenty columns when the trailing word and word2 here
// continues onto a second line that is shorter than the limit
fn main() {}
";
        let expected = "// This comment is sized so it sits just barely over one hundred and twenty columns when the trailing word and word2
// here continues onto a second line that is shorter than the limit
fn main() {}
";
        assert_format_wrapping_comments(src, expected, 120);
    }

    #[test]
    fn reflows_each_comment_paragraph_independently_around_a_blank_comment_line() {
        let src = "// This is the first paragraph that is long
// and continues here
//
// This is the second paragraph that is long
// and continues here
fn main() {}
";
        let expected = "// This is the first
// paragraph that is long and
// continues here
//
// This is the second
// paragraph that is long and
// continues here
fn main() {}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn does_not_merge_markdown_list_items_when_wrapping() {
        let src = "// Here is a list of things:
// - the first item in the list
// - the second item in the list
fn main() {}
";
        let expected = "// Here is a list of things:
// - the first item in the list
// - the second item in the list
fn main() {}
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn does_not_reflow_a_code_fence_when_wrapping() {
        let src = "/// Example of using the function below:
/// ```
/// let result = the_function(a, b);
/// let other = result + another_value;
/// ```
fn main() {}
";
        let expected = "/// Example of using the
/// function below:
/// ```
/// let result = the_function(a, b);
/// let other = result + another_value;
/// ```
fn main() {}
";
        assert_format_wrapping_comments(src, expected, 29);
    }
}
