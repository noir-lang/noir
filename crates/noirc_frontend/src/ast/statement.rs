use std::fmt::Display;

use crate::lexer::token::SpannedToken;
use crate::parser::ParserError;
use crate::token::Token;
use crate::util::vecmap;
use crate::{Expression, ExpressionKind, InfixExpression, NoirStruct, Type};
use noirc_errors::{Span, Spanned};

#[derive(PartialOrd, Eq, Ord, Debug, Clone)]
pub struct Ident(pub Spanned<String>);

impl PartialEq<Ident> for Ident {
    fn eq(&self, other: &Ident) -> bool {
        self.0.contents == other.0.contents
    }
}

impl std::hash::Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.contents.hash(state);
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.contents.fmt(f)
    }
}

impl From<Spanned<String>> for Ident {
    fn from(a: Spanned<String>) -> Ident {
        Ident(a)
    }
}

impl From<String> for Ident {
    fn from(a: String) -> Ident {
        Spanned::from_position(Default::default(), Default::default(), a).into()
    }
}
impl From<&str> for Ident {
    fn from(a: &str) -> Ident {
        Ident::from(a.to_owned())
    }
}

impl From<SpannedToken> for Ident {
    fn from(st: SpannedToken) -> Ident {
        let spanned_str = Spanned::from(st.to_span(), st.token().to_string());
        Ident(spanned_str)
    }
}

impl From<Ident> for Expression {
    fn from(i: Ident) -> Expression {
        Expression {
            span: i.0.span(),
            kind: ExpressionKind::Ident(i.0.contents),
        }
    }
}

impl From<Ident> for ExpressionKind {
    fn from(i: Ident) -> ExpressionKind {
        ExpressionKind::Ident(i.0.contents)
    }
}

impl Ident {
    pub fn span(&self) -> Span {
        self.0.span()
    }

    pub fn new(token: Token, span: Span) -> Ident {
        Ident::from(SpannedToken::new(token, span))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    Let(LetStatement),
    Const(ConstStatement),
    Constrain(ConstrainStatement),
    Private(PrivateStatement),
    Expression(Expression),
    Assign(AssignStatement),
    // This is an expression with a trailing semi-colon
    // terminology Taken from rustc
    Semi(Expression),

    // This statement is the result of a recovered parse error.
    // To avoid issuing multiple errors in later steps, it should
    // be skipped in any future analysis if possible.
    Error,
}

impl Statement {
    pub fn new_let(((identifier, r#type), expression): ((Ident, Type), Expression)) -> Statement {
        Statement::Let(LetStatement {
            identifier,
            r#type,
            expression,
        })
    }

    pub fn new_const(((identifier, r#type), expression): ((Ident, Type), Expression)) -> Statement {
        Statement::Const(ConstStatement {
            identifier,
            r#type,
            expression,
        })
    }

    pub fn new_priv(((identifier, r#type), expression): ((Ident, Type), Expression)) -> Statement {
        Statement::Private(PrivateStatement {
            identifier,
            r#type,
            expression,
        })
    }

    pub fn add_semicolon(
        self,
        semi: Option<Token>,
        span: Span,
        last_statement_in_block: bool,
        emit_error: &mut dyn FnMut(ParserError),
    ) -> Statement {
        match self {
            Statement::Let(_)
            | Statement::Const(_)
            | Statement::Constrain(_)
            | Statement::Private(_)
            | Statement::Assign(_)
            | Statement::Semi(_)
            | Statement::Error => {
                // To match rust, statements always require a semicolon, even at the end of a block
                if semi.is_none() {
                    let reason = "Expected a ; after this statement".to_string();
                    emit_error(ParserError::with_reason(reason, span));
                }
                self
            }

            Statement::Expression(expr) => {
                match (&expr.kind, semi, last_statement_in_block) {
                    // Semicolons are optional for these expressions
                    (ExpressionKind::Block(_), semi, _)
                    | (ExpressionKind::For(_), semi, _)
                    | (ExpressionKind::If(_), semi, _) => {
                        if semi.is_some() {
                            Statement::Semi(expr)
                        } else {
                            Statement::Expression(expr)
                        }
                    }

                    // Don't wrap expressions that are not the last expression in
                    // a block in a Semi so that we can report errors in the type checker
                    // for unneeded expressions like { 1 + 2; 3 }
                    (_, Some(_), false) => Statement::Expression(expr),
                    (_, None, false) => {
                        let reason = "Expected a ; after this expression".to_string();
                        emit_error(ParserError::with_reason(reason, span));
                        Statement::Expression(expr)
                    }

                    (_, Some(_), true) => Statement::Semi(expr),
                    (_, None, true) => Statement::Expression(expr),
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportStatement {
    pub path: Path,
    pub alias: Option<Ident>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PathKind {
    Crate,
    Dep,
    Plain,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path {
    pub segments: Vec<Ident>,
    pub kind: PathKind,
}

impl Path {
    pub fn span(&self) -> Span {
        let mut segments = self.segments.iter();
        let first_segment = segments.next().expect("ice : cannot have an empty path");
        let mut span = first_segment.0.span();

        for segment in segments {
            span = span.merge(segment.0.span());
        }
        span
    }

    pub fn last_segment(&self) -> Ident {
        assert!(!self.segments.is_empty());
        self.segments.last().unwrap().clone()
    }

    pub fn is_ident(&self) -> bool {
        self.segments.len() == 1 && self.kind == PathKind::Plain
    }

    pub fn as_ident(&self) -> Option<&Ident> {
        if !self.is_ident() {
            return None;
        }
        self.segments.first()
    }

    pub fn to_ident(&self) -> Option<Ident> {
        if !self.is_ident() {
            return None;
        }
        self.segments.first().cloned()
    }

    pub fn as_string(&self) -> String {
        let mut string = String::new();

        let mut segments = self.segments.iter();
        match segments.next() {
            None => panic!("empty segment"),
            Some(seg) => {
                string.push_str(&seg.0.contents);
            }
        }

        for segment in segments {
            string.push_str("::");
            string.push_str(&segment.0.contents);
        }

        string
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
// This will be used for non primitive data types like Arrays and Structs
pub struct LetStatement {
    pub identifier: Ident,
    pub r#type: Type,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstStatement {
    pub identifier: Ident,
    pub r#type: Type, // This will always be a Literal FieldElement
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrivateStatement {
    pub identifier: Ident,
    pub r#type: Type,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssignStatement {
    pub identifier: Ident,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainStatement(pub InfixExpression);

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(let_statement) => let_statement.fmt(f),
            Statement::Const(const_statement) => const_statement.fmt(f),
            Statement::Constrain(constrain) => constrain.fmt(f),
            Statement::Private(private) => private.fmt(f),
            Statement::Expression(expression) => expression.fmt(f),
            Statement::Assign(assign) => assign.fmt(f),
            Statement::Semi(semi) => write!(f, "{};", semi),
            Statement::Error => write!(f, "Error"),
        }
    }
}

impl Display for LetStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "let {}: {} = {}",
            self.identifier, self.r#type, self.expression
        )
    }
}

impl Display for ConstStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "const {}: {} = {}",
            self.identifier, self.r#type, self.expression
        )
    }
}

impl Display for PrivateStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "priv {}: {} = {}",
            self.identifier, self.r#type, self.expression
        )
    }
}

impl Display for ConstrainStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "constrain {}", self.0)
    }
}

impl Display for AssignStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.identifier, self.expression)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let segments = vecmap(&self.segments, ToString::to_string);
        write!(f, "{}::{}", self.kind, segments.join("::"))
    }
}

impl Display for PathKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathKind::Crate => write!(f, "crate"),
            PathKind::Dep => write!(f, "dep"),
            PathKind::Plain => write!(f, "plain"),
        }
    }
}

impl Display for ImportStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "use {}", self.path)?;
        if let Some(alias) = &self.alias {
            write!(f, " as {}", alias)?;
        }
        Ok(())
    }
}

impl Display for NoirStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "struct {} {{", self.name)?;

        for (name, typ) in self.fields.iter() {
            writeln!(f, "    {}: {},", name, typ)?;
        }

        write!(f, "}}")
    }
}
