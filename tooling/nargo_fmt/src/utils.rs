use crate::items::HasItem;
use crate::rewrite;
use crate::visitor::{FmtVisitor, Shape};
use noirc_frontend::hir::resolution::errors::Span;
use noirc_frontend::lexer::Lexer;
use noirc_frontend::token::Token;
use noirc_frontend::{Expression, Ident, Param, Visibility};

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

pub(crate) trait FindToken {
    fn find_token(&self, token: Token) -> Option<Span>;
    fn find_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<Span>;
}

impl FindToken for str {
    fn find_token(&self, token: Token) -> Option<Span> {
        Lexer::new(self).flatten().find_map(|it| (it.token() == &token).then(|| it.to_span()))
    }

    fn find_token_with(&self, f: impl Fn(&Token) -> bool) -> Option<Span> {
        Lexer::new(self)
            .skip_comments(false)
            .flatten()
            .find_map(|spanned| f(spanned.token()).then(|| spanned.to_span()))
    }
}

pub(crate) fn find_comment_end(slice: &str, is_last: bool) -> usize {
    fn find_comment_end(slice: &str) -> usize {
        slice
            .find_token_with(|token| {
                matches!(token, Token::LineComment(_, _) | Token::BlockComment(_, _))
            })
            .map(|index| index.end() as usize)
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

pub(crate) fn comment_len(comment: &str) -> usize {
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

impl HasItem for Expression {
    fn span(&self) -> Span {
        self.span
    }

    fn format(self, visitor: &FmtVisitor, shape: Shape) -> String {
        rewrite::sub_expr(visitor, shape, self)
    }
}

impl HasItem for (Ident, Expression) {
    fn span(&self) -> Span {
        let (name, value) = self;
        (name.span().start()..value.span.end()).into()
    }

    fn format(self, visitor: &FmtVisitor, shape: Shape) -> String {
        let (name, expr) = self;

        let name = name.0.contents;
        let expr = rewrite::sub_expr(visitor, shape, expr);

        if name == expr {
            name
        } else {
            format!("{name}: {expr}")
        }
    }
}

impl HasItem for Param {
    fn span(&self) -> Span {
        self.span
    }

    fn format(self, visitor: &FmtVisitor, shape: Shape) -> String {
        let pattern = visitor.slice(self.pattern.span());
        let visibility = match self.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
            Visibility::DataBus => "call_data",
        };

        if self.pattern.is_synthesized() || self.typ.is_synthesized() {
            pattern.to_string()
        } else {
            let ty = rewrite::typ(visitor, shape, self.typ);
            format!("{pattern}: {visibility}{ty}")
        }
    }
}

impl HasItem for Ident {
    fn span(&self) -> Span {
        self.span()
    }

    fn format(self, visitor: &FmtVisitor, _shape: Shape) -> String {
        visitor.slice(self.span()).into()
    }
}

pub(crate) fn first_line_width(exprs: &str) -> usize {
    exprs.lines().next().map_or(0, |line: &str| line.chars().count())
}

pub(crate) fn last_line_width(s: &str) -> usize {
    s.rsplit('\n').next().unwrap_or("").chars().count()
}

pub(crate) fn is_single_line(s: &str) -> bool {
    !s.chars().any(|c| c == '\n')
}

pub(crate) fn last_line_contains_single_line_comment(s: &str) -> bool {
    s.lines().last().map_or(false, |line| line.contains("//"))
}

pub(crate) fn last_line_used_width(s: &str, offset: usize) -> usize {
    if s.contains('\n') {
        last_line_width(s)
    } else {
        offset + s.chars().count()
    }
}

pub(crate) fn span_is_empty(span: Span) -> bool {
    span.start() == span.end()
}
