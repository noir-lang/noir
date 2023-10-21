use crate::visitor::FmtVisitor;
use noirc_frontend::hir::resolution::errors::Span;
use noirc_frontend::lexer::Lexer;
use noirc_frontend::token::Token;
use noirc_frontend::Expression;

pub(crate) fn recover_comment_removed(original: &str, new: String) -> String {
    if changed_comment_content(original, &new) {
        original.to_string()
    } else {
        new
    }
}

pub(crate) fn changed_comment_content(original: &str, new: &str) -> bool {
    comments(original).ne(comments(new))
}

pub(crate) fn comments(source: &str) -> impl Iterator<Item = String> + '_ {
    Lexer::new(source).skip_comments(false).flatten().filter_map(|spanned| {
        if let Token::LineComment(content, _) | Token::BlockComment(content, _) =
            spanned.into_token()
        {
            Some(content)
        } else {
            None
        }
    })
}

#[derive(Debug)]
pub(crate) struct Expr {
    pub(crate) leading: String,
    pub(crate) value: String,
    pub(crate) trailing: String,
    pub(crate) different_line: bool,
}

impl Expr {
    pub(crate) fn total_width(&self) -> usize {
        comment_len(&self.leading) + self.value.chars().count() + comment_len(&self.trailing)
    }

    pub(crate) fn is_multiline(&self) -> bool {
        self.leading.contains('\n') || self.trailing.contains('\n')
    }
}

pub(crate) struct Exprs<'me> {
    pub(crate) visitor: &'me FmtVisitor<'me>,
    pub(crate) elements: std::iter::Peekable<std::vec::IntoIter<Expression>>,
    pub(crate) last_position: u32,
    pub(crate) end_position: u32,
}

impl<'me> Exprs<'me> {
    pub(crate) fn new(
        visitor: &'me FmtVisitor<'me>,
        span: Span,
        elements: Vec<Expression>,
    ) -> Self {
        Self {
            visitor,
            last_position: span.start() + 1, /*(*/
            end_position: span.end() - 1,    /*)*/
            elements: elements.into_iter().peekable(),
        }
    }
}

impl Iterator for Exprs<'_> {
    type Item = Expr;

    fn next(&mut self) -> Option<Self::Item> {
        let element = self.elements.next()?;
        let element_span = element.span;

        let start = self.last_position;
        let end = element_span.start();

        let is_last = self.elements.peek().is_none();
        let next_start = self.elements.peek().map_or(self.end_position, |expr| expr.span.start());

        let (leading, different_line) = self.leading(start, end);
        let expr = self.visitor.format_expr(element);
        let trailing = self.trailing(element_span.end(), next_start, is_last);

        Expr { leading, value: expr, trailing, different_line }.into()
    }
}

impl<'me> Exprs<'me> {
    pub(crate) fn leading(&mut self, start: u32, end: u32) -> (String, bool) {
        let mut different_line = false;

        let leading = slice!(self.visitor, start, end);
        let leading_trimmed = slice!(self.visitor, start, end).trim();

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
        let slice = slice!(self.visitor, start, end);
        let comment_end = find_comment_end(slice, is_last);
        let trailing = slice[..comment_end].trim_matches(',').trim();
        self.last_position = start + (comment_end as u32);
        trailing.to_string()
    }
}

pub(crate) trait FindToken {
    fn find_token(&self, token: Token) -> Option<u32>;
    fn find_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<u32>;
}

impl FindToken for str {
    fn find_token(&self, token: Token) -> Option<u32> {
        Lexer::new(self)
            .flatten()
            .find_map(|it| (it.token() == &token).then(|| it.to_span().start()))
    }

    fn find_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<u32> {
        Lexer::new(self)
            .skip_comments(false)
            .flatten()
            .into_iter()
            .find_map(|spanned| f(spanned.token()).then(|| spanned.to_span().end()))
    }
}

pub(crate) fn find_comment_end(slice: &str, is_last: bool) -> usize {
    fn find_comment_end(slice: &str) -> usize {
        slice
            .find_token_with(|token| {
                matches!(token, Token::LineComment(_, _) | Token::BlockComment(_, _))
            })
            .map(|index| index as usize)
            .unwrap_or(slice.len())
    }

    if is_last {
        return slice.len();
    }

    let mut block_open_index = slice.find("/*");
    if let Some(index) = block_open_index {
        match slice.find('/') {
            Some(slash) if slash < index => block_open_index = None,
            _ if slice[..index].ends_with('/') => block_open_index = None,
            _ => (),
        }
    }

    let newline_index = slice.find('\n');
    if let Some(separator_index) = slice.find_token(Token::Comma).map(|index| index as usize) {
        match (block_open_index, newline_index) {
            (Some(block), None) if block > separator_index => separator_index + 1,
            (Some(block), None) => {
                let slice = &slice[block..];
                std::cmp::max(find_comment_end(slice) + block, separator_index + 1)
            }
            (Some(block), Some(newline)) if block < newline => {
                let slice = &slice[block..];
                std::cmp::max(find_comment_end(slice) + block, separator_index + 1)
            }
            (_, Some(newline)) if newline > separator_index => newline + 1,
            _ => slice.len(),
        }
    } else if let Some(newline_index) = newline_index {
        newline_index + 1
    } else {
        0
    }
}

fn comment_len(comment: &str) -> usize {
    match comment {
        "" => 0,
        _ => {
            let len = comment.trim().len();
            if len > 0 {
                len + 6
            } else {
                len
            }
        }
    }
}
