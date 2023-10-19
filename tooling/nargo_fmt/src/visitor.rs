/// A macro to create a slice from a given data source, helping to avoid borrow checker errors.
#[macro_export]
macro_rules! slice {
    ($this:expr, $start:expr, $end:expr) => {
        &$this.source[$start as usize..$end as usize]
    };
}

mod expr;
mod item;
mod stmt;

use noirc_frontend::hir::resolution::errors::Span;

use crate::config::Config;

pub(crate) struct FmtVisitor<'me> {
    config: &'me Config,
    buffer: String,
    pub(crate) source: &'me str,
    indent: Indent,
    last_position: u32,
}

impl<'me> FmtVisitor<'me> {
    pub(crate) fn new(source: &'me str, config: &'me Config) -> Self {
        Self {
            buffer: String::new(),
            config,
            source,
            last_position: 0,
            indent: Indent { block_indent: 0 },
        }
    }

    fn shape(&self) -> Shape {
        Shape {
            width: self.config.max_width.saturating_sub(self.indent.width()),
            indent: self.indent,
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            buffer: String::new(),
            config: self.config,
            source: self.source,
            last_position: self.last_position,
            indent: self.indent,
        }
    }

    pub(crate) fn finish(self) -> String {
        self.buffer
    }

    fn with_indent<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.indent.block_indent(self.config);
        let ret = f(self);
        self.indent.block_unindent(self.config);
        ret
    }

    fn at_start(&self) -> bool {
        self.buffer.is_empty()
    }

    fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    #[track_caller]
    fn push_rewrite(&mut self, s: String, span: Span) {
        self.format_missing_indent(span.start(), true);
        self.push_str(&s);
    }

    fn format_missing(&mut self, end: u32) {
        self.format_missing_inner(end, |this, slice, _| this.push_str(slice));
    }

    #[track_caller]
    fn format_missing_indent(&mut self, end: u32, should_indent: bool) {
        self.format_missing_inner(end, |this, last_slice, slice| {
            this.push_str(last_slice.trim_end());

            if last_slice == slice && !this.at_start() {
                this.push_str("\n");
            }

            if should_indent {
                let indent = this.indent.to_string();
                this.push_str(&indent);
            }
        });
    }

    #[track_caller]
    fn format_missing_inner(
        &mut self,
        end: u32,
        process_last_slice: impl Fn(&mut Self, &str, &str),
    ) {
        let start = self.last_position;

        if start == end {
            if !self.at_start() {
                process_last_slice(self, "", "");
            }
            return;
        }

        let slice = slice!(self, start, end);
        self.last_position = end;

        if slice.trim().is_empty() && !self.at_start() {
            self.push_vertical_spaces(slice);
            process_last_slice(self, "", slice);
        } else {
            process_last_slice(self, slice, slice);
        }
    }

    fn push_vertical_spaces(&mut self, slice: &str) {
        let newline_upper_bound = 2;
        let newline_lower_bound = 1;

        let mut newline_count = bytecount::count(slice.as_bytes(), b'\n');
        let offset = self.buffer.chars().rev().take_while(|c| *c == '\n').count();

        if newline_count + offset > newline_upper_bound {
            if offset >= newline_upper_bound {
                newline_count = 0;
            } else {
                newline_count = newline_upper_bound - offset;
            }
        } else if newline_count + offset < newline_lower_bound {
            if offset >= newline_lower_bound {
                newline_count = 0;
            } else {
                newline_count = newline_lower_bound - offset;
            }
        }

        let blank_lines = "\n".repeat(newline_count);
        self.push_str(&blank_lines);
    }

    pub(crate) fn format_comment(&self, span: Span) -> String {
        let slice = slice!(self, span.start(), span.end()).trim();
        let pos = slice.find('/');

        if !slice.is_empty() && pos.is_some() {
            slice.to_string()
        } else {
            String::new()
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Indent {
    block_indent: usize,
}

impl Indent {
    fn width(&self) -> usize {
        self.block_indent
    }

    fn block_indent(&mut self, config: &Config) {
        self.block_indent += config.tab_spaces;
    }

    fn block_unindent(&mut self, config: &Config) {
        self.block_indent -= config.tab_spaces;
    }

    fn to_string_with_newline(self) -> String {
        "\n".to_string() + &self.to_string()
    }

    #[allow(clippy::inherent_to_string)]
    fn to_string(self) -> String {
        " ".repeat(self.block_indent)
    }
}

#[derive(Clone, Copy, Debug)]
struct Shape {
    width: usize,
    indent: Indent,
}
