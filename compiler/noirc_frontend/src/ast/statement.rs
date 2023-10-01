use std::fmt::Display;

use crate::lexer::token::SpannedToken;
use crate::parser::{ParserError, ParserErrorReason};
use crate::token::Token;
use crate::{Expression, ExpressionKind, IndexExpression, MemberAccessExpression, UnresolvedType};
use iter_extended::vecmap;
use noirc_errors::{Span, Spanned};

/// This is used when an identifier fails to parse in the parser.
/// Instead of failing the parse, we can often recover using this
/// as the default value instead. Further passes like name resolution
/// should also check for this ident to avoid issuing multiple errors
/// for an identifier that already failed to parse.
pub const ERROR_IDENT: &str = "$error";

/// Ast node for statements in noir. Statements are always within a block { }
/// of some kind and are terminated via a Semicolon, except if the statement
/// ends in a block, such as a Statement::Expression containing an if expression.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    Let(LetStatement),
    Constrain(ConstrainStatement),
    Expression(Expression),
    Assign(AssignStatement),
    // This is an expression with a trailing semi-colon
    Semi(Expression),
    // This statement is the result of a recovered parse error.
    // To avoid issuing multiple errors in later steps, it should
    // be skipped in any future analysis if possible.
    Error,
}

impl Recoverable for Statement {
    fn error(_: Span) -> Self {
        Statement::Error
    }
}

impl Statement {
    pub fn new_let(
        ((pattern, r#type), expression): ((Pattern, UnresolvedType), Expression),
    ) -> Statement {
        Statement::Let(LetStatement { pattern, r#type, expression })
    }

    pub fn add_semicolon(
        self,
        semi: Option<Token>,
        span: Span,
        last_statement_in_block: bool,
        emit_error: &mut dyn FnMut(ParserError),
    ) -> Statement {
        let missing_semicolon =
            ParserError::with_reason(ParserErrorReason::MissingSeparatingSemi, span);
        match self {
            Statement::Let(_)
            | Statement::Constrain(_)
            | Statement::Assign(_)
            | Statement::Semi(_)
            | Statement::Error => {
                // To match rust, statements always require a semicolon, even at the end of a block
                if semi.is_none() {
                    emit_error(missing_semicolon);
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
                    (_, None, false) => {
                        emit_error(missing_semicolon);
                        Statement::Expression(expr)
                    }
                    (_, Some(_), _) => Statement::Semi(expr),
                    (_, None, true) => Statement::Expression(expr),
                }
            }
        }
    }

    /// Create a Statement::Assign value, desugaring any combined operators like += if needed.
    pub fn assign(
        lvalue: LValue,
        operator: Token,
        mut expression: Expression,
        span: Span,
    ) -> Statement {
        // Desugar `a <op>= b` to `a = a <op> b`. This relies on the evaluation of `a` having no side effects,
        // which is currently enforced by the restricted syntax of LValues.
        if operator != Token::Assign {
            let lvalue_expr = lvalue.as_expression(span);
            let error_msg = "Token passed to Statement::assign is not a binary operator";

            let infix = crate::InfixExpression {
                lhs: lvalue_expr,
                operator: operator.try_into_binary_op(span).expect(error_msg),
                rhs: expression,
            };
            expression = Expression::new(ExpressionKind::Infix(Box::new(infix)), span);
        }

        Statement::Assign(AssignStatement { lvalue, expression })
    }
}

#[derive(Eq, Debug, Clone)]
pub struct Ident(pub Spanned<String>);

impl PartialEq<Ident> for Ident {
    fn eq(&self, other: &Ident) -> bool {
        self.0.contents == other.0.contents
    }
}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ident {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.contents.cmp(&other.0.contents)
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.0.contents == other
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
            kind: ExpressionKind::Variable(Path { segments: vec![i], kind: PathKind::Plain }),
        }
    }
}

impl Ident {
    pub fn span(&self) -> Span {
        self.0.span()
    }

    pub fn from_token(token: Token, span: Span) -> Ident {
        Ident::from(SpannedToken::new(token, span))
    }

    pub fn new(text: String, span: Span) -> Ident {
        Ident(Spanned::from(span, text))
    }
}

impl Recoverable for Ident {
    fn error(span: Span) -> Self {
        Ident(Spanned::from(span, ERROR_IDENT.to_owned()))
    }
}

impl<T> Recoverable for Vec<T> {
    fn error(_: Span) -> Self {
        vec![]
    }
}

/// Trait for recoverable nodes during parsing.
/// This is similar to Default but is expected
/// to return an Error node of the appropriate type.
pub trait Recoverable {
    fn error(span: Span) -> Self;
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
pub struct UseTree {
    pub prefix: Path,
    pub kind: UseTreeKind,
}

impl Display for UseTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prefix)?;

        match &self.kind {
            UseTreeKind::Path(name, alias) => {
                write!(f, "{name}")?;

                while let Some(alias) = alias {
                    write!(f, " as {alias}")?;
                }

                Ok(())
            }
            UseTreeKind::List(trees) => {
                write!(f, "::{{")?;
                let tree = vecmap(trees, ToString::to_string).join(", ");
                write!(f, "{tree}}}")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UseTreeKind {
    Path(Ident, Option<Ident>),
    List(Vec<UseTree>),
}

impl UseTree {
    pub fn desugar(self, root: Option<Path>) -> Vec<ImportStatement> {
        let prefix = if let Some(mut root) = root {
            root.segments.extend(self.prefix.segments);
            root
        } else {
            self.prefix
        };

        match self.kind {
            UseTreeKind::Path(name, alias) => {
                vec![ImportStatement { path: prefix.join(name), alias }]
            }
            UseTreeKind::List(trees) => {
                trees.into_iter().flat_map(|tree| tree.desugar(Some(prefix.clone()))).collect()
            }
        }
    }
}

// Note: Path deliberately doesn't implement Recoverable.
// No matter which default value we could give in Recoverable::error,
// it would most likely cause further errors during name resolution
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path {
    pub segments: Vec<Ident>,
    pub kind: PathKind,
}

impl Path {
    pub fn pop(&mut self) -> Ident {
        self.segments.pop().unwrap()
    }

    fn join(mut self, ident: Ident) -> Path {
        self.segments.push(ident);
        self
    }

    /// Construct a PathKind::Plain from this single
    pub fn from_single(name: String, span: Span) -> Path {
        let segment = Ident::from(Spanned::from(span, name));
        Path::from_ident(segment)
    }

    pub fn from_ident(name: Ident) -> Path {
        Path { segments: vec![name], kind: PathKind::Plain }
    }

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
pub struct LetStatement {
    pub pattern: Pattern,
    pub r#type: UnresolvedType,
    pub expression: Expression,
}

impl LetStatement {
    pub fn new_let(
        ((pattern, r#type), expression): ((Pattern, UnresolvedType), Expression),
    ) -> LetStatement {
        LetStatement { pattern, r#type, expression }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssignStatement {
    pub lvalue: LValue,
    pub expression: Expression,
}

/// Represents an Ast form that can be assigned to
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LValue {
    Ident(Ident),
    MemberAccess { object: Box<LValue>, field_name: Ident },
    Index { array: Box<LValue>, index: Expression },
    Dereference(Box<LValue>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainStatement(pub Expression, pub Option<String>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern {
    Identifier(Ident),
    Mutable(Box<Pattern>, Span),
    Tuple(Vec<Pattern>, Span),
    Struct(Path, Vec<(Ident, Pattern)>, Span),
}

impl Pattern {
    pub fn name_ident(&self) -> &Ident {
        match self {
            Pattern::Identifier(name_ident) => name_ident,
            _ => panic!("only the identifier pattern can return a name"),
        }
    }

    pub(crate) fn into_ident(self) -> Ident {
        match self {
            Pattern::Identifier(ident) => ident,
            Pattern::Mutable(pattern, _) => pattern.into_ident(),
            other => panic!("Pattern::into_ident called on {other} pattern with no identifier"),
        }
    }
}

impl Recoverable for Pattern {
    fn error(span: Span) -> Self {
        Pattern::Identifier(Ident::error(span))
    }
}

impl LValue {
    fn as_expression(&self, span: Span) -> Expression {
        let kind = match self {
            LValue::Ident(ident) => ExpressionKind::Variable(Path::from_ident(ident.clone())),
            LValue::MemberAccess { object, field_name } => {
                ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                    lhs: object.as_expression(span),
                    rhs: field_name.clone(),
                }))
            }
            LValue::Index { array, index } => ExpressionKind::Index(Box::new(IndexExpression {
                collection: array.as_expression(span),
                index: index.clone(),
            })),
            LValue::Dereference(lvalue) => {
                ExpressionKind::Prefix(Box::new(crate::PrefixExpression {
                    operator: crate::UnaryOp::Dereference { implicitly_added: false },
                    rhs: lvalue.as_expression(span),
                }))
            }
        };
        Expression::new(kind, span)
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(let_statement) => let_statement.fmt(f),
            Statement::Constrain(constrain) => constrain.fmt(f),
            Statement::Expression(expression) => expression.fmt(f),
            Statement::Assign(assign) => assign.fmt(f),
            Statement::Semi(semi) => write!(f, "{semi};"),
            Statement::Error => write!(f, "Error"),
        }
    }
}

impl Display for LetStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {}: {} = {}", self.pattern, self.r#type, self.expression)
    }
}

impl Display for ConstrainStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "constrain {}", self.0)
    }
}

impl Display for AssignStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.lvalue, self.expression)
    }
}

impl Display for LValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LValue::Ident(ident) => ident.fmt(f),
            LValue::MemberAccess { object, field_name } => write!(f, "{object}.{field_name}"),
            LValue::Index { array, index } => write!(f, "{array}[{index}]"),
            LValue::Dereference(lvalue) => write!(f, "*{lvalue}"),
        }
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
            write!(f, " as {alias}")?;
        }
        Ok(())
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Identifier(name) => name.fmt(f),
            Pattern::Mutable(name, _) => write!(f, "mut {name}"),
            Pattern::Tuple(fields, _) => {
                let fields = vecmap(fields, ToString::to_string);
                write!(f, "({})", fields.join(", "))
            }
            Pattern::Struct(typename, fields, _) => {
                let fields = vecmap(fields, |(name, pattern)| format!("{name}: {pattern}"));
                write!(f, "{} {{ {} }}", typename, fields.join(", "))
            }
        }
    }
}
