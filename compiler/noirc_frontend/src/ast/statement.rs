use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};

use acvm::acir::AcirField;
use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_errors::{Span, Spanned};

use super::{
    BlockExpression, Expression, ExpressionKind, IndexExpression, MemberAccessExpression,
    MethodCallExpression, UnresolvedType,
};
use crate::lexer::token::SpannedToken;
use crate::macros_api::SecondaryAttribute;
use crate::parser::{ParserError, ParserErrorReason};
use crate::token::Token;

/// This is used when an identifier fails to parse in the parser.
/// Instead of failing the parse, we can often recover using this
/// as the default value instead. Further passes like name resolution
/// should also check for this ident to avoid issuing multiple errors
/// for an identifier that already failed to parse.
pub const ERROR_IDENT: &str = "$error";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

/// Ast node for statements in noir. Statements are always within a block { }
/// of some kind and are terminated via a Semicolon, except if the statement
/// ends in a block, such as a Statement::Expression containing an if expression.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StatementKind {
    Let(LetStatement),
    Constrain(ConstrainStatement),
    Expression(Expression),
    Assign(AssignStatement),
    For(ForLoopStatement),
    Break,
    Continue,
    /// This statement should be executed at compile-time
    Comptime(Box<Statement>),
    // This is an expression with a trailing semi-colon
    Semi(Expression),
    // This statement is the result of a recovered parse error.
    // To avoid issuing multiple errors in later steps, it should
    // be skipped in any future analysis if possible.
    Error,
}

impl Statement {
    pub fn add_semicolon(
        mut self,
        semi: Option<Token>,
        span: Span,
        last_statement_in_block: bool,
        emit_error: &mut dyn FnMut(ParserError),
    ) -> Self {
        self.kind = self.kind.add_semicolon(semi, span, last_statement_in_block, emit_error);
        self
    }
}

impl StatementKind {
    pub fn add_semicolon(
        self,
        semi: Option<Token>,
        span: Span,
        last_statement_in_block: bool,
        emit_error: &mut dyn FnMut(ParserError),
    ) -> Self {
        let missing_semicolon =
            ParserError::with_reason(ParserErrorReason::MissingSeparatingSemi, span);

        match self {
            StatementKind::Let(_)
            | StatementKind::Constrain(_)
            | StatementKind::Assign(_)
            | StatementKind::Semi(_)
            | StatementKind::Break
            | StatementKind::Continue
            | StatementKind::Error => {
                // To match rust, statements always require a semicolon, even at the end of a block
                if semi.is_none() {
                    emit_error(missing_semicolon);
                }
                self
            }
            StatementKind::Comptime(mut statement) => {
                *statement =
                    statement.add_semicolon(semi, span, last_statement_in_block, emit_error);
                StatementKind::Comptime(statement)
            }
            // A semicolon on a for loop is optional and does nothing
            StatementKind::For(_) => self,

            StatementKind::Expression(expr) => {
                match (&expr.kind, semi, last_statement_in_block) {
                    // Semicolons are optional for these expressions
                    (ExpressionKind::Block(_), semi, _) | (ExpressionKind::If(_), semi, _) => {
                        if semi.is_some() {
                            StatementKind::Semi(expr)
                        } else {
                            StatementKind::Expression(expr)
                        }
                    }
                    (_, None, false) => {
                        emit_error(missing_semicolon);
                        StatementKind::Expression(expr)
                    }
                    (_, Some(_), _) => StatementKind::Semi(expr),
                    (_, None, true) => StatementKind::Expression(expr),
                }
            }
        }
    }
}

impl Recoverable for StatementKind {
    fn error(_: Span) -> Self {
        StatementKind::Error
    }
}

impl StatementKind {
    pub fn new_let(
        ((pattern, r#type), expression): ((Pattern, UnresolvedType), Expression),
    ) -> StatementKind {
        StatementKind::Let(LetStatement {
            pattern,
            r#type,
            expression,
            comptime: false,
            attributes: vec![],
        })
    }

    /// Create a Statement::Assign value, desugaring any combined operators like += if needed.
    pub fn assign(
        lvalue: LValue,
        operator: Token,
        mut expression: Expression,
        span: Span,
    ) -> StatementKind {
        // Desugar `a <op>= b` to `a = a <op> b`. This relies on the evaluation of `a` having no side effects,
        // which is currently enforced by the restricted syntax of LValues.
        if operator != Token::Assign {
            let lvalue_expr = lvalue.as_expression();
            let error_msg = "Token passed to Statement::assign is not a binary operator";

            let infix = crate::ast::InfixExpression {
                lhs: lvalue_expr,
                operator: operator.try_into_binary_op(span).expect(error_msg),
                rhs: expression,
            };
            expression = Expression::new(ExpressionKind::Infix(Box::new(infix)), span);
        }

        StatementKind::Assign(AssignStatement { lvalue, expression })
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
            kind: ExpressionKind::Variable(
                Path { span: i.span(), segments: vec![i], kind: PathKind::Plain },
                None,
            ),
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
pub struct ModuleDeclaration {
    pub ident: Ident,
}

impl std::fmt::Display for ModuleDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mod {}", self.ident)
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
    pub span: Span,
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
        Path { span: name.span(), segments: vec![name], kind: PathKind::Plain }
    }

    pub fn span(&self) -> Span {
        self.span
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
    pub attributes: Vec<SecondaryAttribute>,

    // True if this should only be run during compile-time
    pub comptime: bool,
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
    MemberAccess { object: Box<LValue>, field_name: Ident, span: Span },
    Index { array: Box<LValue>, index: Expression, span: Span },
    Dereference(Box<LValue>, Span),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainStatement(pub Expression, pub Option<Expression>, pub ConstrainKind);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConstrainKind {
    Assert,
    AssertEq,
    Constrain,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern {
    Identifier(Ident),
    Mutable(Box<Pattern>, Span, /*is_synthesized*/ bool),
    Tuple(Vec<Pattern>, Span),
    Struct(Path, Vec<(Ident, Pattern)>, Span),
}

impl Pattern {
    pub fn is_synthesized(&self) -> bool {
        matches!(self, Pattern::Mutable(_, _, true))
    }

    pub fn span(&self) -> Span {
        match self {
            Pattern::Identifier(ident) => ident.span(),
            Pattern::Mutable(_, span, _)
            | Pattern::Tuple(_, span)
            | Pattern::Struct(_, _, span) => *span,
        }
    }
    pub fn name_ident(&self) -> &Ident {
        match self {
            Pattern::Identifier(name_ident) => name_ident,
            Pattern::Mutable(pattern, ..) => pattern.name_ident(),
            _ => panic!("Only the Identifier or Mutable patterns can return a name"),
        }
    }

    pub(crate) fn into_ident(self) -> Ident {
        match self {
            Pattern::Identifier(ident) => ident,
            Pattern::Mutable(pattern, _, _) => pattern.into_ident(),
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
    fn as_expression(&self) -> Expression {
        let kind = match self {
            LValue::Ident(ident) => ExpressionKind::Variable(Path::from_ident(ident.clone()), None),
            LValue::MemberAccess { object, field_name, span: _ } => {
                ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                    lhs: object.as_expression(),
                    rhs: field_name.clone(),
                }))
            }
            LValue::Index { array, index, span: _ } => {
                ExpressionKind::Index(Box::new(IndexExpression {
                    collection: array.as_expression(),
                    index: index.clone(),
                }))
            }
            LValue::Dereference(lvalue, _span) => {
                ExpressionKind::Prefix(Box::new(crate::ast::PrefixExpression {
                    operator: crate::ast::UnaryOp::Dereference { implicitly_added: false },
                    rhs: lvalue.as_expression(),
                }))
            }
        };
        let span = self.span();
        Expression::new(kind, span)
    }

    pub fn span(&self) -> Span {
        match self {
            LValue::Ident(ident) => ident.span(),
            LValue::MemberAccess { span, .. }
            | LValue::Index { span, .. }
            | LValue::Dereference(_, span) => *span,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ForRange {
    Range(/*start:*/ Expression, /*end:*/ Expression),
    Array(Expression),
}

impl ForRange {
    /// Create a 'for' expression taking care of desugaring a 'for e in array' loop
    /// into the following if needed:
    ///
    /// {
    ///     let fresh1 = array;
    ///     for fresh2 in 0 .. std::array::len(fresh1) {
    ///         let elem = fresh1[fresh2];
    ///         ...
    ///     }
    /// }
    pub(crate) fn into_for(
        self,
        identifier: Ident,
        block: Expression,
        for_loop_span: Span,
    ) -> Statement {
        /// Counter used to generate unique names when desugaring
        /// code in the parser requires the creation of fresh variables.
        /// The parser is stateless so this is a static global instead.
        static UNIQUE_NAME_COUNTER: AtomicU32 = AtomicU32::new(0);

        match self {
            ForRange::Range(..) => {
                unreachable!()
            }
            ForRange::Array(array) => {
                let array_span = array.span;
                let start_range = ExpressionKind::integer(FieldElement::zero());
                let start_range = Expression::new(start_range, array_span);

                let next_unique_id = UNIQUE_NAME_COUNTER.fetch_add(1, Ordering::Relaxed);
                let array_name = format!("$i{next_unique_id}");
                let array_span = array.span;
                let array_ident = Ident::new(array_name, array_span);

                // let fresh1 = array;
                let let_array = Statement {
                    kind: StatementKind::Let(LetStatement {
                        pattern: Pattern::Identifier(array_ident.clone()),
                        r#type: UnresolvedType::unspecified(),
                        expression: array,
                        comptime: false,
                        attributes: vec![],
                    }),
                    span: array_span,
                };

                // array.len()
                let segments = vec![array_ident];
                let array_ident = ExpressionKind::Variable(
                    Path { segments, kind: PathKind::Plain, span: array_span },
                    None,
                );

                let end_range = ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                    object: Expression::new(array_ident.clone(), array_span),
                    method_name: Ident::new("len".to_string(), array_span),
                    generics: None,
                    arguments: vec![],
                }));
                let end_range = Expression::new(end_range, array_span);

                let next_unique_id = UNIQUE_NAME_COUNTER.fetch_add(1, Ordering::Relaxed);
                let index_name = format!("$i{next_unique_id}");
                let fresh_identifier = Ident::new(index_name.clone(), array_span);

                // array[i]
                let segments = vec![Ident::new(index_name, array_span)];
                let index_ident = ExpressionKind::Variable(
                    Path { segments, kind: PathKind::Plain, span: array_span },
                    None,
                );

                let loop_element = ExpressionKind::Index(Box::new(IndexExpression {
                    collection: Expression::new(array_ident, array_span),
                    index: Expression::new(index_ident, array_span),
                }));

                // let elem = array[i];
                let let_elem = Statement {
                    kind: StatementKind::Let(LetStatement {
                        pattern: Pattern::Identifier(identifier),
                        r#type: UnresolvedType::unspecified(),
                        expression: Expression::new(loop_element, array_span),
                        comptime: false,
                        attributes: vec![],
                    }),
                    span: array_span,
                };

                let block_span = block.span;
                let new_block = BlockExpression {
                    statements: vec![
                        let_elem,
                        Statement { kind: StatementKind::Expression(block), span: block_span },
                    ],
                };
                let new_block = Expression::new(ExpressionKind::Block(new_block), block_span);
                let for_loop = Statement {
                    kind: StatementKind::For(ForLoopStatement {
                        identifier: fresh_identifier,
                        range: ForRange::Range(start_range, end_range),
                        block: new_block,
                        span: for_loop_span,
                    }),
                    span: for_loop_span,
                };

                let block = ExpressionKind::Block(BlockExpression {
                    statements: vec![let_array, for_loop],
                });
                let kind = StatementKind::Expression(Expression::new(block, for_loop_span));
                Statement { kind, span: for_loop_span }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ForLoopStatement {
    pub identifier: Ident,
    pub range: ForRange,
    pub block: Expression,
    pub span: Span,
}

impl Display for StatementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatementKind::Let(let_statement) => let_statement.fmt(f),
            StatementKind::Constrain(constrain) => constrain.fmt(f),
            StatementKind::Expression(expression) => expression.fmt(f),
            StatementKind::Assign(assign) => assign.fmt(f),
            StatementKind::For(for_loop) => for_loop.fmt(f),
            StatementKind::Break => write!(f, "break"),
            StatementKind::Continue => write!(f, "continue"),
            StatementKind::Comptime(statement) => write!(f, "comptime {}", statement.kind),
            StatementKind::Semi(semi) => write!(f, "{semi};"),
            StatementKind::Error => write!(f, "Error"),
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
            LValue::MemberAccess { object, field_name, span: _ } => {
                write!(f, "{object}.{field_name}")
            }
            LValue::Index { array, index, span: _ } => write!(f, "{array}[{index}]"),
            LValue::Dereference(lvalue, _span) => write!(f, "*{lvalue}"),
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
            Pattern::Mutable(name, _, _) => write!(f, "mut {name}"),
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

impl Display for ForLoopStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let range = match &self.range {
            ForRange::Range(start, end) => format!("{start}..{end}"),
            ForRange::Array(expr) => expr.to_string(),
        };

        write!(f, "for {} in {range} {}", self.identifier, self.block)
    }
}
