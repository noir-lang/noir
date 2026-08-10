/// A buffer to write to.
/// It keeps track of the current line width and provides a few useful methods
/// to deal with the buffer contents.
#[derive(Default, Debug)]
pub(crate) struct Buffer {
    buffer: String,

    /// How many characters we've written so far in the current line
    /// (useful to avoid exceeding the configurable maximum)
    current_line_width: usize,

    /// Whether the current line ends inside a `//` line comment that a newline must
    /// terminate before anything else is appended to the line (otherwise whatever
    /// is appended would become part of the comment).
    line_comment_needs_termination: bool,
}

impl Buffer {
    pub(crate) fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub(crate) fn ends_with_newline(&self) -> bool {
        self.buffer.ends_with('\n')
    }

    pub(crate) fn ends_with_double_newline(&self) -> bool {
        self.buffer.ends_with("\n\n")
    }

    pub(crate) fn ends_with_space(&self) -> bool {
        self.buffer.ends_with(' ')
    }

    /// Returns the contents of the current (last) line in the buffer.
    pub(crate) fn current_line(&self) -> &str {
        match self.buffer.rfind('\n') {
            Some(index) => &self.buffer[index + 1..],
            None => &self.buffer,
        }
    }

    pub(crate) fn write(&mut self, str: &str) {
        if str.contains('\n') {
            self.line_comment_needs_termination = false;
        }

        self.buffer.push_str(str);

        if str.ends_with('\n') {
            self.current_line_width = 0;
        } else {
            self.current_line_width += str.chars().count();
        }
    }

    pub(crate) fn line_comment_needs_termination(&self) -> bool {
        self.line_comment_needs_termination
    }

    pub(crate) fn set_line_comment_needs_termination(&mut self) {
        self.line_comment_needs_termination = true;
    }

    /// Trim spaces from the end of the buffer.
    pub(crate) fn trim_spaces(&mut self) {
        while self.buffer.ends_with(' ') {
            self.buffer.truncate(self.buffer.len() - 1);
            self.current_line_width -= 1;
        }
    }

    /// Trim commas from the end of the buffer. Returns true if a comma was trimmed.
    pub(super) fn trim_comma(&mut self) -> bool {
        if self.buffer.ends_with(',') {
            self.buffer.truncate(self.buffer.len() - 1);
            self.current_line_width -= 1;
            true
        } else {
            false
        }
    }

    /// Trim multiple newlines from the end of the buffer, keeping at most one.
    pub(crate) fn trim_multiple_newlines(&mut self) {
        while self.buffer.ends_with("\n\n") {
            self.buffer.truncate(self.buffer.len() - 1);
        }
    }

    pub(crate) fn contents(self) -> String {
        self.buffer
    }

    pub(crate) fn current_line_width(&self) -> usize {
        self.current_line_width
    }
}
