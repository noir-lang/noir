use noirc_frontend::macros_api::Span;

use crate::{
    utils::{comment_len, find_comment_end},
    visitor::{FmtVisitor, Shape},
};

#[derive(Debug)]
pub(crate) struct Item {
    pub(crate) leading: String,
    pub(crate) value: String,
    pub(crate) trailing: String,
    pub(crate) different_line: bool,
}

impl Item {
    pub(crate) fn total_width(&self) -> usize {
        comment_len(&self.leading) + self.value.chars().count() + comment_len(&self.trailing)
    }

    pub(crate) fn is_multiline(&self) -> bool {
        self.leading.contains('\n') || self.trailing.contains('\n')
    }
}

pub(crate) struct Items<'me, T> {
    visitor: &'me FmtVisitor<'me>,
    shape: Shape,
    elements: std::iter::Peekable<std::vec::IntoIter<T>>,
    last_position: u32,
    end_position: u32,
}

impl<'me, T: HasItem> Items<'me, T> {
    pub(crate) fn new(
        visitor: &'me FmtVisitor<'me>,
        shape: Shape,
        span: Span,
        elements: Vec<T>,
    ) -> Self {
        Self {
            visitor,
            shape,
            last_position: span.start() + 1,
            end_position: span.end() - 1,
            elements: elements.into_iter().peekable(),
        }
    }
}

impl<T: HasItem> Iterator for Items<'_, T> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        let element = self.elements.next()?;
        let element_span = element.span();

        let start = self.last_position;
        let end = element_span.start();

        let is_last = self.elements.peek().is_none();
        let next_start = self.elements.peek().map_or(self.end_position, |expr| expr.start());

        let (leading, different_line) = self.leading(start, end);
        let expr = element.format(self.visitor, self.shape);
        let trailing = self.trailing(element_span.end(), next_start, is_last);

        Item { leading, value: expr, trailing, different_line }.into()
    }
}

impl<'me, T> Items<'me, T> {
    pub(crate) fn leading(&mut self, start: u32, end: u32) -> (String, bool) {
        let mut different_line = false;

        let leading = self.visitor.slice(start..end);
        let leading_trimmed = leading.trim();

        let starts_with_block_comment = leading_trimmed.starts_with("/*");
        let ends_with_block_comment = leading_trimmed.ends_with("*/");
        let starts_with_single_line_comment = leading_trimmed.starts_with("//");

        if ends_with_block_comment {
            let comment_end = leading_trimmed.rfind(|c| c == '/').unwrap();

            if leading[comment_end..].contains('\n') {
                different_line = true;
            }
        } else if starts_with_single_line_comment || starts_with_block_comment {
            different_line = true;
        };

        (leading_trimmed.to_string(), different_line)
    }

    pub(crate) fn trailing(&mut self, start: u32, end: u32, is_last: bool) -> String {
        let slice = self.visitor.slice(start..end);
        let comment_end = find_comment_end(slice, is_last);
        let trailing = slice[..comment_end].trim_matches(',').trim();
        self.last_position = start + (comment_end as u32);
        trailing.to_string()
    }
}

pub(crate) trait HasItem {
    fn span(&self) -> Span;

    fn format(self, visitor: &FmtVisitor, shape: Shape) -> String;

    fn start(&self) -> u32 {
        self.span().start()
    }

    fn end(&self) -> u32 {
        self.span().end()
    }
}
