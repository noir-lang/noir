use crate::visitor::FmtVisitor;
use noirc_frontend::hir::resolution::errors::Span;
use noirc_frontend::lexer::Lexer;
use noirc_frontend::token::Token;
use noirc_frontend::{Expression, Ident};

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

pub(crate) struct Exprs<'me, T> {
    pub(crate) visitor: &'me FmtVisitor<'me>,
    pub(crate) elements: std::iter::Peekable<std::vec::IntoIter<T>>,
    pub(crate) last_position: u32,
    pub(crate) end_position: u32,
}

impl<'me, T: Item> Exprs<'me, T> {
    pub(crate) fn new(visitor: &'me FmtVisitor<'me>, span: Span, elements: Vec<T>) -> Self {
        Self {
            visitor,
            last_position: span.start() + 1, /*(*/
            end_position: span.end() - 1,    /*)*/
            elements: elements.into_iter().peekable(),
        }
    }
}

impl<T: Item> Iterator for Exprs<'_, T> {
    type Item = Expr;

    fn next(&mut self) -> Option<Self::Item> {
        let element = self.elements.next()?;
        let element_span = element.span();

        let start = self.last_position;
        let end = element_span.start();

        let is_last = self.elements.peek().is_none();
        let next_start = self.elements.peek().map_or(self.end_position, |expr| expr.start());

        let (leading, different_line) = self.leading(start, end);
        let expr = element.format(self.visitor);
        let trailing = self.trailing(element_span.end(), next_start, is_last);

        Expr { leading, value: expr, trailing, different_line }.into()
    }
}

impl<'me, T> Exprs<'me, T> {
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

pub(crate) trait FindToken {
    fn find_token(&self, token: Token) -> Option<Span>;
    fn find_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<u32>;
}

impl FindToken for str {
    fn find_token(&self, token: Token) -> Option<Span> {
        Lexer::new(self).flatten().find_map(|it| (it.token() == &token).then(|| it.to_span()))
    }

    fn find_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<u32> {
        Lexer::new(self)
            .skip_comments(false)
            .flatten()
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
    if let Some(separator_index) =
        slice.find_token(Token::Comma).map(|index| index.start() as usize)
    {
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

pub(crate) fn count_newlines(slice: &str) -> usize {
    bytecount::count(slice.as_bytes(), b'\n')
}

pub(crate) trait Item {
    fn span(&self) -> Span;

    fn format(self, visitor: &FmtVisitor) -> String;

    fn start(&self) -> u32 {
        self.span().start()
    }

    fn end(&self) -> u32 {
        self.span().end()
    }
}

impl Item for Expression {
    fn span(&self) -> Span {
        self.span
    }

    fn format(self, visitor: &FmtVisitor) -> String {
        visitor.format_sub_expr(self)
    }
}

impl Item for (Ident, Expression) {
    fn span(&self) -> Span {
        let (name, value) = self;
        (name.span().start()..value.span.end()).into()
    }

    fn format(self, visitor: &FmtVisitor) -> String {
        let (name, expr) = self;

        let name = name.0.contents;
        let expr = visitor.format_sub_expr(expr);

        if name == expr {
            name
        } else {
            format!("{name}: {expr}")
        }
    }
}

pub(crate) fn first_line_width(exprs: &str) -> usize {
    exprs.lines().next().map_or(0, |line: &str| line.chars().count())
}

pub(crate) fn is_single_line(s: &str) -> bool {
    !s.chars().any(|c| c == '\n')
}

pub(crate) fn last_line_contains_single_line_comment(s: &str) -> bool {
    s.lines().last().map_or(false, |line| line.contains("//"))
}
