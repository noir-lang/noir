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
    /// A trailing comma that's only written if we decide to format chunks in multiple lines
    /// (for example for a call we'll add a trailing comma to the last argument).
    TrailingComma,
    /// A trailing comment (happens at the end of a line, and always after something else have been written).
    TrailingComment(TextChunk),
    /// A leading comment. Happens at the beginning of a line.
    LeadingComment(TextChunk),
    /// A group of chunks.
    Group(Chunks),
    /// Write a line (or two) if we decide to format chunks in multiple lines, otherwise do nothing.
    Line { two: bool },
    /// Writes a space if we can write a group in one line, otherwise writes a line.
    /// However, a space might be written if `one_chunk_per_line` of a Chunks object is set to false.
    SpaceOrLine,
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
            Chunk::Group(chunks) => chunks.width(),
            Chunk::SpaceOrLine => 1,
            Chunk::Line { .. }
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation
            | Chunk::TrailingComma => 0,
        }
    }

    pub(crate) fn has_newlines(&self) -> bool {
        match self {
            Chunk::Text(chunk) | Chunk::TrailingComment(chunk) | Chunk::LeadingComment(chunk) => {
                chunk.has_newlines
            }
            Chunk::Group(chunks) => chunks.has_newlines(),
            Chunk::TrailingComma
            | Chunk::Line { .. }
            | Chunk::SpaceOrLine
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation => false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Chunks {
    pub(crate) chunks: Vec<Chunk>,
    pub(crate) one_chunk_per_line: bool,
    pub(crate) force_multiple_lines: bool,

    /// Chunks can be tagged. For example we tag chunks as `IfConsequenceOrAlternative` if they are consequences
    /// or alternatives of an `if` expression. Then, if we determine an outer if would
    /// exceed the maximum allowed length for an if, we tell all tha inner chunks marked
    /// as `if`
    pub(crate) tag: Option<ChunkTag>,

    /// This name is a bit long and explicit, but it's to make things clearer:
    /// if we determine that this group needs to be formatted in multiple lines,
    /// children groups with the same tag will also be formatted in multiple lines.
    ///
    /// This is used for example in infix expressions like `a + b + c + d`, where if we
    /// determine that `a + b` needs to be formatted in multiple lines, we want the entire
    /// tree (of those infix expressions) to be formatted in multiple lines.
    pub(crate) force_multiline_on_children_with_same_tag_if_multiline: bool,
}

impl Chunks {
    pub(crate) fn new() -> Self {
        Self {
            chunks: Vec::new(),
            one_chunk_per_line: true,
            force_multiple_lines: false,
            tag: None,
            force_multiline_on_children_with_same_tag_if_multiline: false,
        }
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

    pub(crate) fn trailing_comma(&mut self) {
        self.push(Chunk::TrailingComma);
    }

    pub(crate) fn group(&mut self, chunks: Chunks) {
        self.push(Chunk::Group(chunks));
    }

    /// Append one line to this chunk.
    pub(crate) fn line(&mut self) {
        self.lines(false)
    }

    /// Append one or two lines to this chunk.
    pub(crate) fn lines(&mut self, two: bool) {
        self.push(Chunk::Line { two });
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
        self.force_multiple_lines || self.chunks.iter().any(|chunk| chunk.has_newlines())
    }

    /// Before writing a Chunks object in multiple lines, create a new one where `TrailingComma`
    /// is turned into `Text`. Because Chunks will glue two consecutive `Text`s together, if we
    /// have two chunks `Text("123"), TrailingComma`, we'll consider the entire string "123,"
    /// when deciding whether we can still write in the current line or not.
    pub(crate) fn prepare_for_multiple_lines(self) -> Chunks {
        let mut chunks = Chunks {
            chunks: Vec::new(),
            one_chunk_per_line: self.one_chunk_per_line,
            force_multiple_lines: self.force_multiple_lines,
            tag: self.tag,
            force_multiline_on_children_with_same_tag_if_multiline: self
                .force_multiline_on_children_with_same_tag_if_multiline,
        };

        for chunk in self.chunks {
            match chunk {
                Chunk::Text(chunk) => chunks.text(chunk),
                Chunk::TrailingComma => {
                    // If there's a trailing comma after a group, append the text to that group
                    // so that it glues with the last text present there (if any)
                    if let Some(Chunk::Group(group)) = chunks.chunks.last_mut() {
                        group.add_trailing_comma_to_last_text();
                    } else {
                        chunks.text(TextChunk::new(",".to_string()));
                    }
                }
                Chunk::TrailingComment(chunk) => chunks.trailing_comment(chunk),
                Chunk::LeadingComment(chunk) => chunks.leading_comment(chunk),
                Chunk::Group(group) => chunks.group(group),
                Chunk::Line { two } => chunks.lines(two),
                Chunk::SpaceOrLine => chunks.space_or_line(),
                Chunk::IncreaseIndentation => chunks.increase_indentation(),
                Chunk::DecreaseIndentation => chunks.decrease_indentation(),
            }
        }
        chunks
    }

    fn add_trailing_comma_to_last_text(&mut self) {
        if let Some(Chunk::Group(group)) = self.chunks.last_mut() {
            group.add_trailing_comma_to_last_text();
        } else {
            self.text(TextChunk::new(",".to_string()));
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct ChunkTag(usize);

impl<'a> Formatter<'a> {
    pub(super) fn chunk<F>(&mut self, f: F) -> TextChunk
    where
        F: FnOnce(&mut Formatter<'a>),
    {
        let previous_buffer = std::mem::take(&mut self.buffer);
        let previous_line_width = self.current_line_width;
        let previous_indentation = self.indentation;
        self.current_line_width = 0;
        self.indentation = 0;

        f(self);

        self.current_line_width = previous_line_width;
        self.indentation = previous_indentation;

        let string = std::mem::replace(&mut self.buffer, previous_buffer);
        TextChunk::new(string)
    }

    pub(super) fn format_chunks(&mut self, chunks: Chunks) {
        let previous_indentation = self.indentation;
        self.format_chunks_impl(chunks);
        self.indentation = previous_indentation;
    }

    pub(super) fn format_chunks_impl(&mut self, chunks: Chunks) {
        if chunks.force_multiple_lines || chunks.has_newlines() {
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
                Chunk::TrailingComment(text_chunk) | Chunk::LeadingComment(text_chunk) => {
                    self.write(&text_chunk.string);
                    self.write(" ");
                }
                Chunk::Group(chunks) => self.format_chunks_in_one_line(chunks),
                Chunk::SpaceOrLine => self.write(" "),
                Chunk::TrailingComma
                | Chunk::Line { .. }
                | Chunk::IncreaseIndentation
                | Chunk::DecreaseIndentation => (),
            }
        }
    }

    pub(super) fn format_chunks_in_multiple_lines(&mut self, chunks: Chunks) {
        let chunks = chunks.prepare_for_multiple_lines();

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
                Chunk::Text(text_chunk) => {
                    if text_chunk.has_newlines {
                        self.write_chunk_lines(&text_chunk.string);
                    } else {
                        // If we didn't exceed the max width, but this chunk will, insert a newline,
                        // increase indentation and indent (the indentation will be undone
                        // after `format_chunks` finishes)
                        if self.current_line_width <= self.config.max_width
                            && self.current_line_width + text_chunk.width > self.config.max_width
                            && !self.buffer.ends_with(' ')
                        {
                            self.write_line_without_skipping_whitespace_and_comments();
                            self.increase_indentation();
                            self.write_indentation();
                        }
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
                Chunk::Group(mut group) => {
                    if chunks.force_multiline_on_children_with_same_tag_if_multiline
                        && chunks.tag == group.tag
                    {
                        group.force_multiple_lines = true;
                        group.force_multiline_on_children_with_same_tag_if_multiline = true;
                    }

                    self.format_chunks_impl(group)
                }
                Chunk::Line { two } => {
                    if two {
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                    } else {
                        self.write_line_without_skipping_whitespace_and_comments();
                    }
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
                Chunk::TrailingComma => {
                    unreachable!(
                        "TrailingComma should have been removed by `prepare_for_multiple_lines`"
                    )
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

    pub(super) fn next_chunk_tag(&mut self) -> ChunkTag {
        let tag = ChunkTag(self.next_chunk_tag);
        self.next_chunk_tag += 1;
        tag
    }
}
