use std::fmt::Display;

use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    ast::{ExpressionKind, LValue, Pattern, StatementKind, UnresolvedTypeData},
    macros_api::NodeInterner,
    token::{Keyword, Token},
};

use super::{value::TypedExpr, Value};

pub(super) fn print_quoted(
    tokens: &[Token],
    indent: usize,
    interner: &NodeInterner,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    if tokens.is_empty() {
        write!(f, "quote {{ }}")
    } else {
        writeln!(f, "quote {{")?;
        let indent = indent + 1;
        write!(f, "{}", " ".repeat(indent * 4))?;
        let mut token_printer = TokenPrettyPrinter::new(interner, indent);
        for token in tokens.iter() {
            token_printer.print(token, f)?;
        }
        writeln!(f)?;
        let indent = indent - 1;
        write!(f, "{}", " ".repeat(indent * 4))?;
        write!(f, "}}")
    }
}

/// Tries to print tokens in a way that it'll be easier for the user to understand a
/// stream of tokens without having to format it themselves.
///
/// The gist is:
/// - Keep track of the current indent level
/// - When '{' is found, a newline is inserted and the indent is incremented
/// - When '}' is found, a newline is inserted and the indent is decremented
/// - When ';' is found a newline is inserted
/// - When interned values are encountered, they are turned into strings and indented
///   according to the current indentation.
///
/// There are a few more details that needs to be taken into account:
/// - two consecutive words shouldn't be glued together (as they are separate tokens)
/// - inserting spaces when needed
/// - not inserting extra newlines if possible
/// - ';' shouldn't always insert newlines (this is when it's something like `[Field; 2]`)
struct TokenPrettyPrinter<'interner> {
    interner: &'interner NodeInterner,
    indent: usize,
    last_was_name: bool,
    last_was_right_brace: bool,
    last_was_semicolon: bool,
}

impl<'interner> TokenPrettyPrinter<'interner> {
    fn new(interner: &'interner NodeInterner, indent: usize) -> Self {
        Self {
            interner,
            indent,
            last_was_name: false,
            last_was_right_brace: false,
            last_was_semicolon: false,
        }
    }

    fn print(&mut self, token: &Token, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let last_was_name = self.last_was_name;
        self.last_was_name = false;

        // After `}` we usually want a newline... but not always!
        if self.last_was_right_brace {
            self.last_was_right_brace = false;

            match token {
                Token::Keyword(Keyword::Else) => {
                    // If we have `} else` we don't want a newline
                    write!(f, " else")?;
                    self.last_was_name = true;
                    return Ok(());
                }
                Token::RightBrace => {
                    // Because we insert a newline right before `}`, if we have two
                    // (or more) in a row we don't want extra newlines.
                }
                _ => {
                    writeln!(f)?;
                    self.write_indent(f)?;
                }
            }
        }

        // Heuristic: if we have `; 2` then we assume we are inside something like `[Field; 2]`
        // and don't include a newline.
        // The only consequence of getting this wrong is that we'll end with two consecutive
        // statements in a single line (not a big deal).
        if self.last_was_semicolon {
            self.last_was_semicolon = false;

            match token {
                Token::Int(..) => {
                    write!(f, " ")?;
                }
                Token::Ident(ident) => {
                    if ident.chars().all(|char| char.is_ascii_uppercase()) {
                        write!(f, " ")?;
                    } else {
                        writeln!(f)?;
                        self.write_indent(f)?;
                    }
                }
                Token::RightBrace => {
                    // We don't want an extra newline in this case
                }
                _ => {
                    writeln!(f)?;
                    self.write_indent(f)?;
                }
            }
        }

        match token {
            Token::QuotedType(id) => write!(f, "{}", self.interner.get_quoted_type(*id)),
            Token::InternedExpr(id) => {
                let value = Value::expression(ExpressionKind::Interned(*id));
                self.print_value(&value, f)
            }
            Token::InternedStatement(id) => {
                let value = Value::statement(StatementKind::Interned(*id));
                self.print_value(&value, f)
            }
            Token::InternedLValue(id) => {
                let value = Value::lvalue(LValue::Interned(*id, Span::default()));
                self.print_value(&value, f)
            }
            Token::InternedUnresolvedTypeData(id) => {
                let value = Value::UnresolvedType(UnresolvedTypeData::Interned(*id));
                self.print_value(&value, f)
            }
            Token::InternedPattern(id) => {
                let value = Value::pattern(Pattern::Interned(*id, Span::default()));
                self.print_value(&value, f)
            }
            Token::UnquoteMarker(id) => {
                let value = Value::TypedExpr(TypedExpr::ExprId(*id));
                self.print_value(&value, f)
            }
            Token::Keyword(..) | Token::Ident(..) => {
                if last_was_name {
                    write!(f, " ")?;
                }
                self.last_was_name = true;
                write!(f, "{token}")
            }
            Token::Comma => {
                write!(f, "{token} ")
            }
            Token::LeftBrace => {
                writeln!(f, " {{")?;
                self.indent += 1;
                self.write_indent(f)
            }
            Token::RightBrace => {
                self.last_was_right_brace = true;
                writeln!(f)?;
                self.indent -= 1;
                self.write_indent(f)?;
                write!(f, "}}")
            }
            Token::Semicolon => {
                self.last_was_semicolon = true;
                write!(f, ";")
            }
            Token::IntType(..) => {
                if last_was_name {
                    write!(f, " ")?;
                }
                self.last_was_name = true;
                write!(f, "{token}")
            }
            Token::Quote(tokens) => {
                if last_was_name {
                    write!(f, " ")?;
                }
                let tokens = vecmap(&tokens.0, |spanned_token| spanned_token.clone().into_token());
                print_quoted(&tokens, self.indent, self.interner, f)
            }
            Token::Colon => {
                write!(f, "{token} ")
            }
            Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Equal
            | Token::NotEqual
            | Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Ampersand
            | Token::ShiftLeft
            | Token::ShiftRight
            | Token::Assign
            | Token::Arrow => write!(f, " {token} "),
            Token::LeftParen
            | Token::RightParen
            | Token::LeftBracket
            | Token::RightBracket
            | Token::Dot
            | Token::DoubleColon
            | Token::DoubleDot
            | Token::Caret
            | Token::Pound
            | Token::Pipe
            | Token::Bang
            | Token::DollarSign => {
                write!(f, "{token}")
            }
            Token::Int(..)
            | Token::Bool(..)
            | Token::Str(..)
            | Token::RawStr(..)
            | Token::FmtStr(..)
            | Token::Whitespace(_)
            | Token::LineComment(..)
            | Token::BlockComment(..)
            | Token::Attribute(..)
            | Token::InnerAttribute(..)
            | Token::Invalid(_) => {
                if last_was_name {
                    write!(f, " ")?;
                }
                write!(f, "{token}")
            }
            Token::EOF => Ok(()),
        }
    }

    fn print_value(&mut self, value: &Value, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = value.display(self.interner).to_string();
        for (index, line) in string.lines().enumerate() {
            if index > 0 {
                writeln!(f)?;
                self.write_indent(f)?;
            }
            line.fmt(f)?;
        }
        Ok(())
    }

    fn write_indent(&mut self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", " ".repeat(self.indent * 4))
    }
}
