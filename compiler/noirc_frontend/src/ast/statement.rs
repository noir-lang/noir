use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};

use acvm::acir::AcirField;
use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_errors::{Span, Spanned};

use super::{
    BinaryOpKind, BlockExpression, ConstructorExpression, Expression, ExpressionKind,
    GenericTypeArgs, IndexExpression, InfixExpression, ItemVisibility, MemberAccessExpression,
    MethodCallExpression, UnresolvedType,
};
use crate::ast::UnresolvedTypeData;
use crate::elaborator::types::SELF_TYPE_NAME;
use crate::lexer::token::SpannedToken;
use crate::node_interner::{
    InternedExpressionKind, InternedPattern, InternedStatementKind, NodeInterner,
};
use crate::parser::{ParserError, ParserErrorReason};
use crate::token::{SecondaryAttribute, Token};

/// This is used when an identifier fails to parse in the parser.
/// Instead of failing the parse, we can often recover using this
/// as the default value instead. Further passes like name resolution
/// should also check for this ident to avoid issuing multiple errors
/// for an identifier that already failed to parse.
pub const ERROR_IDENT: &str = "$error";

/// This is used to represent an UnresolvedTypeData::Unspecified in a Path
pub const WILDCARD_TYPE: &str = "_";

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
    // This is an interned StatementKind during comptime code.
    // The actual StatementKind can be retrieved with a NodeInterner.
    Interned(InternedStatementKind),
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

            // No semicolon needed for a resolved statement
            StatementKind::Interned(_) => self,

            StatementKind::Expression(expr) => {
                match (&expr.kind, semi, last_statement_in_block) {
                    // Semicolons are optional for these expressions
                    (ExpressionKind::Block(_), semi, _)
                    | (ExpressionKind::Unsafe(..), semi, _)
                    | (ExpressionKind::Interned(..), semi, _)
                    | (ExpressionKind::InternedStatement(..), semi, _)
                    | (ExpressionKind::If(_), semi, _) => {
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
        pattern: Pattern,
        r#type: UnresolvedType,
        expression: Expression,
        attributes: Vec<SecondaryAttribute>,
    ) -> StatementKind {
        StatementKind::Let(LetStatement {
            pattern,
            r#type,
            expression,
            comptime: false,
            attributes,
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

#[derive(Eq, Debug, Clone, Default)]
pub struct Ident(pub Spanned<String>);

impl Ident {
    pub fn is_self_type_name(&self) -> bool {
        self.0.contents == SELF_TYPE_NAME
    }
}

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
            kind: ExpressionKind::Variable(Path {
                span: i.span(),
                segments: vec![PathSegment::from(i)],
                kind: PathKind::Plain,
            }),
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
    pub visibility: ItemVisibility,
    pub ident: Ident,
    pub outer_attributes: Vec<SecondaryAttribute>,
}

impl std::fmt::Display for ModuleDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mod {}", self.ident)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportStatement {
    pub visibility: ItemVisibility,
    pub path: Path,
    pub alias: Option<Ident>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PathKind {
    Crate,
    Dep,
    Plain,
    Super,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UseTree {
    pub prefix: Path,
    pub kind: UseTreeKind,
    pub span: Span,
}

impl Display for UseTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prefix)?;

        if !self.prefix.segments.is_empty() {
            write!(f, "::")?;
        }

        match &self.kind {
            UseTreeKind::Path(name, alias) => {
                write!(f, "{name}")?;

                if let Some(alias) = alias {
                    write!(f, " as {alias}")?;
                }

                Ok(())
            }
            UseTreeKind::List(trees) => {
                write!(f, "{{")?;
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
    pub fn desugar(self, root: Option<Path>, visibility: ItemVisibility) -> Vec<ImportStatement> {
        let prefix = if let Some(mut root) = root {
            root.segments.extend(self.prefix.segments);
            root
        } else {
            self.prefix
        };

        match self.kind {
            UseTreeKind::Path(name, alias) => {
                // Desugar `use foo::{self}` to `use foo`
                let path = if name.0.contents == "self" { prefix } else { prefix.join(name) };
                vec![ImportStatement { visibility, path, alias }]
            }
            UseTreeKind::List(trees) => {
                let trees = trees.into_iter();
                trees.flat_map(|tree| tree.desugar(Some(prefix.clone()), visibility)).collect()
            }
        }
    }
}

/// A special kind of path in the form `<MyType as Trait>::ident`.
/// Note that this path must consist of exactly two segments.
///
/// An AsTraitPath may be used in either a type context where `ident`
/// refers to an associated type of a particular impl, or in a value
/// context where `ident` may refer to an associated constant or a
/// function within the impl.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct AsTraitPath {
    pub typ: UnresolvedType,
    pub trait_path: Path,
    pub trait_generics: GenericTypeArgs,
    pub impl_item: Ident,
}

/// A special kind of path in the form `Type::ident::<turbofish>`
/// Unlike normal paths, the type here can be a primitive type or
/// interned type.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypePath {
    pub typ: UnresolvedType,
    pub item: Ident,
    pub turbofish: Option<GenericTypeArgs>,
}

// Note: Path deliberately doesn't implement Recoverable.
// No matter which default value we could give in Recoverable::error,
// it would most likely cause further errors during name resolution
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub kind: PathKind,
    pub span: Span,
}

impl Path {
    pub fn pop(&mut self) -> PathSegment {
        self.segments.pop().unwrap()
    }

    fn join(mut self, ident: Ident) -> Path {
        self.segments.push(PathSegment::from(ident));
        self
    }

    /// Construct a PathKind::Plain from this single
    pub fn from_single(name: String, span: Span) -> Path {
        let segment = Ident::from(Spanned::from(span, name));
        Path::from_ident(segment)
    }

    pub fn from_ident(name: Ident) -> Path {
        Path { span: name.span(), segments: vec![PathSegment::from(name)], kind: PathKind::Plain }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn last_segment(&self) -> PathSegment {
        assert!(!self.segments.is_empty());
        self.segments.last().unwrap().clone()
    }

    pub fn last_ident(&self) -> Ident {
        self.last_segment().ident
    }

    pub fn first_name(&self) -> Option<&str> {
        self.segments.first().map(|segment| segment.ident.0.contents.as_str())
    }

    pub fn last_name(&self) -> &str {
        assert!(!self.segments.is_empty());
        &self.segments.last().unwrap().ident.0.contents
    }

    pub fn is_ident(&self) -> bool {
        self.kind == PathKind::Plain
            && self.segments.len() == 1
            && self.segments.first().unwrap().generics.is_none()
    }

    pub fn as_ident(&self) -> Option<&Ident> {
        if !self.is_ident() {
            return None;
        }
        self.segments.first().map(|segment| &segment.ident)
    }

    pub fn to_ident(&self) -> Option<Ident> {
        if !self.is_ident() {
            return None;
        }
        self.segments.first().cloned().map(|segment| segment.ident)
    }

    pub(crate) fn is_wildcard(&self) -> bool {
        self.to_ident().map(|ident| ident.0.contents) == Some(WILDCARD_TYPE.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty() && self.kind == PathKind::Plain
    }

    pub fn as_string(&self) -> String {
        let mut string = String::new();

        let mut segments = self.segments.iter();
        match segments.next() {
            None => panic!("empty segment"),
            Some(seg) => {
                string.push_str(&seg.ident.0.contents);
            }
        }

        for segment in segments {
            string.push_str("::");
            string.push_str(&segment.ident.0.contents);
        }

        string
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PathSegment {
    pub ident: Ident,
    pub generics: Option<Vec<UnresolvedType>>,
    pub span: Span,
}

impl PathSegment {
    /// Returns the span where turbofish happen. For example:
    ///
    ///    foo::<T>
    ///       ~^^^^
    ///
    /// Returns an empty span at the end of `foo` if there's no turbofish.
    pub fn turbofish_span(&self) -> Span {
        Span::from(self.ident.span().end()..self.span.end())
    }
}

impl From<Ident> for PathSegment {
    fn from(ident: Ident) -> PathSegment {
        let span = ident.span();
        PathSegment { ident, generics: None, span }
    }
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ident.fmt(f)?;

        if let Some(generics) = &self.generics {
            let generics = vecmap(generics, ToString::to_string);
            write!(f, "::<{}>", generics.join(", "))?;
        }

        Ok(())
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
    Interned(InternedExpressionKind, Span),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainStatement {
    pub kind: ConstrainKind,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

impl Display for ConstrainStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ConstrainKind::Assert | ConstrainKind::AssertEq => write!(
                f,
                "{}({})",
                self.kind,
                vecmap(&self.arguments, |arg| arg.to_string()).join(", ")
            ),
            ConstrainKind::Constrain => {
                write!(f, "constrain {}", &self.arguments[0])
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConstrainKind {
    Assert,
    AssertEq,
    Constrain,
}

impl ConstrainKind {
    pub fn required_arguments_count(&self) -> usize {
        match self {
            ConstrainKind::Assert | ConstrainKind::Constrain => 1,
            ConstrainKind::AssertEq => 2,
        }
    }
}

impl Display for ConstrainKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstrainKind::Assert => write!(f, "assert"),
            ConstrainKind::AssertEq => write!(f, "assert_eq"),
            ConstrainKind::Constrain => write!(f, "constrain"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern {
    Identifier(Ident),
    Mutable(Box<Pattern>, Span, /*is_synthesized*/ bool),
    Tuple(Vec<Pattern>, Span),
    Struct(Path, Vec<(Ident, Pattern)>, Span),
    Interned(InternedPattern, Span),
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
            | Pattern::Struct(_, _, span)
            | Pattern::Interned(_, span) => *span,
        }
    }
    pub fn name_ident(&self) -> &Ident {
        match self {
            Pattern::Identifier(name_ident) => name_ident,
            Pattern::Mutable(pattern, ..) => pattern.name_ident(),
            _ => panic!("Only the Identifier or Mutable patterns can return a name"),
        }
    }

    pub(crate) fn try_as_expression(&self, interner: &NodeInterner) -> Option<Expression> {
        match self {
            Pattern::Identifier(ident) => Some(Expression {
                kind: ExpressionKind::Variable(Path::from_ident(ident.clone())),
                span: ident.span(),
            }),
            Pattern::Mutable(_, _, _) => None,
            Pattern::Tuple(patterns, span) => {
                let mut expressions = Vec::new();
                for pattern in patterns {
                    expressions.push(pattern.try_as_expression(interner)?);
                }
                Some(Expression { kind: ExpressionKind::Tuple(expressions), span: *span })
            }
            Pattern::Struct(path, patterns, span) => {
                let mut fields = Vec::new();
                for (field, pattern) in patterns {
                    let expression = pattern.try_as_expression(interner)?;
                    fields.push((field.clone(), expression));
                }
                Some(Expression {
                    kind: ExpressionKind::Constructor(Box::new(ConstructorExpression {
                        typ: UnresolvedType::from_path(path.clone()),
                        fields,
                        struct_type: None,
                    })),
                    span: *span,
                })
            }
            Pattern::Interned(id, _) => interner.get_pattern(*id).try_as_expression(interner),
        }
    }
}

impl Recoverable for Pattern {
    fn error(span: Span) -> Self {
        Pattern::Identifier(Ident::error(span))
    }
}

impl LValue {
    pub fn as_expression(&self) -> Expression {
        let kind = match self {
            LValue::Ident(ident) => ExpressionKind::Variable(Path::from_ident(ident.clone())),
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
            LValue::Interned(id, _) => ExpressionKind::Interned(*id),
        };
        let span = self.span();
        Expression::new(kind, span)
    }

    pub fn from_expression(expr: Expression) -> Option<LValue> {
        LValue::from_expression_kind(expr.kind, expr.span)
    }

    pub fn from_expression_kind(expr: ExpressionKind, span: Span) -> Option<LValue> {
        match expr {
            ExpressionKind::Variable(path) => Some(LValue::Ident(path.as_ident().unwrap().clone())),
            ExpressionKind::MemberAccess(member_access) => Some(LValue::MemberAccess {
                object: Box::new(LValue::from_expression(member_access.lhs)?),
                field_name: member_access.rhs,
                span,
            }),
            ExpressionKind::Index(index) => Some(LValue::Index {
                array: Box::new(LValue::from_expression(index.collection)?),
                index: index.index,
                span,
            }),
            ExpressionKind::Prefix(prefix) => {
                if matches!(
                    prefix.operator,
                    crate::ast::UnaryOp::Dereference { implicitly_added: false }
                ) {
                    Some(LValue::Dereference(Box::new(LValue::from_expression(prefix.rhs)?), span))
                } else {
                    None
                }
            }
            ExpressionKind::Parenthesized(expr) => LValue::from_expression(*expr),
            ExpressionKind::Interned(id) => Some(LValue::Interned(id, span)),
            _ => None,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            LValue::Ident(ident) => ident.span(),
            LValue::MemberAccess { span, .. }
            | LValue::Index { span, .. }
            | LValue::Dereference(_, span) => *span,
            LValue::Interned(_, span) => *span,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ForBounds {
    pub start: Expression,
    pub end: Expression,
    pub inclusive: bool,
}

impl ForBounds {
    /// Create a half-open range bounded inclusively below and exclusively above (`start..end`),  
    /// desugaring `start..=end` into `start..end+1` if necessary.
    ///
    /// Returns the `start` and `end` expressions.
    pub(crate) fn into_half_open(self) -> (Expression, Expression) {
        let end = if self.inclusive {
            let end_span = self.end.span;
            let end = ExpressionKind::Infix(Box::new(InfixExpression {
                lhs: self.end,
                operator: Spanned::from(end_span, BinaryOpKind::Add),
                rhs: Expression::new(ExpressionKind::integer(FieldElement::from(1u32)), end_span),
            }));
            Expression::new(end, end_span)
        } else {
            self.end
        };

        (self.start, end)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ForRange {
    Range(ForBounds),
    Array(Expression),
}

impl ForRange {
    /// Create a half-open range, bounded inclusively below and exclusively above.
    pub fn range(start: Expression, end: Expression) -> Self {
        Self::Range(ForBounds { start, end, inclusive: false })
    }

    /// Create a range bounded inclusively below and above.
    pub fn range_inclusive(start: Expression, end: Expression) -> Self {
        Self::Range(ForBounds { start, end, inclusive: true })
    }

    /// Create a range over some array.
    pub fn array(value: Expression) -> Self {
        Self::Array(value)
    }

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
                    kind: StatementKind::new_let(
                        Pattern::Identifier(array_ident.clone()),
                        UnresolvedTypeData::Unspecified.with_span(Default::default()),
                        array,
                        vec![],
                    ),
                    span: array_span,
                };

                // array.len()
                let segments = vec![PathSegment::from(array_ident)];
                let array_ident = ExpressionKind::Variable(Path {
                    segments,
                    kind: PathKind::Plain,
                    span: array_span,
                });

                let end_range = ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                    object: Expression::new(array_ident.clone(), array_span),
                    method_name: Ident::new("len".to_string(), array_span),
                    generics: None,
                    is_macro_call: false,
                    arguments: vec![],
                }));
                let end_range = Expression::new(end_range, array_span);

                let next_unique_id = UNIQUE_NAME_COUNTER.fetch_add(1, Ordering::Relaxed);
                let index_name = format!("$i{next_unique_id}");
                let fresh_identifier = Ident::new(index_name.clone(), array_span);

                // array[i]
                let segments = vec![PathSegment::from(Ident::new(index_name, array_span))];
                let index_ident = ExpressionKind::Variable(Path {
                    segments,
                    kind: PathKind::Plain,
                    span: array_span,
                });

                let loop_element = ExpressionKind::Index(Box::new(IndexExpression {
                    collection: Expression::new(array_ident, array_span),
                    index: Expression::new(index_ident, array_span),
                }));

                // let elem = array[i];
                let let_elem = Statement {
                    kind: StatementKind::new_let(
                        Pattern::Identifier(identifier),
                        UnresolvedTypeData::Unspecified.with_span(Default::default()),
                        Expression::new(loop_element, array_span),
                        vec![],
                    ),
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
                        range: ForRange::range(start_range, end_range),
                        block: new_block,
                        span: for_loop_span,
                    }),
                    span: for_loop_span,
                };

                let block = ExpressionKind::Block(BlockExpression {
                    statements: vec![let_array, for_loop],
                });
                Statement {
                    kind: StatementKind::Expression(Expression::new(block, for_loop_span)),
                    span: for_loop_span,
                }
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
            StatementKind::Interned(_) => write!(f, "(resolved);"),
            StatementKind::Error => write!(f, "Error"),
        }
    }
}

impl Display for LetStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(&self.r#type.typ, UnresolvedTypeData::Unspecified) {
            write!(f, "let {} = {}", self.pattern, self.expression)
        } else {
            write!(f, "let {}: {} = {}", self.pattern, self.r#type, self.expression)
        }
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
            LValue::Interned(_, _) => write!(f, "?Interned"),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let segments = vecmap(&self.segments, ToString::to_string);
        if self.kind == PathKind::Plain {
            write!(f, "{}", segments.join("::"))
        } else {
            write!(f, "{}::{}", self.kind, segments.join("::"))
        }
    }
}

impl Display for PathKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathKind::Crate => write!(f, "crate"),
            PathKind::Dep => write!(f, "dep"),
            PathKind::Super => write!(f, "super"),
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
            Pattern::Interned(_, _) => {
                write!(f, "?Interned")
            }
        }
    }
}

impl Display for ForLoopStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let range = match &self.range {
            ForRange::Range(bounds) => {
                format!(
                    "{}{}{}",
                    bounds.start,
                    if bounds.inclusive { "..=" } else { ".." },
                    bounds.end
                )
            }
            ForRange::Array(expr) => expr.to_string(),
        };

        write!(f, "for {} in {range} {}", self.identifier, self.block)
    }
}
