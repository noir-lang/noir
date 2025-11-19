use std::borrow::Cow;
use std::fmt::Display;

use thiserror::Error;

use crate::ast::{
    Ident, ItemVisibility, Path, Pattern, Statement, UnresolvedTraitConstraint, UnresolvedType,
    UnresolvedTypeData,
};
use crate::elaborator::PrimitiveType;
use crate::node_interner::{ExprId, InternedExpressionKind, InternedStatementKind, QuotedTypeId};
use crate::shared::Visibility;
use crate::signed_field::SignedField;
use crate::token::{Attributes, FmtStrFragment, IntegerTypeSuffix, Token, Tokens};
use crate::{Kind, Type};
use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_errors::{Located, Location, Span};

use super::{AsTraitPath, TraitBound, TypePath, UnsafeExpression};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpressionKind {
    Literal(Literal),
    Block(BlockExpression),
    Prefix(Box<PrefixExpression>),
    Index(Box<IndexExpression>),
    Call(Box<CallExpression>),
    MethodCall(Box<MethodCallExpression>),
    Constrain(ConstrainExpression),
    Constructor(Box<ConstructorExpression>),
    MemberAccess(Box<MemberAccessExpression>),
    Cast(Box<CastExpression>),
    Infix(Box<InfixExpression>),
    If(Box<IfExpression>),
    Match(Box<MatchExpression>),
    Variable(Path),
    Tuple(Vec<Expression>),
    Lambda(Box<Lambda>),
    Parenthesized(Box<Expression>),
    Quote(Tokens),
    Unquote(Box<Expression>),
    Comptime(BlockExpression, Location),
    Unsafe(UnsafeExpression),
    AsTraitPath(Box<AsTraitPath>),
    TypePath(Box<TypePath>),

    // This variant is only emitted when inlining the result of comptime
    // code. It is used to translate function values back into the AST while
    // guaranteeing they have the same instantiated type and definition id without resolving again.
    Resolved(ExprId),

    // This is an interned ExpressionKind during comptime code.
    // The actual ExpressionKind can be retrieved with a NodeInterner.
    Interned(InternedExpressionKind),

    /// Interned statements are allowed to be parsed as expressions in case they resolve
    /// to an StatementKind::Expression or StatementKind::Semi.
    InternedStatement(InternedStatementKind),

    Error,
}

/// A Vec of unresolved names for type variables.
/// For `fn foo<A, B>(...)` this corresponds to vec!["A", "B"].
pub type UnresolvedGenerics = Vec<UnresolvedGeneric>;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnresolvedGeneric {
    Variable(IdentOrQuotedType, Vec<TraitBound>),
    Numeric { ident: IdentOrQuotedType, typ: UnresolvedType },
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum IdentOrQuotedType {
    Ident(Ident),

    /// Already-resolved generics can be parsed as generics when a macro
    /// splices existing types into a generic list. In this case we have
    /// to validate the type refers to a named generic and treat that
    /// as a ResolvedGeneric when this is resolved.
    Quoted(QuotedTypeId, Location),
}

impl IdentOrQuotedType {
    pub fn location(&self) -> Location {
        match self {
            IdentOrQuotedType::Ident(ident) => ident.location(),
            IdentOrQuotedType::Quoted(_, location) => *location,
        }
    }

    pub fn ident(&self) -> Option<&Ident> {
        match self {
            IdentOrQuotedType::Ident(ident) => Some(ident),
            IdentOrQuotedType::Quoted(_, _) => None,
        }
    }
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error(
    "`{typ}` is not a supported type for a numeric generic. The only supported types are integers, fields, and booleans"
)]
pub struct UnsupportedNumericGenericType {
    pub name: Option<String>,
    pub typ: String,
    pub location: Location,
}

impl UnresolvedGeneric {
    pub fn location(&self) -> Location {
        match self {
            UnresolvedGeneric::Variable(ident, _) => ident.location(),
            UnresolvedGeneric::Numeric { ident, typ } => ident.location().merge(typ.location),
        }
    }

    pub fn span(&self) -> Span {
        self.location().span
    }

    pub fn kind(&self) -> Result<Kind, UnsupportedNumericGenericType> {
        match self {
            UnresolvedGeneric::Variable(_, _) => Ok(Kind::Normal),
            UnresolvedGeneric::Numeric { typ, .. } => {
                let typ = self.resolve_numeric_kind_type(typ)?;
                Ok(Kind::numeric(typ))
            }
        }
    }

    fn resolve_numeric_kind_type(
        &self,
        typ: &UnresolvedType,
    ) -> Result<Type, UnsupportedNumericGenericType> {
        // TODO: this should be done with resolved types
        // See https://github.com/noir-lang/noir/issues/8504
        use crate::ast::UnresolvedTypeData::Named;

        if let Named(path, _generics, _) = &typ.typ {
            if path.segments.len() == 1 {
                if let Some(primitive_type) =
                    PrimitiveType::lookup_by_name(path.segments[0].ident.as_str())
                {
                    if let Some(typ) = primitive_type.to_integer_or_field() {
                        return Ok(typ);
                    }
                }
            }
        }

        // Only fields and integers are supported for numeric kinds
        let name = self.ident().ident().map(|name| name.to_string());
        let type_string = typ.typ.to_string();
        Err(UnsupportedNumericGenericType { name, typ: type_string, location: typ.location })
    }

    pub fn ident(&self) -> &IdentOrQuotedType {
        match self {
            UnresolvedGeneric::Variable(ident, _) | UnresolvedGeneric::Numeric { ident, .. } => {
                ident
            }
        }
    }
}

impl Display for UnresolvedGeneric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedGeneric::Variable(ident, trait_bounds) => {
                write!(f, "{ident}")?;
                if !trait_bounds.is_empty() {
                    write!(f, ": ")?;
                    for (index, trait_bound) in trait_bounds.iter().enumerate() {
                        if index > 0 {
                            write!(f, " + ")?;
                        }
                        write!(f, "{trait_bound}")?;
                    }
                }
                Ok(())
            }
            UnresolvedGeneric::Numeric { ident, typ } => write!(f, "let {ident}: {typ}"),
        }
    }
}

impl Display for IdentOrQuotedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentOrQuotedType::Ident(ident) => write!(f, "{ident}"),
            IdentOrQuotedType::Quoted(..) => write!(f, "(quoted)"),
        }
    }
}

impl From<Ident> for UnresolvedGeneric {
    fn from(value: Ident) -> Self {
        UnresolvedGeneric::Variable(IdentOrQuotedType::Ident(value), Vec::new())
    }
}

impl ExpressionKind {
    pub fn prefix(operator: UnaryOp, rhs: Expression) -> ExpressionKind {
        match (operator, &rhs) {
            (
                UnaryOp::Minus,
                Expression {
                    kind: ExpressionKind::Literal(Literal::Integer(field, suffix)), ..
                },
            ) if !field.is_negative() => {
                ExpressionKind::Literal(Literal::Integer(-*field, *suffix))
            }
            _ => ExpressionKind::Prefix(Box::new(PrefixExpression { operator, rhs })),
        }
    }

    pub fn integer(contents: FieldElement, suffix: Option<IntegerTypeSuffix>) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Integer(SignedField::positive(contents), suffix))
    }

    pub fn boolean(contents: bool) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Bool(contents))
    }

    pub fn string(contents: String) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Str(contents))
    }

    pub fn raw_string(contents: String, hashes: u8) -> ExpressionKind {
        ExpressionKind::Literal(Literal::RawStr(contents, hashes))
    }

    pub fn format_string(fragments: Vec<FmtStrFragment>, length: u32) -> ExpressionKind {
        ExpressionKind::Literal(Literal::FmtStr(fragments, length))
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub location: Location,
}

// This is important for tests. Two expressions are the same, if their Kind is the same
// We are ignoring Span
impl PartialEq<Expression> for Expression {
    fn eq(&self, rhs: &Expression) -> bool {
        self.kind == rhs.kind
    }
}

impl Expression {
    pub fn new(kind: ExpressionKind, location: Location) -> Expression {
        Expression { kind, location }
    }

    /// Returns the innermost location that gives this expression its type.
    pub fn type_location(&self) -> Location {
        match &self.kind {
            ExpressionKind::Block(block_expression)
            | ExpressionKind::Comptime(block_expression, _)
            | ExpressionKind::Unsafe(UnsafeExpression { block: block_expression, .. }) => {
                if let Some(statement) = block_expression.statements.last() {
                    statement.type_location()
                } else {
                    self.location
                }
            }
            ExpressionKind::Parenthesized(expression) => expression.type_location(),
            ExpressionKind::Literal(..)
            | ExpressionKind::Prefix(..)
            | ExpressionKind::Index(..)
            | ExpressionKind::Call(..)
            | ExpressionKind::MethodCall(..)
            | ExpressionKind::Constrain(..)
            | ExpressionKind::Constructor(..)
            | ExpressionKind::MemberAccess(..)
            | ExpressionKind::Cast(..)
            | ExpressionKind::Infix(..)
            | ExpressionKind::If(..)
            | ExpressionKind::Match(..)
            | ExpressionKind::Variable(..)
            | ExpressionKind::Tuple(..)
            | ExpressionKind::Lambda(..)
            | ExpressionKind::Quote(..)
            | ExpressionKind::Unquote(..)
            | ExpressionKind::AsTraitPath(..)
            | ExpressionKind::TypePath(..)
            | ExpressionKind::Resolved(..)
            | ExpressionKind::Interned(..)
            | ExpressionKind::InternedStatement(..)
            | ExpressionKind::Error => self.location,
        }
    }
}

pub type BinaryOp = Located<BinaryOpKind>;

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone, strum_macros::EnumIter)]
pub enum BinaryOpKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Xor,
    ShiftRight,
    ShiftLeft,
    Modulo,
}

impl BinaryOpKind {
    /// Comparator operators return a 0 or 1
    /// When seen in the middle of an infix operator,
    /// they transform the infix expression into a predicate expression
    pub fn is_comparator(self) -> bool {
        matches!(
            self,
            BinaryOpKind::Equal
                | BinaryOpKind::NotEqual
                | BinaryOpKind::LessEqual
                | BinaryOpKind::Less
                | BinaryOpKind::Greater
                | BinaryOpKind::GreaterEqual
        )
    }

    /// `==` and `!=`
    pub fn is_equality(self) -> bool {
        matches!(self, BinaryOpKind::Equal | BinaryOpKind::NotEqual)
    }

    /// `+`, `-`, `*`, `/` and `%`
    pub fn is_arithmetic(self) -> bool {
        matches!(
            self,
            BinaryOpKind::Add
                | BinaryOpKind::Subtract
                | BinaryOpKind::Multiply
                | BinaryOpKind::Divide
                | BinaryOpKind::Modulo
        )
    }

    pub fn is_bitwise(self) -> bool {
        matches!(self, BinaryOpKind::And | BinaryOpKind::Or | BinaryOpKind::Xor)
    }

    pub fn is_bitshift(self) -> bool {
        matches!(self, BinaryOpKind::ShiftLeft | BinaryOpKind::ShiftRight)
    }

    pub fn is_valid_for_field_type(self) -> bool {
        matches!(
            self,
            BinaryOpKind::Add
                | BinaryOpKind::Subtract
                | BinaryOpKind::Multiply
                | BinaryOpKind::Divide
                | BinaryOpKind::Equal
                | BinaryOpKind::NotEqual
        )
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
pub enum UnaryOp {
    Minus,
    Not,
    Reference {
        mutable: bool,
    },

    /// If `implicitly_added` is true, this operation was implicitly added by the compiler for a
    /// field dereference. The compiler may undo some of these implicitly added dereferences if
    /// the reference later turns out to be needed (e.g. passing a field by reference to a function
    /// requiring an `&mut` parameter).
    Dereference {
        implicitly_added: bool,
    },
}

impl UnaryOp {
    /// Converts a token to a unary operator
    /// If you want the parser to recognize another Token as being a prefix operator, it is defined here
    pub fn from(token: &Token) -> Option<UnaryOp> {
        match token {
            Token::Minus => Some(UnaryOp::Minus),
            Token::Bang => Some(UnaryOp::Not),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    Array(ArrayLiteral),
    Slice(ArrayLiteral),
    Bool(bool),
    Integer(SignedField, Option<IntegerTypeSuffix>),
    Str(String),
    RawStr(String, u8),
    FmtStr(Vec<FmtStrFragment>, u32 /* length */),
    Unit,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrefixExpression {
    pub operator: UnaryOp,
    pub rhs: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InfixExpression {
    pub lhs: Expression,
    pub operator: BinaryOp,
    pub rhs: Expression,
}

// This is an infix expression with 'as' as the binary operator
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CastExpression {
    pub lhs: Expression,
    pub r#type: UnresolvedType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IfExpression {
    pub condition: Expression,
    pub consequence: Expression,
    pub alternative: Option<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MatchExpression {
    pub expression: Expression,
    pub rules: Vec<(/*pattern*/ Expression, /*branch*/ Expression)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lambda {
    pub parameters: Vec<(Pattern, Option<UnresolvedType>)>,
    pub return_type: Option<UnresolvedType>,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDefinition {
    pub name: Ident,

    // The `Attributes` container holds both `primary` (ones that change the function kind)
    // and `secondary` attributes (ones that do not change the function kind)
    pub attributes: Attributes,

    /// True if this function was defined with the 'unconstrained' keyword
    pub is_unconstrained: bool,

    /// True if this function was defined with the 'comptime' keyword
    pub is_comptime: bool,

    /// Indicate if this function was defined with the 'pub' keyword
    pub visibility: ItemVisibility,

    pub generics: UnresolvedGenerics,
    pub parameters: Vec<Param>,
    pub body: BlockExpression,
    pub location: Location,
    pub where_clause: Vec<UnresolvedTraitConstraint>,
    pub return_type: FunctionReturnType,
    pub return_visibility: Visibility,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Param {
    pub visibility: Visibility,
    pub pattern: Pattern,
    pub typ: UnresolvedType,
    pub location: Location,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionReturnType {
    /// Returns type is not specified.
    Default(Location),
    /// Everything else.
    Ty(UnresolvedType),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArrayLiteral {
    Standard(Vec<Expression>),
    Repeated { repeated_element: Box<Expression>, length: Box<Expression> },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallExpression {
    pub func: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub is_macro_call: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MethodCallExpression {
    pub object: Expression,
    pub method_name: Ident,
    /// Method calls have an optional list of generics if the turbofish operator was used
    pub generics: Option<Vec<UnresolvedType>>,
    pub arguments: Vec<Expression>,
    pub is_macro_call: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstructorExpression {
    pub typ: UnresolvedType,
    pub fields: Vec<(Ident, Expression)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MemberAccessExpression {
    pub lhs: Expression,
    pub rhs: Ident,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexExpression {
    pub collection: Expression, // XXX: For now, this will be the name of the array, as we do not support other collections
    pub index: Expression, // XXX: We accept two types of indices, either a normal integer or a constant
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockExpression {
    pub statements: Vec<Statement>,
}

impl BlockExpression {
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainExpression {
    pub kind: ConstrainKind,
    pub arguments: Vec<Expression>,
    pub location: Location,
}

impl Display for ConstrainExpression {
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
    /// The number of arguments expected by the constraint,
    /// not counting the optional assertion message.
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

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl Display for ExpressionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ExpressionKind::*;
        match self {
            Literal(literal) => literal.fmt(f),
            Block(block) => block.fmt(f),
            Prefix(prefix) => prefix.fmt(f),
            Index(index) => index.fmt(f),
            Call(call) => call.fmt(f),
            MethodCall(call) => call.fmt(f),
            Constrain(constrain) => constrain.fmt(f),
            Cast(cast) => cast.fmt(f),
            Infix(infix) => infix.fmt(f),
            If(if_expr) => if_expr.fmt(f),
            Match(match_expr) => match_expr.fmt(f),
            Variable(path) => path.fmt(f),
            Constructor(constructor) => constructor.fmt(f),
            MemberAccess(access) => access.fmt(f),
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                if elements.len() == 1 {
                    write!(f, "({},)", elements[0])
                } else {
                    write!(f, "({})", elements.join(", "))
                }
            }
            Lambda(lambda) => lambda.fmt(f),
            Parenthesized(sub_expr) => write!(f, "({sub_expr})"),
            Comptime(block, _) => write!(f, "comptime {block}"),
            Unsafe(UnsafeExpression { block, .. }) => write!(f, "unsafe {block}"),
            Error => write!(f, "Error"),
            Resolved(_) => write!(f, "?Resolved"),
            Interned(_) => write!(f, "?Interned"),
            Unquote(expr) => write!(f, "$({expr})"),
            Quote(tokens) => {
                let tokens = vecmap(&tokens.0, ToString::to_string);
                write!(f, "quote {{ {} }}", tokens.join(" "))
            }
            AsTraitPath(path) => write!(f, "{path}"),
            TypePath(path) => write!(f, "{path}"),
            InternedStatement(_) => write!(f, "?InternedStatement"),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Array(ArrayLiteral::Standard(elements)) => {
                let contents = vecmap(elements, ToString::to_string);
                write!(f, "[{}]", contents.join(", "))
            }
            Literal::Array(ArrayLiteral::Repeated { repeated_element, length }) => {
                write!(f, "[{repeated_element}; {length}]")
            }
            Literal::Slice(ArrayLiteral::Standard(elements)) => {
                let contents = vecmap(elements, ToString::to_string);
                write!(f, "&[{}]", contents.join(", "))
            }
            Literal::Slice(ArrayLiteral::Repeated { repeated_element, length }) => {
                write!(f, "&[{repeated_element}; {length}]")
            }
            Literal::Bool(boolean) => write!(f, "{}", if *boolean { "true" } else { "false" }),
            Literal::Integer(signed_field, Some(suffix)) => write!(f, "{signed_field}_{suffix}"),
            Literal::Integer(signed_field, None) => write!(f, "{signed_field}"),
            Literal::Str(string) => write!(f, "\"{string}\""),
            Literal::RawStr(string, num_hashes) => {
                let hashes: String =
                    std::iter::once('#').cycle().take(*num_hashes as usize).collect();
                write!(f, "r{hashes}\"{string}\"{hashes}")
            }
            Literal::FmtStr(fragments, _length) => {
                write!(f, "f\"")?;
                for fragment in fragments {
                    fragment.fmt(f)?;
                }
                write!(f, "\"")
            }
            Literal::Unit => write!(f, "()"),
        }
    }
}

impl Display for BlockExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in &self.statements {
            let statement = statement.kind.to_string();
            for line in statement.lines() {
                writeln!(f, "    {line}")?;
            }
        }
        write!(f, "}}")
    }
}

impl Display for PrefixExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.rhs)
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Minus => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Reference { mutable } if *mutable => write!(f, "&mut"),
            UnaryOp::Reference { .. } => write!(f, "&"),
            UnaryOp::Dereference { .. } => write!(f, "*"),
        }
    }
}

impl Display for IndexExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.collection, self.index)
    }
}

impl Display for CallExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = vecmap(&self.arguments, ToString::to_string);
        write!(f, "{}({})", self.func, args.join(", "))
    }
}

impl Display for MethodCallExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = vecmap(&self.arguments, ToString::to_string);
        write!(f, "{}.{}({})", self.object, self.method_name, args.join(", "))
    }
}

impl Display for CastExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} as {})", self.lhs, self.r#type)
    }
}

impl Display for ConstructorExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields = vecmap(&self.fields, |(ident, expr)| format!("{ident}: {expr}"));

        write!(f, "({} {{ {} }})", self.typ, fields.join(", "))
    }
}

impl Display for MemberAccessExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}.{})", self.lhs, self.rhs)
    }
}

impl Display for InfixExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.lhs, self.operator.contents, self.rhs)
    }
}

impl Display for BinaryOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOpKind::Add => write!(f, "+"),
            BinaryOpKind::Subtract => write!(f, "-"),
            BinaryOpKind::Multiply => write!(f, "*"),
            BinaryOpKind::Divide => write!(f, "/"),
            BinaryOpKind::Equal => write!(f, "=="),
            BinaryOpKind::NotEqual => write!(f, "!="),
            BinaryOpKind::Less => write!(f, "<"),
            BinaryOpKind::LessEqual => write!(f, "<="),
            BinaryOpKind::Greater => write!(f, ">"),
            BinaryOpKind::GreaterEqual => write!(f, ">="),
            BinaryOpKind::And => write!(f, "&"),
            BinaryOpKind::Or => write!(f, "|"),
            BinaryOpKind::Xor => write!(f, "^"),
            BinaryOpKind::ShiftLeft => write!(f, "<<"),
            BinaryOpKind::ShiftRight => write!(f, ">>"),
            BinaryOpKind::Modulo => write!(f, "%"),
        }
    }
}

impl Display for IfExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} {}", self.condition, self.consequence)?;
        if let Some(alternative) = &self.alternative {
            write!(f, " else {alternative}")?;
        }
        Ok(())
    }
}

impl Display for MatchExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "match {} {{", self.expression)?;
        for (pattern, branch) in &self.rules {
            writeln!(f, "    {pattern} => {branch},")?;
        }
        write!(f, "}}")
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parameters = vecmap(&self.parameters, |(name, r#type)| {
            if let Some(typ) = r#type { format!("{name}: {typ}") } else { format!("{name}") }
        });

        let parameters = parameters.join(", ");
        if let Some(return_type) = &self.return_type {
            write!(f, "|{}| -> {} {{ {} }}", parameters, return_type, self.body)
        } else {
            write!(f, "|{}| {{ {} }}", parameters, self.body)
        }
    }
}

impl Display for AsTraitPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} as {}>::{}", self.typ, self.trait_path, self.impl_item)
    }
}

impl Display for TypePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.typ, self.item)?;
        if let Some(turbofish) = &self.turbofish {
            write!(f, "::{turbofish}")?;
        }
        Ok(())
    }
}

impl FunctionDefinition {
    pub fn normal(
        name: &Ident,
        is_unconstrained: bool,
        generics: &UnresolvedGenerics,
        parameters: &[(Ident, UnresolvedType)],
        body: BlockExpression,
        where_clause: Vec<UnresolvedTraitConstraint>,
        return_type: &FunctionReturnType,
    ) -> FunctionDefinition {
        let p = parameters
            .iter()
            .map(|(ident, unresolved_type)| Param {
                visibility: Visibility::Private,
                pattern: Pattern::Identifier(ident.clone()),
                typ: unresolved_type.clone(),
                location: ident.location().merge(unresolved_type.location),
            })
            .collect();

        FunctionDefinition {
            name: name.clone(),
            attributes: Attributes::empty(),
            is_unconstrained,
            is_comptime: false,
            visibility: ItemVisibility::Private,
            generics: generics.clone(),
            parameters: p,
            body,
            location: name.location(),
            where_clause,
            return_type: return_type.clone(),
            return_visibility: Visibility::Private,
        }
    }

    pub fn signature(&self) -> String {
        let parameters =
            vecmap(&self.parameters, |Param { visibility, pattern, typ, location: _ }| {
                if *visibility == Visibility::Public {
                    format!("{pattern}: {visibility} {typ}")
                } else {
                    format!("{pattern}: {typ}")
                }
            });

        let where_clause = vecmap(&self.where_clause, ToString::to_string);
        let where_clause_str = if !where_clause.is_empty() {
            format!(" where {}", where_clause.join(", "))
        } else {
            "".to_string()
        };

        let return_type = if matches!(&self.return_type, FunctionReturnType::Default(_)) {
            String::new()
        } else {
            format!(" -> {}", self.return_type)
        };

        format!("fn {}({}){}{}", self.name, parameters.join(", "), return_type, where_clause_str)
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.attributes)?;
        write!(f, "{} {}", self.signature(), self.body)
    }
}

impl FunctionReturnType {
    pub fn get_type(&self) -> Cow<UnresolvedType> {
        match self {
            FunctionReturnType::Default(location) => {
                Cow::Owned(UnresolvedType { typ: UnresolvedTypeData::Unit, location: *location })
            }
            FunctionReturnType::Ty(typ) => Cow::Borrowed(typ),
        }
    }

    pub fn location(&self) -> Location {
        match self {
            FunctionReturnType::Default(location) => *location,
            FunctionReturnType::Ty(typ) => typ.location,
        }
    }
}

impl Display for FunctionReturnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionReturnType::Default(_) => f.write_str(""),
            FunctionReturnType::Ty(ty) => write!(f, "{ty}"),
        }
    }
}
