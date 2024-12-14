//! This module has all the logic to format a series of chunks (a piece of text) in a way
//! that we (almost always) never exceed the configurable maximum line width.
//!
//! It's heavily inspired by this excellent blog post:
//!
//! https://yorickpeterse.com/articles/how-to-write-a-code-formatter/
//!
//! However, some changes were introduces to handle comments and other particularities of Noir.
use std::ops::Deref;

use noirc_frontend::token::Token;

use super::Formatter;

/// A text chunk. It precomputes the text width and whether it has newlines.
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

/// A chunk can either be text or a directive that instructs the formatter to do something
/// (for example: increase or decrease the current indentation).
#[derive(Debug)]
pub(crate) enum Chunk {
    /// A text chunk. It might contain leading comments.
    Text(TextChunk),
    /// A text chunk that should be printed unmodified (used for `quote { ... }` contents).
    Verbatim(TextChunk),
    /// A trailing comma that's only written if we decide to format chunks in multiple lines
    /// (for example for a call we'll add a trailing comma to the last argument).
    TrailingComma,
    /// A trailing comment (happens at the end of a line, and always after something else have been written).
    TrailingComment(TextChunk),
    /// A leading comment. Happens at the beginning of a line.
    LeadingComment(TextChunk),
    /// A group of chunks.
    Group(ChunkGroup),
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
    /// Set the current indentation by popping it from the indentation stack.
    PopIndentation,
}

impl Chunk {
    pub(crate) fn width(&self) -> usize {
        match self {
            Chunk::Text(chunk)
            | Chunk::Verbatim(chunk)
            | Chunk::TrailingComment(chunk)
            | Chunk::LeadingComment(chunk) => chunk.width,
            Chunk::Group(group) => group.width(),
            Chunk::SpaceOrLine => 1,
            Chunk::Line { .. }
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation
            | Chunk::TrailingComma
            | Chunk::PushIndentation
            | Chunk::PopIndentation => 0,
        }
    }

    /// Computes the width of this chunk considering it's inside an ExpressionList.
    /// The only thing that changes here compared to `width` is that a LambdaAsLastExpressionInList's
    /// width is considered to be only the first line, so we can avoid splitting the entire call
    /// arguments into separate lines.
    pub(crate) fn width_inside_an_expression_list(&self) -> usize {
        if let Chunk::Group(group) = &self {
            if let GroupKind::LambdaAsLastExpressionInList { first_line_width, .. } = &group.kind {
                return *first_line_width;
            }
        }

        self.width()
    }

    pub(crate) fn has_newlines(&self) -> bool {
        match self {
            Chunk::Text(chunk)
            | Chunk::Verbatim(chunk)
            | Chunk::TrailingComment(chunk)
            | Chunk::LeadingComment(chunk) => chunk.has_newlines,
            Chunk::Group(group) => group.has_newlines(),
            Chunk::TrailingComma
            | Chunk::Line { .. }
            | Chunk::SpaceOrLine
            | Chunk::IncreaseIndentation
            | Chunk::DecreaseIndentation
            | Chunk::PushIndentation
            | Chunk::PopIndentation => false,
        }
    }

    /// Returns the current chunk as a Group, if it is one. Otherwise returns None.
    pub(crate) fn group(self) -> Option<ChunkGroup> {
        if let Chunk::Group(group) = self {
            Some(group)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct ChunkGroup {
    pub(crate) chunks: Vec<Chunk>,

    /// If `true`, when formatting in multiple lines, and after a SpaceOrLine,
    /// a line will be written.
    /// If `false`, when formatting in multiple lines, and after a SpaceOrLine,
    /// a space will be inserted and the next chunk will go in the same line if
    /// it fits that line.
    ///
    /// This is used to, for example, control how arrays are formatted. If each
    /// element is short, we'll format the array like this:
    ///
    /// [
    ///   1, 2, 3,
    ///   4, 5
    /// ]
    ///
    /// but if one of the elements is long, each one will go in a separate line:
    ///
    /// ```text
    /// [
    ///     1,
    ///     1234567890123,
    ///     3
    /// ]
    /// ```
    pub(crate) one_chunk_per_line: bool,

    /// If true, regardless of this group's chunks, this group will be formatted in
    /// multiple lines.
    /// This is set to true when, for example, we format a block that has at least
    /// two statements: we always want to show that in multiple lines.
    pub(crate) force_multiple_lines: bool,

    /// Groups can be tagged. For example we tag all consequences and alternative blocks
    /// of an `if` expression. If we determine one of them needs to be formatted in multiple
    /// lines, we find all other chunks with the same tag and mark them too to be formatted
    /// in multiple lines.
    pub(crate) tag: Option<GroupTag>,

    /// The kind of this group. Some group kinds are formatted in a special way
    /// (mainly lambda arguments that are the last expression in a list).
    pub(crate) kind: GroupKind,

    /// This name is a bit long and explicit, but it's to make things clearer:
    /// if we determine that this group needs to be formatted in multiple lines,
    /// children groups with the same tag will also be formatted in multiple lines.
    ///
    /// This is used for example in infix expressions like `a + b + c + d`, where if we
    /// determine that `a + b` needs to be formatted in multiple lines, we want the entire
    /// tree (of those infix expressions) to be formatted in multiple lines.
    pub(crate) force_multiline_on_children_with_same_tag_if_multiline: bool,
}

impl ChunkGroup {
    pub(crate) fn new() -> Self {
        Self {
            chunks: Vec::new(),
            one_chunk_per_line: true,
            force_multiple_lines: false,
            tag: None,
            kind: GroupKind::Regular,
            force_multiline_on_children_with_same_tag_if_multiline: false,
        }
    }

    /// Appends a text to this group.
    /// If the last chunk in this group is a text, no new chunk is inserted and
    /// instead the last text chunk is extended.
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

    /// Appends a verbatim text chunk to this group.
    pub(crate) fn verbatim(&mut self, chunk: TextChunk) {
        if chunk.width == 0 {
            return;
        }

        self.push(Chunk::Verbatim(chunk));
    }

    /// Appends a single space to this group by reading it from the given formatter.
    pub(crate) fn space(&mut self, formatter: &mut ChunkFormatter<'_, '_>) {
        self.text(formatter.chunk(|formatter| {
            formatter.write_space();
        }));
    }

    /// Appends a semicolon to this group by reading it from the given formatter.
    /// This will actually end up attaching the semicolon to the last text in this
    /// group so that we don't end up with stray semicolons.
    pub(crate) fn semicolon(&mut self, formatter: &mut ChunkFormatter<'_, '_>) {
        self.text_attached_to_last_group(formatter.chunk(|formatter| {
            formatter.write_semicolon();
        }));
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

    /// Appends a trailing comment (it's formatted slightly differently than a regular text chunk).
    pub(crate) fn trailing_comment(&mut self, chunk: TextChunk) {
        if chunk.width > 0 {
            self.push(Chunk::TrailingComment(chunk));
        }
    }

    /// Appends a leading comment (it's formatted slightly differently than a regular text chunk).
    pub(crate) fn leading_comment(&mut self, chunk: TextChunk) {
        if chunk.width > 0 {
            self.push(Chunk::LeadingComment(chunk));
        }
    }

    /// Appends a trailing comma (will only show up if the group is formatted in multiple lines).
    pub(crate) fn trailing_comma(&mut self) {
        self.push(Chunk::TrailingComma);
    }

    /// Appends another group as a nested group.
    pub(crate) fn group(&mut self, group: ChunkGroup) {
        self.push(Chunk::Group(group));
    }

    /// Append one line to this chunk.
    pub(crate) fn line(&mut self) {
        self.lines(false);
    }

    /// Append one or two lines to this chunk.
    pub(crate) fn lines(&mut self, two: bool) {
        self.push(Chunk::Line { two });
    }

    /// Appends a SpaceOrLine chunk, which means that it's a space when this group is
    /// formatted in a single line, or a line when it's formatted in multiple lines.
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
        self.chunks.iter().map(|chunk| chunk.width_inside_an_expression_list()).sum()
    }

    pub(crate) fn has_newlines(&self) -> bool {
        self.force_multiple_lines || self.chunks.iter().any(|chunk| chunk.has_newlines())
    }

    /// Determines if this group has a LambdaAsLastExpressionInList chunk.
    /// Note that if this group is a MethodCall, this is checked for the ExpressionList group
    /// inside it.
    pub(crate) fn has_lambda_as_last_expression_in_list(&self) -> bool {
        self.chunks.iter().any(|chunk| {
            if let Chunk::Group(group) = chunk {
                if self.kind.is_method_call() && group.kind.is_expression_list() {
                    group.has_lambda_as_last_expression_in_list()
                } else {
                    matches!(group.kind, GroupKind::LambdaAsLastExpressionInList { .. })
                }
            } else {
                false
            }
        })
    }

    /// Finds the `LambdaAsLastExpressionInList` associated to this group and sets its indentation
    /// to the given value.
    pub(crate) fn set_lambda_as_last_expression_in_list_indentation(
        &mut self,
        indentation_to_set: i32,
    ) {
        for chunk in self.chunks.iter_mut() {
            if let Chunk::Group(group) = chunk {
                if self.kind.is_method_call() && group.kind.is_expression_list() {
                    group.set_lambda_as_last_expression_in_list_indentation(indentation_to_set);
                } else if let GroupKind::LambdaAsLastExpressionInList { indentation, .. } =
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
    pub(crate) fn prepare_for_multiple_lines(self) -> ChunkGroup {
        let mut group = ChunkGroup { chunks: Vec::new(), ..self };

        for chunk in self.chunks {
            match chunk {
                Chunk::Text(chunk) => group.text(chunk),
                Chunk::Verbatim(chunk) => group.verbatim(chunk),
                Chunk::TrailingComma => {
                    // If there's a trailing comma after a group, append the text to that group
                    // so that it glues with the last text present there (if any)
                    group.add_trailing_comma_to_last_text();
                }
                Chunk::TrailingComment(chunk) => group.trailing_comment(chunk),
                Chunk::LeadingComment(chunk) => group.leading_comment(chunk),
                Chunk::Group(inner_group) => group.group(inner_group),
                Chunk::Line { two } => group.lines(two),
                Chunk::SpaceOrLine => group.space_or_line(),
                Chunk::IncreaseIndentation => group.increase_indentation(),
                Chunk::DecreaseIndentation => group.decrease_indentation(),
                Chunk::PushIndentation => group.push_indentation(),
                Chunk::PopIndentation => group.pop_indentation(),
            }
        }
        group
    }

    fn add_trailing_comma_to_last_text(&mut self) {
        if let Some(Chunk::Group(group)) = self.chunks.last_mut() {
            group.add_trailing_comma_to_last_text();
        } else {
            self.text(TextChunk::new(",".to_string()));
        }
    }

    /// Returns the width of text until we hit a Line or LineOrSpace, together
    /// with whether we hit a Line or LineOrSpace.
    fn width_until_line(&self) -> (usize, bool) {
        let mut width = 0;
        for chunk in &self.chunks {
            match chunk {
                Chunk::Text(text_chunk)
                | Chunk::Verbatim(text_chunk)
                | Chunk::TrailingComment(text_chunk)
                | Chunk::LeadingComment(text_chunk) => {
                    width += text_chunk.width;
                }
                Chunk::Group(chunk_group) => {
                    let (group_width, hit_line) = chunk_group.width_until_line();
                    width += group_width;
                    if hit_line {
                        return (width, true);
                    }
                }
                Chunk::Line { .. } | Chunk::SpaceOrLine => {
                    return (width, true);
                }
                Chunk::IncreaseIndentation
                | Chunk::DecreaseIndentation
                | Chunk::PushIndentation
                | Chunk::PopIndentation
                | Chunk::TrailingComma => (),
            }
        }

        (width, false)
    }

    fn first_group(&self) -> Option<&ChunkGroup> {
        self.chunks
            .iter()
            .filter_map(|chunk| if let Chunk::Group(group) = chunk { Some(group) } else { None })
            .next()
    }

    fn has_expression_list_or_method_call_group(&self) -> bool {
        for chunk in &self.chunks {
            if let Chunk::Group(group) = chunk {
                if group.kind.is_expression_list() || group.kind.is_method_call() {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct GroupTag(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum GroupKind {
    /// Most chunks are regular chunks and are not of interest.
    Regular,
    /// This is a chunk that has a list of expression in it, for example:
    /// a call, a method call, an array literal, a tuple literal, etc.
    /// `prefix_width` is the width of whatever is before the actual expression list.
    /// For example, for an array this is 1 (for "["), for a slice it's 2 ("&["), etc.
    ExpressionList { prefix_width: usize, expressions_count: usize },
    /// This is a chunk for a lambda argument that is the last expression of an ExpressionList.
    /// `first_line_width` is the width of the first line of the lambda argument: the parameters
    /// list and the left bracket.
    LambdaAsLastExpressionInList { first_line_width: usize, indentation: Option<i32> },
    /// The body of a lambda.
    /// We track this as a group kind so that when we have to write it, if it doesn't
    /// fit in the current line and it's not a block, instead of splitting that expression
    /// somewhere that's probably undesired, we'll "turn it" into a block
    /// (write the "{" and "}" delimiters) and write the lambda body in the next line.
    LambdaBody {
        block_statement_count: Option<usize>,
        has_comments: bool,
        lambda_has_return_type: bool,
    },
    /// A method call.
    /// We track all this information to see, if we end up needing to format this call
    /// in multiple lines, if we can write everything up to the left parentheses (inclusive)
    /// in one line, and just the call arguments in multiple lines.
    MethodCall {
        /// This is the width of the group until the left parenthesis (inclusive).
        width_until_left_paren_inclusive: usize,
        /// Are there newlines before the left parentheses in this group?
        has_newlines_before_left_paren: bool,
        /// Is this method call the left-hand side of a call chain? If so, this is true,
        /// otherwise this is false an it means it's the outermost call.
        lhs: bool,
    },
    /// The value of an assignment or let statement. We know this is the last group in a chunk so
    /// if it doesn't fit in the current line but it fits in the next line, we can
    /// write a newline, indent, and put it there (instead of writing the value in
    /// multiple lines).
    AssignValue,
}

impl GroupKind {
    fn is_method_call(&self) -> bool {
        matches!(self, GroupKind::MethodCall { .. })
    }

    fn is_expression_list(&self) -> bool {
        matches!(self, GroupKind::ExpressionList { .. })
    }
}

/// Interface for creating TextChunks.
pub(crate) struct ChunkFormatter<'a, 'b>(&'b mut Formatter<'a>);

impl<'a, 'b> ChunkFormatter<'a, 'b> {
    pub(crate) fn new(formatter: &'b mut Formatter<'a>) -> Self {
        Self(formatter)
    }

    /// Stops writing to the current buffer for the duration of the `f` call, which takes
    /// a formatter to write to. Then, returns a `TextChunk` with the written text.
    ///
    /// This allows a caller to format pieces of code and then pre-process them before
    /// writing to the main buffer.
    pub(crate) fn chunk(&mut self, f: impl FnOnce(&mut Formatter)) -> TextChunk {
        let previous_buffer = std::mem::take(&mut self.0.buffer);
        let previous_indentation = self.0.indentation;
        self.0.indentation = 0;

        f(self.0);

        self.0.indentation = previous_indentation;

        let buffer = std::mem::replace(&mut self.0.buffer, previous_buffer);
        TextChunk::new(buffer.contents())
    }

    /// Stops writing to the current buffer, skips comments and whitespaces (formatting them)
    /// and returns the formatted result as a `TextChunk`.
    pub(crate) fn skip_comments_and_whitespace_chunk(&mut self) -> TextChunk {
        self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
        })
    }

    pub(super) fn new_group_tag(&mut self) -> GroupTag {
        self.0.new_group_tag()
    }

    pub(super) fn bump(&mut self) -> Token {
        self.0.bump()
    }
}

/// Treating a `ChunkFormatter` as a `Formatter` in read-only mode is always fine,
/// and reduces some boilerplate.
impl<'a, 'b> Deref for ChunkFormatter<'a, 'b> {
    type Target = Formatter<'b>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Formatter<'a> {
    /// Returns an object that has a `chunk` method to get a TextChunk.
    /// This method exists so that we can't mix the two operation modes:
    /// using the formatter directly while writing to the buffer, or creating text chunks.
    pub(super) fn chunk_formatter(&mut self) -> ChunkFormatter<'a, '_> {
        ChunkFormatter::new(self)
    }

    /// Main interface to format a chunk group.
    /// Here it's determined if the chunk will group in a single line or multiple lines.
    pub(super) fn format_chunk_group(&mut self, group: ChunkGroup) {
        let previous_indentation = self.indentation;
        self.format_chunk_group_impl(group);
        self.indentation = previous_indentation;
    }

    pub(super) fn format_chunk_group_impl(&mut self, group: ChunkGroup) {
        if let GroupKind::LambdaAsLastExpressionInList { indentation: Some(indentation), .. } =
            group.kind
        {
            let previous_indentation = self.indentation;
            self.indentation = indentation;
            self.format_chunks_group_impl_without_lambda_handling(group);
            self.indentation = previous_indentation;
        } else {
            self.format_chunks_group_impl_without_lambda_handling(group);
        }
    }

    pub(super) fn format_chunks_group_impl_without_lambda_handling(
        &mut self,
        mut group: ChunkGroup,
    ) {
        let chunks_width = group.width();
        let total_width = self.current_line_width() + chunks_width;

        if total_width > self.max_width {
            // If this is a method call that doesn't fit in the current line, we check if
            // everything that follows up to the left parentheses fits in the current line.
            // If so, we write that and we'll end up formatting the arguments in multiple
            // lines, instead of splitting this entire call chain in multiple lines.
            //
            // For example, a call like this:
            //
            // foo.bar.baz.qux(1)
            //
            // if it exceeds the maximum width, will end up being formatted like this:
            //
            // foo.bar.baz.qux(
            //     1,
            // )
            //
            // instead of like this (many more lines):
            //
            // foo
            //     .bar
            //     .baz
            //     .qux(1)
            //
            // This is something that rustfmt seems to do too.
            if let GroupKind::MethodCall {
                width_until_left_paren_inclusive,
                has_newlines_before_left_paren: false,
                lhs: false,
            } = group.kind
            {
                let total_width = self.current_line_width() + width_until_left_paren_inclusive;
                if total_width <= self.max_width {
                    // Check if this method call has another call or method call nested in it.
                    // If not, it means tis is the last nested call and after it we'll need to start
                    // writing at least one closing parentheses. So the argument list will actually
                    // have one less character available for writing, and that's why we (temporarily) decrease
                    // max width.
                    let expression_list_group = group.first_group().unwrap();
                    let has_expression_list_or_call_group =
                        expression_list_group.has_expression_list_or_method_call_group();
                    if !has_expression_list_or_call_group {
                        self.max_width -= 1;
                    }

                    // When a method call's group is formed, we indent after the first dot. But with that
                    // indentation, and the arguments indentation, we'll end up with too much indentation,
                    // so here we decrease it to compensate that.
                    self.decrease_indentation();
                    self.format_chunk_group_in_one_line(group);
                    self.increase_indentation();

                    if !has_expression_list_or_call_group {
                        self.max_width += 1;
                    }
                    return;
                }
            }

            // If this is an expression list with a single expression, see if we can fit whatever
            // comes next until a line in the current line. For example, if we have this:
            //
            // foo(bar(baz(1)))
            //
            // then `foo(...)` is an ExpressionList. We check if `foo(` fits in the current line.
            // If yes, we write it in the current line and continue. Then we'll find `bar(...)`,
            // which is also an ExpressionList, and if `bar(` fits the current line, we'll write it,
            // etc. But we only do this if we have nested calls (nested expression lists, etc.)
            //
            // This is to avoid formatting the above like this:
            //
            // foo(
            //     bar(
            //         baz(
            //             1,
            //         ),
            //     ),
            // )
            //
            // (rustfmt seems to do the same thing)
            if let GroupKind::ExpressionList { prefix_width, expressions_count: 1 } = group.kind {
                if let Some(inner_group) = group.first_group() {
                    if inner_group.kind.is_expression_list() || inner_group.kind.is_method_call() {
                        let total_width = self.current_line_width()
                            + prefix_width
                            + inner_group.width_until_line().0;
                        if total_width <= self.max_width {
                            self.decrease_indentation();
                            self.format_chunk_group_in_one_line(group);
                            self.increase_indentation();
                            return;
                        }
                    }
                }
            }
        }

        if group.force_multiple_lines {
            self.format_chunk_group_in_multiple_lines(group);
            return;
        }

        // if chunks.has_newlines() {
        // When formatting an expression list we have to check if the last argument is a lambda,
        // because we format that in a special way:
        // 1. to compute the group width we'll consider only the `|...| {` part of the lambda
        // 2. If it fits in a line, we'll format this expression list in a single line
        // 3. However, an expression list is instructed to increase indentation after, say,
        //    `(` or `[` (depending on the expression list) and then the `{` part of a lambda
        //    will also increase the indentation, resulting in too much indentation.
        // 4. For that reason we adjust the lambda to be formatted with the indentation
        //    we have right that (that is, that of the call that holds the lambda).
        //    We do that by setting the `indentation` field of the LambdaAsLastExpressionInList.
        //
        // Note that this logic is a bit complex because for method calls, the arguments list
        // is in a group so all arguments can potentially be formatted in a single line, and
        // that group has the `ExpressionList` kind. The method call itself has the `MethodCall`
        // kind. So when determining the first line width of a method call with a lambda as
        // the last argument we have to find the nested ExpressionList and do some nested calls.
        if (group.kind.is_expression_list() || group.kind.is_method_call())
            && group.has_lambda_as_last_expression_in_list()
        {
            let chunks_width = group.expression_list_width();
            let total_width = self.current_line_width() + chunks_width;
            if total_width <= self.max_width {
                group.set_lambda_as_last_expression_in_list_indentation(self.indentation);
                self.format_chunk_group_in_one_line(group);
                return;
            }
        }

        if group.has_newlines() {
            self.format_chunk_group_in_multiple_lines(group);
            return;
        }

        // Check if the group first in the remainder of the current line.
        if total_width > self.max_width {
            // If this chunk is the value of an assignment (either a normal assignment or a let statement)
            // and it doesn't fit the current line, we check if it fits the next line with an increased
            // indentation.
            //
            // That way this:
            //
            // let x = foo(1, 2);
            //                 ^
            //                 assume the max width is here
            //
            // is formatted like this:
            //
            // let x =
            //     foo(1, 2);
            //
            // instead of:
            //
            // let x = foo(
            //     1,
            //     2,
            // )
            if group.kind == GroupKind::AssignValue {
                let total_width_next_line =
                    (self.indentation as usize + 1) * self.config.tab_spaces + chunks_width;
                if total_width_next_line <= self.max_width {
                    // We might have trailing spaces
                    // (for example a space after the `=` of a let statement or an assignment)
                    self.trim_spaces();
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.increase_indentation();
                    self.write_indentation();
                    self.format_chunk_group_in_one_line(group);
                    self.decrease_indentation();
                    return;
                }
            }

            // If a lambda body doesn't fit in the current line and it's not a block,
            // we can turn it into a block and write it in the next line, so its contents fit.
            if let GroupKind::LambdaBody { block_statement_count: None, .. } = group.kind {
                // Try to format it again in the next line, but we don't want to recurse
                // infinitely so we change the group kind.
                group.kind = GroupKind::Regular;
                self.write("{");
                self.trim_spaces();
                self.increase_indentation();
                self.write_line_without_skipping_whitespace_and_comments();
                self.write_indentation();
                self.format_chunk_group_impl(group);

                // If this lambda was in an expression list and it was formatted in multiple
                // lines, it might be that the trailing comma happened after the lambda body:
                //
                // foo(
                //     1,
                //     |lambda| body,
                // )
                //
                // Because we attach commas to the last text to avoid splitting it, the body
                // in this case is "body,", so if we end up writing it as a block it will
                // look like this:
                //
                // foo(
                //     1,
                //     |lambda| {
                //         body,
                //     }
                // )
                //
                // So, if after writing the body we find a comma (there will be at most one)
                // we remove it, but place it after the right brace, so it looks like this:
                //
                // foo(
                //     1,
                //     |lambda| {
                //         body
                //     },
                // )
                let comma_trimmed = self.trim_comma();
                self.decrease_indentation();
                self.write_line_without_skipping_whitespace_and_comments();
                self.write_indentation();
                self.write("}");
                if comma_trimmed {
                    self.write(",");
                }
                return;
            }

            self.format_chunk_group_in_multiple_lines(group);
            return;
        }

        // At this point we determined we are going to write this group in a single line.
        // If the current group is a lambda body that is a block with a single statement, like this:
        //
        // |x| { 1 + 2 }
        //
        // given that we determined the block fits the current line, if we remove the surrounding
        // `{ .. }` it will still fit the current line, and reduce some noise from the code
        // (this is what rustfmt seems to do too).
        if let GroupKind::LambdaBody {
            block_statement_count: Some(1),
            has_comments: false,
            lambda_has_return_type: false,
        } = group.kind
        {
            self.format_lambda_body_removing_braces(group);
            return;
        }

        self.format_chunk_group_in_one_line(group);
    }

    pub(super) fn format_chunk_group_in_one_line(&mut self, group: ChunkGroup) {
        for chunk in group.chunks {
            match chunk {
                Chunk::Text(text_chunk) | Chunk::Verbatim(text_chunk) => {
                    self.write(&text_chunk.string);
                }
                Chunk::TrailingComment(text_chunk) | Chunk::LeadingComment(text_chunk) => {
                    self.write(&text_chunk.string);
                    self.write_space_without_skipping_whitespace_and_comments();
                }
                Chunk::Group(chunks) => self.format_chunk_group_impl(chunks),
                Chunk::SpaceOrLine => self.write_space_without_skipping_whitespace_and_comments(),
                Chunk::IncreaseIndentation => self.increase_indentation(),
                Chunk::DecreaseIndentation => self.decrease_indentation(),
                Chunk::PushIndentation => self.push_indentation(),
                Chunk::PopIndentation => self.pop_indentation(),
                Chunk::TrailingComma | Chunk::Line { .. } => (),
            }
        }
    }

    pub(super) fn format_chunk_group_in_multiple_lines(&mut self, group: ChunkGroup) {
        let chunks = group.prepare_for_multiple_lines();

        let mut last_was_space_or_line = false;

        for chunk in chunks.chunks {
            if last_was_space_or_line {
                if chunks.one_chunk_per_line {
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.write_indentation();
                } else {
                    // "+ 1" because we still need to add a space before the next chunk
                    if self.current_line_width() + chunk.width() + 1 > self.max_width {
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
                        // after `format_chunks` finishes).
                        // This is the logic to automatically wrap a line when a ChunkGroup doesn't
                        // have Line or SpaceOrLine in it.
                        if self.current_line_width() <= self.max_width
                            && self.current_line_width() + text_chunk.width > self.max_width
                            && !self.buffer.ends_with_space()
                        {
                            self.write_line_without_skipping_whitespace_and_comments();
                            self.increase_indentation();
                            self.write_indentation();
                        }
                        self.write(&text_chunk.string);
                    }
                }
                Chunk::Verbatim(text_chunk) => {
                    self.write(&text_chunk.string);
                }
                Chunk::TrailingComment(text_chunk) => {
                    self.write_chunk_lines(&text_chunk.string);
                    self.write_line_without_skipping_whitespace_and_comments();
                    self.write_indentation();
                }
                Chunk::LeadingComment(text_chunk) => {
                    let ends_with_multiple_newlines = text_chunk.string.ends_with("\n\n");
                    let ends_with_newline =
                        ends_with_multiple_newlines || text_chunk.string.ends_with('\n');
                    self.write_chunk_lines(text_chunk.string.trim());

                    // Respect whether the leading comment had a newline before what comes next or not
                    if ends_with_multiple_newlines {
                        // Remove any indentation that could exist (we'll add it later)
                        self.buffer.trim_spaces();
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if ends_with_newline {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else {
                        self.write_space_without_skipping_whitespace_and_comments();
                    }
                }
                Chunk::Group(mut group) => {
                    if chunks.force_multiline_on_children_with_same_tag_if_multiline
                        && chunks.tag == group.tag
                    {
                        group.force_multiple_lines = true;
                        group.force_multiline_on_children_with_same_tag_if_multiline = true;
                    }

                    self.format_chunk_group_impl(group);
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

    fn format_lambda_body_removing_braces(&mut self, group: ChunkGroup) {
        // Write to an intermediate string so we can remove the braces if needed.
        let text_chunk = self.chunk_formatter().chunk(|formatter| {
            formatter.format_chunk_group_in_one_line(group);
        });
        let string = text_chunk.string;

        // Don't remove the braces if the lambda's body is a Semi expression.
        if string.ends_with("; }") || string.ends_with("; },") {
            self.write(&string);
            return;
        }

        let string = string.strip_prefix("{ ").unwrap();

        // The lambda might have a trailing comma if it's inside an arguments list
        if let Some(string) = string.strip_suffix(" },") {
            self.write(string);
            self.write(",");
        } else {
            let string = string.strip_suffix(" }").unwrap();
            self.write(string);
        }
    }

    /// Appends the string to the current buffer line by line, with some pre-checks.
    fn write_chunk_lines(&mut self, string: &str) {
        let lines: Vec<_> = string.lines().collect();

        let mut index = 0;
        while index < lines.len() {
            let line = &lines[index];

            // Don't indent the first line (it should already be indented).
            // Also don't indent if the current line already has a space as the last char
            // (it means it's already indented)
            if index > 0 && !self.buffer.ends_with_space() {
                self.write_line_without_skipping_whitespace_and_comments();
                // Only indent if the line doesn't start with a space. When that happens
                // it's likely a block comment part that we don't want to modify.
                if !line.starts_with(' ') {
                    self.write_indentation();
                }
            }

            // If we already have a space in the buffer and the line starts with a space,
            // don't repeat that space.
            if self.buffer.ends_with_space() && line.starts_with(' ') {
                self.write(line.trim_start());
            } else {
                self.write(line);
            }

            index += 1;

            // If a newline comes next too, write multiple lines to preserve original formatting
            while index < lines.len() && lines[index].is_empty() {
                self.write_multiple_lines_without_skipping_whitespace_and_comments();
                index += 1;
            }
        }
    }

    /// Returns a new GroupTag that is unique compared to other `new_group_tag` calls.
    pub(super) fn new_group_tag(&mut self) -> GroupTag {
        let tag = GroupTag(self.group_tag_counter);
        self.group_tag_counter += 1;
        tag
    }
}
