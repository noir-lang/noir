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
    Text(TextChunk),
    TextIfMultiline(TextChunk),
    Line,
    SpaceOrLine,
    IncreaseIndentation,
    DecreaseIndentation,
}

impl Chunk {
    pub(crate) fn width(&self) -> usize {
        match self {
            Chunk::Text(chunk) => chunk.width,
            Chunk::SpaceOrLine => 1,
            Chunk::Line
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation
            | Chunk::TextIfMultiline(..) => 0,
        }
    }

    pub(crate) fn has_newlines(&self) -> bool {
        match self {
            Chunk::Text(text_chunk) | Chunk::TextIfMultiline(text_chunk) => text_chunk.has_newlines,
            Chunk::Line
            | Chunk::SpaceOrLine
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation => false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Chunks {
    chunks: Vec<Chunk>,
}

impl Chunks {
    pub(crate) fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub(crate) fn text(&mut self, chunk: TextChunk) {
        if chunk.width > 0 {
            self.chunks.push(Chunk::Text(chunk));
        }
    }

    pub(crate) fn text_if_multiline(&mut self, chunk: TextChunk) {
        self.chunks.push(Chunk::TextIfMultiline(chunk));
    }

    pub(crate) fn line(&mut self) {
        self.chunks.push(Chunk::Line);
    }

    pub(crate) fn space_or_line(&mut self) {
        self.chunks.push(Chunk::SpaceOrLine);
    }

    pub(crate) fn increase_indentation(&mut self) {
        self.chunks.push(Chunk::IncreaseIndentation);
    }

    pub(crate) fn decrease_indentation(&mut self) {
        self.chunks.push(Chunk::DecreaseIndentation);
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

        let string =
            std::mem::replace(&mut self.buffer, previous_buffer).trim_end_matches(' ').to_string();
        TextChunk::new(string)
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

    fn format_chunks_in_one_line(&mut self, chunks: Chunks) {
        for chunk in chunks.chunks {
            match chunk {
                Chunk::Text(text_chunk) => self.write(&text_chunk.string),
                Chunk::SpaceOrLine => self.write(" "),
                Chunk::TextIfMultiline(..)
                | Chunk::Line
                | Chunk::IncreaseIndentation
                | Chunk::DecreaseIndentation => (),
            }
        }
    }

    fn format_chunks_in_multiple_lines(&mut self, chunks: Chunks) {
        for chunk in chunks.chunks {
            match chunk {
                Chunk::Text(text_chunk) | Chunk::TextIfMultiline(text_chunk) => {
                    self.write(&text_chunk.string)
                }
                Chunk::Line | Chunk::SpaceOrLine => {
                    self.write_line();
                    self.write_indentation();
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
}
