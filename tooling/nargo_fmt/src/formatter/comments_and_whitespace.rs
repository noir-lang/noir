use noirc_frontend::{parser::block_comment_has_all_leading_stars, token::Token};

use super::Formatter;
use super::comment_reflow;
use crate::Config;

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

    /// Similar to `skip_comments_and_whitespace`, but will write two lines if
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

                    // If this comment is trailing-inline (no newline came before it and we
                    // aren't at the start of the file), keep it isolated. Subsequent
                    // standalone `//` comments on the following lines belong to the next
                    // "block" of comments, not to this one — they would arrive in a
                    // separate `skip_comments_and_whitespace_impl` call in normal flow,
                    // but here they'd get swallowed by the group loop. The buffer check
                    // alone isn't enough because chunk construction runs us against a
                    // fresh sub-buffer, so we rely on `number_of_newlines` instead.
                    let first_is_trailing_inline = number_of_newlines == 0 && !at_beginning;

                    let mut group = vec![comment];
                    self.bump();
                    let mut group_count = 1;

                    // Extend the group with consecutive `//` line comments at the same
                    // indentation, so that reflow can treat them as one paragraph. We
                    // never group across blank lines, ignore directives, doc-style
                    // changes (`///` and `//!` arrive as separate token variants), or
                    // when the leading comment is attached to code on its own line.
                    if self.config.wrap_comments && !self.ignore_next && !first_is_trailing_inline {
                        while let Token::Whitespace(ws) = &self.token {
                            let newlines = ws.chars().filter(|c| *c == '\n').count();
                            if newlines != 1 {
                                break;
                            }
                            let extends = match self.peek_next_token() {
                                Token::LineComment(next_body, None) => {
                                    next_body.trim() != "noir-fmt:ignore"
                                }
                                _ => false,
                            };
                            if !extends {
                                break;
                            }
                            self.bump();
                            let Token::LineComment(next_body, None) = self.bump() else {
                                unreachable!("peek confirmed a plain line comment")
                            };
                            group.push(next_body);
                            group_count += 1;
                        }
                    }

                    self.write_line_comment_group(&group, "//");
                    self.write_line_without_skipping_whitespace_and_comments();
                    number_of_newlines = 1;
                    passed_whitespace = false;
                    last_was_block_comment = false;
                    self.written_comments_count += group_count;
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

    /// Reflow a sequence of consecutive line-comment bodies as a single paragraph-aware
    /// group. The bodies arrive with the per-line `//` prefix already stripped (i.e. they
    /// are the lexer's comment body strings). The caller is responsible for having written
    /// any leading indentation for the first comment.
    ///
    /// While inside a chunk's sub-buffer we don't yet know the rendered position of the
    /// emitted text, so we don't reflow here — we just emit each body verbatim. The
    /// chunks layer (`write_chunk_lines`) does the actual reflow once we're back in the
    /// main buffer and the budget is known.
    pub(crate) fn write_line_comment_group(&mut self, bodies: &[String], prefix: &str) {
        if self.ignore_next || !self.config.wrap_comments || self.in_chunk {
            for (index, body) in bodies.iter().enumerate() {
                if index > 0 {
                    self.start_new_line();
                }
                self.write(prefix);
                self.write(body.trim_end());
            }
            return;
        }

        let reflow_enabled = reflow_enabled_for_prefix(self.config, prefix);

        // With reflow disabled, take the pre-reflow per-line wrap path: each body wraps
        // independently and internal whitespace (e.g. multiple spaces inside the
        // comment) is preserved. The paragraph-aware engine normalizes runs of
        // whitespace via `split_whitespace`, which we don't want here.
        if !reflow_enabled {
            for (index, body) in bodies.iter().enumerate() {
                if index > 0 {
                    self.start_new_line();
                }
                self.write_comment_with_prefix(body.trim_end(), prefix);
            }
            return;
        }

        let stripped: Vec<String> = bodies
            .iter()
            .map(|body| {
                let trimmed = body.trim_end();
                trimmed.strip_prefix(' ').unwrap_or(trimmed).to_string()
            })
            .collect();
        let lines: Vec<&str> = stripped.iter().map(String::as_str).collect();

        let prefix_cols = prefix.chars().count() + 1;
        let indent_cols = (self.indentation.max(0) as usize) * self.config.tab_spaces;
        let comment_width = self.config.comment_width;
        let first_budget =
            comment_width.saturating_sub(self.current_line_width()).saturating_sub(prefix_cols);
        let cont_budget = comment_width.saturating_sub(indent_cols).saturating_sub(prefix_cols);

        let config = self.config;
        let snippet_width =
            self.config.max_width.saturating_sub(indent_cols).saturating_sub(prefix_cols);
        let outputs = comment_reflow::reflow_comment_with_code_formatter(
            &lines,
            first_budget,
            cont_budget,
            reflow_enabled,
            |source, lang| format_noir_snippet(source, lang, config, snippet_width),
        );

        for (index, content) in outputs.iter().enumerate() {
            if index > 0 {
                self.start_new_line();
            }
            self.write(prefix);
            if !content.is_empty() {
                self.write(" ");
                self.write(content);
            }
        }
    }

    /// Wraps a single comment body word-by-word with the given prefix. Kept for the
    /// in-chunk path which performs line-by-line wrapping over already-grouped chunk
    /// strings; the standalone-comment path uses `write_line_comment_group` instead.
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

    /// Pre-reflow body emit for `write_block_comment`. Each source line wraps
    /// independently with the same word-by-word logic as `write_comment_with_prefix`,
    /// preserving internal whitespace. The opening `/*`-style prefix has already been
    /// written by the caller; we write the body and the closing `*/`.
    fn write_block_comment_non_reflow(&mut self, comment: &str, all_stars: bool) {
        if comment.trim_start_matches([' ', '\t']).starts_with('\n') {
            self.start_new_line_no_indentation();
        }

        for (index, line) in comment.lines().enumerate() {
            if index > 0 {
                self.start_new_line_no_indentation();
            }

            for word in line.split_inclusive([' ', '\n', '\t']) {
                if self.current_line_width() + word.trim().chars().count()
                    > self.config.comment_width
                {
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

    pub(crate) fn write_block_comment(&mut self, comment: &str, prefix: &str) {
        self.write(prefix);

        if self.ignore_next || !self.config.wrap_comments || self.in_chunk {
            self.write(comment);
            self.write("*/");
            return;
        }

        let all_stars = block_comment_has_all_leading_stars(comment);
        let indent_cols = (self.indentation.max(0) as usize) * self.config.tab_spaces;
        let comment_width = self.config.comment_width;

        let reflow_enabled = reflow_enabled_for_prefix(self.config, prefix);

        // With reflow disabled, take the pre-reflow per-line wrap path: each source
        // line wraps independently and internal whitespace within each line is
        // preserved (the paragraph-aware engine normalizes via `split_whitespace`).
        if !reflow_enabled {
            self.write_block_comment_non_reflow(comment, all_stars);
            return;
        }

        // A block comment has two emit shapes. We pick based on whether the source
        // body opens with a newline; that decision is idempotent because the wrapped
        // form preserves the opening shape.
        //
        // Single-line shape (no leading newline):
        //     /* first wrap
        //      * second wrap */
        //
        // Multi-line shape (leading newline):
        //     /*
        //     content
        //     */
        let body_opens_with_newline = comment.trim_start_matches([' ', '\t']).starts_with('\n');

        let source_lines: Vec<&str> = comment.lines().collect();
        let mut content_lines: Vec<String> = source_lines
            .iter()
            .map(|line| {
                let after_ws = line.trim_start();
                if all_stars {
                    if let Some(rest) = after_ws.strip_prefix("* ") {
                        rest.to_string()
                    } else if let Some(rest) = after_ws.strip_prefix('*') {
                        rest.strip_prefix(' ').unwrap_or(rest).to_string()
                    } else {
                        after_ws.to_string()
                    }
                } else {
                    after_ws.to_string()
                }
            })
            .collect();

        while content_lines.first().is_some_and(|s| s.trim().is_empty()) {
            content_lines.remove(0);
        }
        while content_lines.last().is_some_and(|s| s.trim().is_empty()) {
            content_lines.pop();
        }

        if content_lines.is_empty() {
            self.write(comment);
            self.write("*/");
            return;
        }

        let line_refs: Vec<&str> = content_lines.iter().map(String::as_str).collect();

        if !body_opens_with_newline {
            let has_leading_space = comment.starts_with(' ') || comment.starts_with('\t');
            let has_trailing_space = comment.ends_with(' ') || comment.ends_with('\t');

            let first_used = self.current_line_width() + if has_leading_space { 1 } else { 0 };
            let cont_used = indent_cols + 3;
            let first_budget = comment_width.saturating_sub(first_used);
            let cont_budget = comment_width.saturating_sub(cont_used);

            let config = self.config;
            let snippet_width = self.config.max_width.saturating_sub(cont_used);
            let outputs = comment_reflow::reflow_comment_with_code_formatter(
                &line_refs,
                first_budget,
                cont_budget,
                reflow_enabled,
                |source, lang| format_noir_snippet(source, lang, config, snippet_width),
            );

            if has_leading_space {
                self.write(" ");
            }
            if let Some((first, rest)) = outputs.split_first() {
                self.write(first);
                for line in rest {
                    self.start_new_line();
                    if all_stars {
                        self.write(" * ");
                    }
                    self.write(line);
                }
            }
            if has_trailing_space {
                self.write(" ");
            }
            self.write("*/");
            return;
        }

        let prefix_used = if all_stars { 3 } else { 0 };
        let content_budget = comment_width.saturating_sub(indent_cols + prefix_used);
        let config = self.config;
        let snippet_width = self.config.max_width.saturating_sub(indent_cols + prefix_used);
        let outputs = comment_reflow::reflow_comment_with_code_formatter(
            &line_refs,
            content_budget,
            content_budget,
            reflow_enabled,
            |source, lang| format_noir_snippet(source, lang, config, snippet_width),
        );

        self.start_new_line_no_indentation();
        for content in &outputs {
            if content.is_empty() && !all_stars {
                self.write(NEWLINE);
                continue;
            }
            self.write_indentation();
            if all_stars {
                if content.is_empty() {
                    self.write(" *");
                } else {
                    self.write(" * ");
                    self.write(content);
                }
            } else {
                self.write(content);
            }
            self.start_new_line_no_indentation();
        }
        self.write_indentation();
        if all_stars {
            self.write(" ");
        }
        self.write("*/");
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

/// Returns whether paragraph reflow is enabled for the comment kind identified by its
/// per-line prefix. Plain `//` and `/*` opt in via `reflow_non_doc_comments`; doc-style
/// `///`, `//!`, `/**`, and `/*!` opt in via `reflow_doc_comments`.
fn reflow_enabled_for_prefix(config: &Config, prefix: &str) -> bool {
    match prefix {
        "//" | "/*" => config.reflow_non_doc_comments,
        _ => config.reflow_doc_comments,
    }
}

/// Recursively formats a Noir code snippet pulled from a doc-comment fenced block.
/// Returns `None` when the snippet should be left alone — either the fence has a
/// non-Noir language tag, the body is empty, or the snippet fails to parse cleanly.
///
/// `available_width` is how many columns the rendered fence body actually gets — the
/// parent's `max_width` minus the comment-line prefix (e.g. `/// `) and any outer
/// indentation. The inner formatter's `max_width` and related single-line thresholds
/// are tightened to this so output that lands inside the prefix still respects the
/// configured maximum.
fn format_noir_snippet(
    source: &str,
    lang: Option<&str>,
    config: &Config,
    available_width: usize,
) -> Option<String> {
    if !config.format_code_blocks {
        return None;
    }
    let lang_ok = match lang {
        None => true,
        Some(l) => l.eq_ignore_ascii_case("noir"),
    };
    if !lang_ok || source.trim().is_empty() {
        return None;
    }
    let (parsed, errors) = noirc_frontend::parse_program_with_dummy_file(source);
    if errors.into_iter().any(|e| !e.is_warning()) {
        return None;
    }

    // Floor at 20 so deeply-indented or very tight comment positions still produce
    // legible output rather than degenerate one-token-per-line wrapping.
    let inner_width = available_width.max(20);
    let mut snippet_config = config.clone();
    snippet_config.max_width = inner_width;
    snippet_config.fn_call_width = config.fn_call_width.min(inner_width);
    snippet_config.array_width = config.array_width.min(inner_width);
    snippet_config.single_line_if_else_max_width =
        config.single_line_if_else_max_width.min(inner_width);
    snippet_config.comment_width = config.comment_width.min(inner_width);

    Some(crate::format(source, parsed, &snippet_config))
}

#[cfg(test)]
mod tests {
    use crate::{
        Config, assert_format, assert_format_with_config, assert_format_with_max_width,
        assert_formatter_changes_with_config,
    };
    use test_case::test_case;

    fn normalize_expected_newlines(expected: &str) -> String {
        if cfg!(windows) { expected.replace('\n', "\r\n") } else { expected.to_owned() }
    }

    fn assert_format_wrapping_comments(src: &str, expected: &str, comment_width: usize) {
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: true,
            reflow_non_doc_comments: true,
            format_code_blocks: true,
            comment_width,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

    fn assert_formatter_changes_wrapping_comments(src: &str, comment_width: usize) {
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: true,
            reflow_non_doc_comments: true,
            format_code_blocks: true,
            comment_width,
            ..Config::default()
        };
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
    fn format_lvalue_index_with_comment() {
        let src = "fn foo(mut bar: [Field; 2]) {
    bar[// hello
    1] = 2;
}";
        let expected = "fn foo(mut bar: [Field; 2]) {
    bar[
        // hello
        1
    ] = 2;
}
";
        assert_format(src, &normalize_expected_newlines(expected));
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
    fn tight_block_comment_preserves_no_spaces() {
        let src = "fn foo(/*static=*/ x: Field) {}\n";
        assert_format_wrapping_comments(src, src, 80);
    }

    #[test]
    fn block_comment_with_only_leading_space_preserved() {
        let src = "fn foo(/* leading_only*/ x: Field) {}\n";
        assert_format_wrapping_comments(src, src, 80);
    }

    #[test]
    fn block_comment_with_only_trailing_space_preserved() {
        let src = "fn foo(/*trailing_only */ x: Field) {}\n";
        assert_format_wrapping_comments(src, src, 80);
    }

    #[test]
    fn nested_list_in_outer_doc_comment() {
        let src = "/// - Parent
///   - Child
fn foo() {}
";
        assert_format_wrapping_comments(src, src, 80);
    }

    #[test]
    fn reflow_respects_blank_line_paragraph_break() {
        let src = "fn foo() {
    // First paragraph that is long enough to wrap.
    //
    // Second paragraph that also goes on for quite a while.
    let x = 1;
}
";
        let expected = "fn foo() {
    // First paragraph that is
    // long enough to wrap.
    //
    // Second paragraph that
    // also goes on for quite
    // a while.
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 30);
    }

    #[test]
    fn reflow_passes_through_markdown_header() {
        let src = "fn foo() {
    // # Title that should not wrap regardless of length whatsoever
    // body text body text body text body text body text
    let x = 1;
}
";
        let expected = "fn foo() {
    // # Title that should not wrap regardless of length whatsoever
    // body text body text
    // body text body text
    // body text
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 30);
    }

    #[test]
    fn reflow_list_item_with_hanging_indent() {
        let src = "fn foo() {
    // - first item that is long enough to wrap to the next line
    let x = 1;
}
";
        let expected = "fn foo() {
    // - first item that is
    //   long enough to wrap
    //   to the next line
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 30);
    }

    #[test]
    fn reflow_comments_off_preserves_per_line_wrap() {
        // Two consecutive `//` lines at narrow width: with reflow off they must not
        // merge into a paragraph. Each line wraps independently.
        let src = "fn foo() {
    // Hello world, I just realized that this
    // is a long comment
    let x = 1;
}
";
        let expected = "fn foo() {
    // Hello world, I just realized
    // that this
    // is a long comment
    let x = 1;
}
";
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: false,
            reflow_non_doc_comments: false,
            comment_width: 35,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn reflow_comments_off_preserves_internal_whitespace_in_line_comment() {
        // Multiple consecutive spaces inside a `//` comment stay verbatim when reflow
        // is off. Under reflow=true the engine would collapse them via
        // `split_whitespace`.
        let src = "// foo  bar    baz
fn main() {}
";
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: false,
            reflow_non_doc_comments: false,
            comment_width: 80,
            ..Config::default()
        };
        assert_format_with_config(src, src, config);
    }

    #[test]
    fn reflow_comments_off_preserves_internal_whitespace_in_block_comment() {
        let src = "/* foo  bar    baz */
fn main() {}
";
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: false,
            reflow_non_doc_comments: false,
            comment_width: 80,
            ..Config::default()
        };
        assert_format_with_config(src, src, config);
    }

    #[test]
    fn reflow_doc_comments_only_does_not_merge_plain_comments() {
        // With `reflow_doc_comments=true` and `reflow_non_doc_comments=false`,
        // doc-style comments merge but plain `//` comments preserve their line breaks.
        let src = "/// Hello world, I just realized that this
/// is a long doc comment
fn one() {}

// Hello world, I just realized that this
// is a long plain comment
fn two() {}
";
        let expected = "/// Hello world, I just realized
/// that this is a long doc comment
fn one() {}

// Hello world, I just realized
// that this
// is a long plain comment
fn two() {}
";
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: true,
            reflow_non_doc_comments: false,
            comment_width: 35,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn reflow_non_doc_comments_only_does_not_merge_doc_comments() {
        // Mirror image: only plain `//` comments merge.
        let src = "/// Hello world, I just realized that this
/// is a long doc comment
fn one() {}

// Hello world, I just realized that this
// is a long plain comment
fn two() {}
";
        let expected = "/// Hello world, I just realized
/// that this
/// is a long doc comment
fn one() {}

// Hello world, I just realized
// that this is a long plain
// comment
fn two() {}
";
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: false,
            reflow_non_doc_comments: true,
            comment_width: 35,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn format_code_blocks_off_leaves_fence_untouched() {
        let src = "/// ```
/// fn   foo()   {  1  }
/// ```
fn bar() {}
";
        let config = Config {
            wrap_comments: true,
            reflow_doc_comments: true,
            reflow_non_doc_comments: true,
            format_code_blocks: false,
            comment_width: 80,
            ..Config::default()
        };
        // The fence opener, body, and closer are emitted verbatim.
        assert_format_with_config(src, src, config);
    }

    #[test]
    fn trailing_inline_comment_not_merged_with_following_standalone_comment() {
        let src = "fn foo() {
    let x = 1; // Some comment
    // This is something else
}
";
        assert_format_wrapping_comments(src, src, 80);
    }

    #[test]
    fn does_not_wrap_fenced_code_block_in_chunks_path_block_comment() {
        let src = "fn foo() {
    /*
    Example:

    ```
    fn inner() {
        x + y + z + a + b + c + d + e + f
    }
    ```

    After the fence.
    */
    let x = 1;
}
";
        assert_format_wrapping_comments(src, src, 30);
    }

    #[test]
    fn does_not_wrap_fenced_code_block_in_chunks_path_line_comments() {
        let src = "fn foo() {
    // Example:
    //
    // ```
    // fn inner() {
    //     x + y + z + a + b + c + d + e + f
    // }
    // ```
    //
    // After the fence.
    let x = 1;
}
";
        assert_format_wrapping_comments(src, src, 30);
    }

    #[test]
    fn reflow_url_line_passthrough() {
        let src = "fn foo() {
    // See https://example.com/some/very/long/path/that/exceeds for details
    let x = 1;
}
";
        let expected = "fn foo() {
    // See https://example.com/some/very/long/path/that/exceeds for details
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 30);
    }

    #[test]
    fn reflow_javadoc_tag_breaks_paragraph_in_doc_comment() {
        let src = "/// Build a Foo.
/// @return a fresh Foo
fn build() {}
";
        let expected = "/// Build a Foo.
/// @return a fresh Foo
fn build() {}
";
        assert_format_wrapping_comments(src, expected, 80);
    }

    #[test]
    fn reflows_two_line_paragraph_into_natural_wrap() {
        let src = "fn foo() {
    // Hello world, I just realized that this
    // is a long comment
    let x = 1;
}
";
        let expected = "fn foo() {
    // Hello world, I just realized
    // that this is a long comment
    let x = 1;
}
";
        assert_format_wrapping_comments(src, expected, 35);
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
     * that's going to be
     * wrapped. */
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
    /*
    This is a long comment
    that's going to be
    wrapped. This is a long
    comment that's going to
    be wrapped.
    */
    // This is a long comment
    // that's going to be
    // wrapped.
    /* This is a long comment
     * that's going to be
     * wrapped. */
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
}
