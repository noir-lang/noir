use noirc_frontend::token::Token;

use super::{chunks::TextChunk, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn write_space(&mut self) {
        self.skip_comments_and_whitespace();
        self.write_space_without_skipping_whitespace_and_comments();
    }

    pub(super) fn write_space_without_skipping_whitespace_and_comments(&mut self) {
        if !self.buffer.ends_with('\n') && !self.buffer.ends_with(' ') {
            self.write(" ");
        }
    }

    pub(super) fn skip_whitespace_if_it_is_not_a_newline(&mut self) {
        while let Token::Whitespace(whitespace) = &self.token {
            if whitespace.contains('\n') {
                break;
            }
            self.bump();
        }
    }

    pub(super) fn skip_comments_and_whitespace(&mut self) {
        self.skip_comments_and_whitespace_impl(
            false, // write lines
            false, // at beginning
        )
    }

    pub(super) fn skip_comments_and_whitespace_writing_lines_if_found(&mut self) {
        self.skip_comments_and_whitespace_impl(
            true,  // write lines
            false, // at beginning
        )
    }

    pub(super) fn skip_comments_and_whitespace_impl(
        &mut self,
        write_lines: bool,
        at_beginning: bool,
    ) {
        let mut number_of_newlines = 0;
        let mut passed_whitespace = false;
        let mut last_was_block_comment = false;
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
                Token::LineComment(_, None) => {
                    if number_of_newlines > 1 && write_lines {
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else {
                        if !(at_beginning && self.buffer.is_empty()) {
                            self.write_space_without_skipping_whitespace_and_comments();
                        }
                    }
                    self.write_current_token_trimming_end();
                    self.write_line_without_skipping_whitespace_and_comments();
                    number_of_newlines = 1;
                    self.bump();
                    passed_whitespace = false;
                    last_was_block_comment = false;
                    self.wrote_comment = true;
                }
                Token::BlockComment(_, None) => {
                    if number_of_newlines > 1 && write_lines {
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if passed_whitespace {
                        self.write_space_without_skipping_whitespace_and_comments();
                    }
                    self.write_current_token();
                    self.bump();
                    passed_whitespace = false;
                    last_was_block_comment = true;
                    self.wrote_comment = true;
                }
                _ => break,
            }
        }

        if number_of_newlines > 1 && write_lines {
            self.write_multiple_lines_without_skipping_whitespace_and_comments();
        }
    }

    pub(super) fn following_newlines_count(&mut self) -> usize {
        let Token::Whitespace(whitespace) = &self.token else {
            return 0;
        };

        whitespace.chars().filter(|char| *char == '\n').count()
    }

    pub(super) fn write_line(&mut self) {
        self.skip_comments_and_whitespace_impl(
            true,  // writing newline
            false, // at beginning
        );
        self.write_line_without_skipping_whitespace_and_comments();
    }

    pub(super) fn write_line_without_skipping_whitespace_and_comments(&mut self) -> bool {
        if !self.buffer.ends_with('\n') && !self.buffer.ends_with(' ') {
            self.write("\n");
            true
        } else {
            false
        }
    }

    pub(super) fn write_multiple_lines_without_skipping_whitespace_and_comments(&mut self) {
        if self.buffer.ends_with("\n\n") {
            // Nothing
        } else if self.buffer.ends_with("\n") {
            self.write("\n")
        } else {
            self.write("\n\n");
        }
    }

    pub(super) fn skip_comments_and_whitespace_chunk(&mut self) -> TextChunk {
        self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
        })
    }
}
