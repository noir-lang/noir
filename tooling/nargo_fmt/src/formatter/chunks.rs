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
    /// Push the current indentation to the indentation stack.
    PushIndentation,
    /// Set the current indetation by popping it from the indentation stack.
    PopIndentation,
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
            | Chunk::TrailingComma
            | Chunk::PushIndentation
            | Chunk::PopIndentation => 0,
        }
    }

    pub(crate) fn width_inside_an_expression_list(&self) -> usize {
        if let Chunk::Group(group) = &self {
            if let ChunkKind::LambdaAsLastExpressionInList { first_line_width, .. } = &group.kind {
                return *first_line_width;
            }
        }

        self.width()
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
            | Chunk::DecreaseIndentation
            | Chunk::PushIndentation
            | Chunk::PopIndentation => false,
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

    pub(crate) kind: ChunkKind,

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
            kind: ChunkKind::Regular,
            force_multiline_on_children_with_same_tag_if_multiline: false,
        }
    }

    pub(crate) fn text(&mut self, chunk: TextChunk) {
        if chunk.width == 0 {
            return;
        }

        if let Some(Chunk::Text(text_chunk)) = self.chunks.last_mut() {
            text_chunk.string.push_str(&chunk.string);
            text_chunk.width += chunk.width;
            text_chunk.has_newlines |= chunk.has_newlines;
        } else {
            self.push(Chunk::Text(chunk));
        }
    }

    /// Appends a TextChunk to this chunks chunks. However, if the last chunk is a group,
    /// it's appended to that group's last text.
    pub(crate) fn text_attached_to_last_group(&mut self, chunk: TextChunk) {
        if chunk.width == 0 {
            return;
        }

        if let Some(Chunk::Group(group)) = self.chunks.last_mut() {
            group.text(chunk);
        } else {
            self.text(chunk);
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
        self.lines(false);
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

    pub(crate) fn push_indentation(&mut self) {
        self.push(Chunk::PushIndentation);
    }

    pub(crate) fn pop_indentation(&mut self) {
        self.push(Chunk::PopIndentation);
    }

    pub(crate) fn push(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    pub(crate) fn width(&self) -> usize {
        self.chunks.iter().map(|chunk| chunk.width()).sum()
    }

    pub(crate) fn expression_list_width(&self) -> usize {
        if self.kind == ChunkKind::MethodCall {
            self.chunks
                .iter()
                .map(|chunk| {
                    if let Chunk::Group(group) = chunk {
                        if group.kind == ChunkKind::ExpressionList {
                            group.expression_list_width()
                        } else {
                            chunk.width_inside_an_expression_list()
                        }
                    } else {
                        chunk.width_inside_an_expression_list()
                    }
                })
                .sum()
        } else {
            self.chunks.iter().map(|chunk| chunk.width_inside_an_expression_list()).sum()
        }
    }

    pub(crate) fn has_newlines(&self) -> bool {
        self.force_multiple_lines || self.chunks.iter().any(|chunk| chunk.has_newlines())
    }

    pub(crate) fn has_lambda_as_last_expression_in_list(&self) -> bool {
        self.chunks.iter().any(|chunk| {
            if let Chunk::Group(group) = chunk {
                if self.kind == ChunkKind::MethodCall {
                    group.has_lambda_as_last_expression_in_list()
                } else {
                    matches!(group.kind, ChunkKind::LambdaAsLastExpressionInList { .. })
                }
            } else {
                false
            }
        })
    }

    pub(crate) fn set_lambda_as_last_expression_in_list_indentation(
        &mut self,
        indentation_to_set: usize,
    ) {
        for chunk in self.chunks.iter_mut() {
            if let Chunk::Group(group) = chunk {
                if self.kind == ChunkKind::MethodCall {
                    group.set_lambda_as_last_expression_in_list_indentation(indentation_to_set);
                } else if let ChunkKind::LambdaAsLastExpressionInList { indentation, .. } =
                    &mut group.kind
                {
                    if indentation.is_none() {
                        *indentation = Some(indentation_to_set);
                    }
                }
            }
        }
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
            kind: self.kind,
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
                Chunk::PushIndentation => chunks.push_indentation(),
                Chunk::PopIndentation => chunks.pop_indentation(),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ChunkKind {
    /// Most chunks are regular chunks and are not of interest.
    Regular,
    /// This is a chunk that has a list of expression in it, for example:
    /// a call, a method call, an array literal, a tuple literal, etc.
    ExpressionList,
    /// This is a chunk for a lambda argument that is the last expression of an ExpressionList.
    /// `first_line_width` is the width of the first line of the lambda argument: the parameters
    /// list and the left bracket.
    LambdaAsLastExpressionInList { first_line_width: usize, indentation: Option<usize> },
    /// A method call (one of its groups is an ExpressionList)
    MethodCall,
}

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
        if let ChunkKind::LambdaAsLastExpressionInList { indentation: Some(indentation), .. } =
            chunks.kind
        {
            let previous_indentation = self.indentation;
            self.indentation = indentation;
            self.format_chunks_impl_2(chunks);
            self.indentation = previous_indentation;
        } else {
            self.format_chunks_impl_2(chunks);
        }
    }

    pub(super) fn format_chunks_impl_2(&mut self, mut chunks: Chunks) {
        if chunks.force_multiple_lines {
            self.format_chunks_in_multiple_lines(chunks);
            return;
        }

        if chunks.has_newlines() {
            // When formatting an expression list we have to check if the last argument is a lambda,
            // because we format that in a special way:
            // 1. to compute the group width we'll consider only the `|...| {` part of the lambda
            // 2. If it fits in a line, we'll format this expression list in a single line
            // 3. However, an expression list is instructed to increase indentation after, say,
            //    `(` or `[` (depending on the expression list) and then the `{` part of a lambda
            //    will also increase the indentation, resulting in too much indentation.
            // 4. For that reason we decrease the indentation knowing that it will be increased
            //    again. This is fine (that's why indentation is i32 and not usize) and if we
            //    determined that we can format the expression list in a single line it means
            //    none of the expression up to the lambda will result in multiple lines, so
            //    this change in indentation won't affect them.
            let chunks_kind = chunks.kind;
            if (chunks_kind == ChunkKind::ExpressionList || chunks_kind == ChunkKind::MethodCall)
                && chunks.has_lambda_as_last_expression_in_list()
            {
                let chunks_width = chunks.expression_list_width();

                let total_width = self.current_line_width + chunks_width;
                if total_width <= self.config.max_width {
                    chunks.set_lambda_as_last_expression_in_list_indentation(self.indentation);
                    self.format_chunks_in_one_line(chunks);
                    return;
                }
            }

            self.format_chunks_in_multiple_lines(chunks);
            return;
        }

        let chunks_width = chunks.width();
        let total_width = self.current_line_width + chunks_width;
        if total_width > self.config.max_width {
            self.format_chunks_in_multiple_lines(chunks);
            return;
        }

        self.format_chunks_in_one_line(chunks);
    }

    pub(super) fn format_chunks_in_one_line(&mut self, chunks: Chunks) {
        for chunk in chunks.chunks {
            match chunk {
                Chunk::Text(text_chunk) => self.write(&text_chunk.string),
                Chunk::TrailingComment(text_chunk) | Chunk::LeadingComment(text_chunk) => {
                    self.write(&text_chunk.string);
                    self.write(" ");
                }
                Chunk::Group(chunks) => self.format_chunks_impl(chunks),
                Chunk::SpaceOrLine => self.write(" "),
                Chunk::IncreaseIndentation => self.increase_indentation(),
                Chunk::DecreaseIndentation => self.decrease_indentation(),
                Chunk::PushIndentation => self.push_indentation(),
                Chunk::PopIndentation => self.pop_indentation(),
                Chunk::TrailingComma | Chunk::Line { .. } => (),
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
                        self.write_space_without_skipping_whitespace_and_comments();
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
                        self.write(&text_chunk.string);
                    }
                }
                Chunk::TrailingComment(text_chunk) => {
                    self.write_chunk_lines(&text_chunk.string);
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.write_indentation();
                }
                Chunk::LeadingComment(text_chunk) => {
                    self.write_chunk_lines(text_chunk.string.trim());
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

                    self.format_chunks_impl(group);
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
                Chunk::PushIndentation => {
                    self.push_indentation();
                }
                Chunk::PopIndentation => {
                    self.pop_indentation();
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
