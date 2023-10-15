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
        if let Token::LineComment(content) | Token::BlockComment(content) = spanned.into_token() {
            Some(content)
        } else {
            None
        }
    })
}

#[derive(Debug)]
pub(crate) struct Expr {
    pub(crate) leading: String,
    pub(crate) expr: String,
    pub(crate) trailing: String,
    pub(crate) newlines: bool,
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

        let next_start = self.elements.peek().map_or(self.end_position, |expr| expr.span.start());

        let (leading, newlines) = self.leading(start, end);
        let expr = self.visitor.format_expr(element);
        let trailing = self.trailing(element_span.end(), next_start);

        Expr { leading, expr, trailing, newlines }.into()
    }
}

impl<'me> Exprs<'me> {
    pub(crate) fn leading(&mut self, start: u32, end: u32) -> (String, bool) {
        let mut newlines = false;

        let leading = slice!(self.visitor, start, end);
        let leading_trimmed = slice!(self.visitor, start, end).trim();

        let starts_with_block_comment = leading_trimmed.starts_with("/*");
        let ends_with_block_comment = leading_trimmed.ends_with("*/");
        let starts_with_single_line_comment = leading_trimmed.starts_with("//");

        if ends_with_block_comment {
            let comment_end = leading_trimmed.rfind(|c| c == '/').unwrap();

            if leading[comment_end..].contains('\n') {
                newlines = true;
            }
        } else if starts_with_single_line_comment || starts_with_block_comment {
            newlines = true;
        };

        (leading_trimmed.to_string(), newlines)
    }

    pub(crate) fn trailing(&mut self, start: u32, end: u32) -> String {
        let trailing = slice!(self.visitor, start, end);
        let end = trailing
            .rfind_token_with(|token| {
                matches!(token, Token::LineComment(_) | Token::BlockComment(_))
            })
            .unwrap_or(trailing.len() as u32);

        let trailing = &trailing[..end as usize].trim_matches(',').trim();
        self.last_position = start + end;

        trailing.to_string()
    }
}

pub(crate) trait FindToken {
    fn find_token(&self, token: Token) -> Option<u32>;
    fn rfind_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<u32>;
}

impl FindToken for str {
    fn find_token(&self, token: Token) -> Option<u32> {
        Lexer::new(self).flatten().find_map(|it| (it.token() == &token).then(|| it.to_span().end()))
    }

    fn rfind_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<u32> {
        let mut tokens: Vec<_> = Lexer::new(self).flatten().collect();
        tokens.reverse();

        tokens.into_iter().find_map(|spanned| (f(spanned.token()).then(|| spanned.to_span().end())))
    }
}
