/// A buffer to write to.
/// It keeps track of the current line width and provides a few useful methods
/// to deal with the buffer contents.
#[derive(Default, Debug)]
pub(crate) struct Buffer {
    buffer: String,

    /// How many characters we've written so far in the current line
    /// (useful to avoid exceeding the configurable maximum)
    current_line_width: usize,
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

    pub(crate) fn write(&mut self, str: &str) {
        self.buffer.push_str(str);

        if str.ends_with('\n') {
            self.current_line_width = 0;
        } else {
            self.current_line_width += str.chars().count();
        }
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

    pub(crate) fn contents(self) -> String {
        self.buffer
    }

    pub(crate) fn current_line_width(&self) -> usize {
        self.current_line_width
    }
}
