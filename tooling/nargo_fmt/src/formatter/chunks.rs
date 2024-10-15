use super::Formatter;

#[derive(Debug)]
pub(crate) struct TextChunk {
    pub(crate) string: String,
    pub(crate) width: usize,
    pub(crate) has_newlines: bool,
}

impl TextChunk {
    pub(crate) fn new(string: String) -> Self {
        TextChunk {
            width: string.chars().count(),
            has_newlines: string.chars().any(|char| char == '\n'),
            string,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Chunk {
    /// A text chunk. It might contain leading comments.
    Text(TextChunk),
    /// A text chunk that should only be written if we decide to format chunks in multiple lines.
    TextIfMultiline(TextChunk),
    /// A trailing comment (happens at the end of a line, and always after something else have been written).
    TrailingComment(TextChunk),
    /// A leading comment. Happens at the beginning of a line.
    LeadingComment(TextChunk),
    /// A group of chunks.
    Chunks(Chunks),
    /// Write a line if we decide to format chunks in multiple lines, otherwise do nothing.
    Line,
    /// Writes a space if we can write a group in one line, otherwise writes a line.
    /// However, a space might be written if `one_chunk_per_line` of a Chunks object is set to false.
    SpaceOrLine,
    /// Forces a line to be written.
    ForceLine,
    /// Command to increase the current indentation.
    IncreaseIndentation,
    /// Command to decrease the current indentation.
    DecreaseIndentation,
}

impl Chunk {
    pub(crate) fn width(&self) -> usize {
        match self {
            Chunk::Text(chunk) | Chunk::TrailingComment(chunk) | Chunk::LeadingComment(chunk) => {
                chunk.width
            }
            Chunk::Chunks(chunks) => chunks.width(),
            Chunk::SpaceOrLine => 1,
            Chunk::Line
            | Chunk::ForceLine
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation
            | Chunk::TextIfMultiline(..) => 0,
        }
    }

    pub(crate) fn has_newlines(&self) -> bool {
        match self {
            Chunk::Text(chunk)
            | Chunk::TextIfMultiline(chunk)
            | Chunk::TrailingComment(chunk)
            | Chunk::LeadingComment(chunk) => chunk.has_newlines,
            Chunk::Chunks(chunks) => chunks.has_newlines(),
            Chunk::Line
            | Chunk::SpaceOrLine
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation => false,
            Chunk::ForceLine => true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Chunks {
    chunks: Vec<Chunk>,
    one_chunk_per_line: bool,
}

impl Chunks {
    pub(crate) fn new() -> Self {
        Self { chunks: Vec::new(), one_chunk_per_line: true }
    }

    pub(crate) fn with_multiple_chunks_per_line(self) -> Self {
        Self { one_chunk_per_line: false, ..self }
    }

    pub(crate) fn text(&mut self, chunk: TextChunk) {
        if chunk.width > 0 {
            if let Some(Chunk::Text(text_chunk)) = self.chunks.last_mut() {
                text_chunk.string.push_str(&chunk.string);
                text_chunk.width += chunk.width;
                text_chunk.has_newlines |= chunk.has_newlines;
            } else {
                self.push(Chunk::Text(chunk));
            }
        }
    }

    pub(crate) fn trailing_comment(&mut self, chunk: TextChunk) {
        if chunk.width > 0 {
            self.push(Chunk::TrailingComment(chunk));
        }
    }

    pub(crate) fn leading_comment(&mut self, chunk: TextChunk) {
        if chunk.width > 0 {
            self.push(Chunk::LeadingComment(chunk));
        }
    }

    pub(crate) fn text_if_multiline(&mut self, chunk: TextChunk) {
        self.push(Chunk::TextIfMultiline(chunk));
    }

    pub(crate) fn chunks(&mut self, chunks: Chunks) {
        self.push(Chunk::Chunks(chunks));
    }

    pub(crate) fn line(&mut self) {
        self.push(Chunk::Line);
    }

    pub(crate) fn force_line(&mut self) {
        self.push(Chunk::ForceLine);
    }

    pub(crate) fn space_or_line(&mut self) {
        self.push(Chunk::SpaceOrLine);
    }

    pub(crate) fn increase_indentation(&mut self) {
        self.push(Chunk::IncreaseIndentation);
    }

    pub(crate) fn decrease_indentation(&mut self) {
        self.push(Chunk::DecreaseIndentation);
    }

    pub(crate) fn push(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    pub(crate) fn width(&self) -> usize {
        self.chunks.iter().map(|chunk| chunk.width()).sum()
    }

    pub(crate) fn has_newlines(&self) -> bool {
        self.chunks.iter().any(|chunk| chunk.has_newlines())
    }
}

impl<'a> Formatter<'a> {
    pub(super) fn chunk<F>(&mut self, f: F) -> TextChunk
    where
        F: FnOnce(&mut Formatter<'a>),
    {
        let previous_buffer = std::mem::take(&mut self.buffer);
        let previous_line_length = self.current_line_width;
        let previous_indentation = self.indentation;

        f(self);

        self.current_line_width = previous_line_length;
        self.indentation = previous_indentation;

        let string = std::mem::replace(&mut self.buffer, previous_buffer);
        TextChunk::new(string)
    }

    pub(super) fn skip_comments_and_whitespace_chunk(&mut self) -> TextChunk {
        self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
        })
    }

    pub(super) fn format_chunks(&mut self, chunks: Chunks) {
        if chunks.has_newlines() {
            self.format_chunks_in_multiple_lines(chunks);
        } else {
            let chunks_width = chunks.width();
            let total_width = self.current_line_width + chunks_width;
            if total_width > self.config.max_width {
                self.format_chunks_in_multiple_lines(chunks);
            } else {
                self.format_chunks_in_one_line(chunks);
            }
        }
    }

    pub(super) fn format_chunks_in_one_line(&mut self, chunks: Chunks) {
        for chunk in chunks.chunks {
            match chunk {
                Chunk::Text(text_chunk) => self.write(&text_chunk.string),
                Chunk::Chunks(chunks) => self.format_chunks_in_one_line(chunks),
                Chunk::SpaceOrLine => self.write(" "),
                Chunk::TextIfMultiline(..)
                | Chunk::TrailingComment(..)
                | Chunk::LeadingComment(..)
                | Chunk::Line
                | Chunk::IncreaseIndentation
                | Chunk::DecreaseIndentation => (),
                Chunk::ForceLine => unreachable!("Should not format ForceLine chunk in one line"),
            }
        }
    }

    pub(super) fn format_chunks_in_multiple_lines(&mut self, chunks: Chunks) {
        let mut last_was_space_or_line = false;

        for chunk in chunks.chunks {
            if last_was_space_or_line {
                if chunks.one_chunk_per_line {
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.write_indentation();
                } else {
                    // "+ 1" because we still need to add a space before the next chunk
                    if self.current_line_width + chunk.width() + 1 > self.config.max_width {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else {
                        self.write_space();
                    }
                }
            }

            last_was_space_or_line = false;

            match chunk {
                Chunk::Text(text_chunk) | Chunk::TextIfMultiline(text_chunk) => {
                    if text_chunk.has_newlines {
                        self.write_chunk_lines(&text_chunk.string);
                    } else {
                        self.write(&text_chunk.string)
                    }
                }
                Chunk::TrailingComment(text_chunk) => {
                    self.write_chunk_lines(&text_chunk.string);
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.write_indentation();
                }
                Chunk::LeadingComment(text_chunk) => {
                    self.write_chunk_lines(&text_chunk.string.trim());
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.write_indentation();
                }
                Chunk::Chunks(chunks) => self.format_chunks(chunks),
                Chunk::Line | Chunk::ForceLine => {
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.write_indentation();
                }
                Chunk::SpaceOrLine => {
                    last_was_space_or_line = true;
                }
                Chunk::IncreaseIndentation => {
                    self.increase_indentation();
                }
                Chunk::DecreaseIndentation => {
                    self.decrease_indentation();
                }
            }
        }
    }

    fn write_chunk_lines(&mut self, string: &str) {
        for (index, line) in string.lines().enumerate() {
            // Don't indent the first line (it should already be indented).
            // Also don't indent if the current line already has a space as the last char
            // (it means it's already indented)
            if index > 0 && !self.buffer.ends_with(' ') {
                self.write_line_without_skipping_whitespace_and_comments();
                // Only indent if the line doesn't start with a space. When that happens
                // it's likely a block comment part that we don't want to modify.
                if !line.starts_with(' ') {
                    self.write_indentation();
                }
            }

            // If we already have a space in the buffer and the line starts with a space,
            // don't repeat that space.
            if self.buffer.ends_with(' ') && line.starts_with(' ') {
                self.write(line.trim_start());
            } else {
                self.write(line);
            }
        }
    }
}
