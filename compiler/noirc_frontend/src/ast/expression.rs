use std::borrow::Cow;
use std::fmt::Display;

use thiserror::Error;

use crate::ast::{
    Ident, ItemVisibility, Path, Pattern, Recoverable, Statement, StatementKind,
    UnresolvedTraitConstraint, UnresolvedType, UnresolvedTypeData, Visibility,
};
use crate::node_interner::{
    ExprId, InternedExpressionKind, InternedStatementKind, QuotedTypeId, StructId,
};
use crate::token::{Attributes, FmtStrFragment, FunctionAttribute, Token, Tokens};
use crate::{Kind, Type};
use acvm::{acir::AcirField, FieldElement};
use iter_extended::vecmap;
use noirc_errors::{Span, Spanned};

use super::{AsTraitPath, TypePath, UnaryRhsMemberAccess};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpressionKind {
    Literal(Literal),
    Block(BlockExpression),
    Prefix(Box<PrefixExpression>),
    Index(Box<IndexExpression>),
    Call(Box<CallExpression>),
    MethodCall(Box<MethodCallExpression>),
    Constructor(Box<ConstructorExpression>),
    MemberAccess(Box<MemberAccessExpression>),
    Cast(Box<CastExpression>),
    Infix(Box<InfixExpression>),
    If(Box<IfExpression>),
    Variable(Path),
    Tuple(Vec<Expression>),
    Lambda(Box<Lambda>),
    Parenthesized(Box<Expression>),
    Quote(Tokens),
    Unquote(Box<Expression>),
    Comptime(BlockExpression, Span),
    Unsafe(BlockExpression, Span),
    AsTraitPath(AsTraitPath),
    TypePath(TypePath),

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
    Variable(Ident),
    Numeric {
        ident: Ident,
        typ: UnresolvedType,
    },

    /// Already-resolved generics can be parsed as generics when a macro
    /// splices existing types into a generic list. In this case we have
    /// to validate the type refers to a named generic and treat that
    /// as a ResolvedGeneric when this is resolved.
    Resolved(QuotedTypeId, Span),
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("The only supported types of numeric generics are integers, fields, and booleans")]
pub struct UnsupportedNumericGenericType {
    pub ident: Ident,
    pub typ: UnresolvedTypeData,
}

impl UnresolvedGeneric {
    pub fn span(&self) -> Span {
        match self {
            UnresolvedGeneric::Variable(ident) => ident.0.span(),
            UnresolvedGeneric::Numeric { ident, typ } => ident.0.span().merge(typ.span),
            UnresolvedGeneric::Resolved(_, span) => *span,
        }
    }

    pub fn kind(&self) -> Result<Kind, UnsupportedNumericGenericType> {
        match self {
            UnresolvedGeneric::Variable(_) => Ok(Kind::Normal),
            UnresolvedGeneric::Numeric { typ, .. } => {
                let typ = self.resolve_numeric_kind_type(typ)?;
                Ok(Kind::numeric(typ))
            }
            UnresolvedGeneric::Resolved(..) => {
                panic!("Don't know the kind of a resolved generic here")
            }
        }
    }

    fn resolve_numeric_kind_type(
        &self,
        typ: &UnresolvedType,
    ) -> Result<Type, UnsupportedNumericGenericType> {
        use crate::ast::UnresolvedTypeData::{FieldElement, Integer};

        match typ.typ {
            FieldElement => Ok(Type::FieldElement),
            Integer(sign, bits) => Ok(Type::Integer(sign, bits)),
            // Only fields and integers are supported for numeric kinds
            _ => Err(UnsupportedNumericGenericType {
                ident: self.ident().clone(),
                typ: typ.typ.clone(),
            }),
        }
    }

    pub(crate) fn ident(&self) -> &Ident {
        match self {
            UnresolvedGeneric::Variable(ident) | UnresolvedGeneric::Numeric { ident, .. } => ident,
            UnresolvedGeneric::Resolved(..) => panic!("UnresolvedGeneric::Resolved no ident"),
        }
    }
}

impl Display for UnresolvedGeneric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedGeneric::Variable(ident) => write!(f, "{ident}"),
            UnresolvedGeneric::Numeric { ident, typ } => write!(f, "let {ident}: {typ}"),
            UnresolvedGeneric::Resolved(..) => write!(f, "(resolved)"),
        }
    }
}

impl From<Ident> for UnresolvedGeneric {
    fn from(value: Ident) -> Self {
        UnresolvedGeneric::Variable(value)
    }
}

impl ExpressionKind {
    pub fn into_path(self) -> Option<Path> {
        match self {
            ExpressionKind::Variable(path) => Some(path),
            _ => None,
        }
    }

    pub fn into_infix(self) -> Option<InfixExpression> {
        match self {
            ExpressionKind::Infix(infix) => Some(*infix),
            _ => None,
        }
    }

    pub fn prefix(operator: UnaryOp, rhs: Expression) -> ExpressionKind {
        match (operator, &rhs) {
            (
                UnaryOp::Minus,
                Expression { kind: ExpressionKind::Literal(Literal::Integer(field, sign)), .. },
            ) => ExpressionKind::Literal(Literal::Integer(*field, !sign)),
            _ => ExpressionKind::Prefix(Box::new(PrefixExpression { operator, rhs })),
        }
    }

    pub fn array(contents: Vec<Expression>) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(contents)))
    }

    pub fn repeated_array(repeated_element: Expression, length: Expression) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Array(ArrayLiteral::Repeated {
            repeated_element: Box::new(repeated_element),
            length: Box::new(length),
        }))
    }

    pub fn slice(contents: Vec<Expression>) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Standard(contents)))
    }

    pub fn repeated_slice(repeated_element: Expression, length: Expression) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Repeated {
            repeated_element: Box::new(repeated_element),
            length: Box::new(length),
        }))
    }

    pub fn integer(contents: FieldElement) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Integer(contents, false))
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

    pub fn constructor(
        (typ, fields): (UnresolvedType, Vec<(Ident, Expression)>),
    ) -> ExpressionKind {
        ExpressionKind::Constructor(Box::new(ConstructorExpression {
            typ,
            fields,
            struct_type: None,
        }))
    }
}

impl Recoverable for ExpressionKind {
    fn error(_: Span) -> Self {
        ExpressionKind::Error
    }
}

impl Recoverable for Expression {
    fn error(span: Span) -> Self {
        Expression::new(ExpressionKind::Error, span)
    }
}

impl Recoverable for Option<Expression> {
    fn error(span: Span) -> Self {
        Some(Expression::new(ExpressionKind::Error, span))
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

// This is important for tests. Two expressions are the same, if their Kind is the same
// We are ignoring Span
impl PartialEq<Expression> for Expression {
    fn eq(&self, rhs: &Expression) -> bool {
        self.kind == rhs.kind
    }
}

impl Expression {
    pub fn new(kind: ExpressionKind, span: Span) -> Expression {
        Expression { kind, span }
    }

    pub fn member_access_or_method_call(
        lhs: Expression,
        rhs: UnaryRhsMemberAccess,
        span: Span,
    ) -> Expression {
        let kind = match rhs.method_call {
            None => {
                let rhs = rhs.method_or_field;
                ExpressionKind::MemberAccess(Box::new(MemberAccessExpression { lhs, rhs }))
            }
            Some(method_call) => ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                object: lhs,
                method_name: rhs.method_or_field,
                generics: method_call.turbofish,
                arguments: method_call.args,
                is_macro_call: method_call.macro_call,
            })),
        };
        Expression::new(kind, span)
    }

    pub fn index(collection: Expression, index: Expression, span: Span) -> Expression {
        let kind = ExpressionKind::Index(Box::new(IndexExpression { collection, index }));
        Expression::new(kind, span)
    }

    pub fn cast(lhs: Expression, r#type: UnresolvedType, span: Span) -> Expression {
        let kind = ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type }));
        Expression::new(kind, span)
    }

    pub fn call(
        lhs: Expression,
        is_macro_call: bool,
        arguments: Vec<Expression>,
        span: Span,
    ) -> Expression {
        let func = Box::new(lhs);
        let kind =
            ExpressionKind::Call(Box::new(CallExpression { func, is_macro_call, arguments }));
        Expression::new(kind, span)
    }
}

pub type BinaryOp = Spanned<BinaryOpKind>;

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
#[cfg_attr(test, derive(strum_macros::EnumIter))]
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

    pub fn as_string(self) -> &'static str {
        match self {
            BinaryOpKind::Add => "+",
            BinaryOpKind::Subtract => "-",
            BinaryOpKind::Multiply => "*",
            BinaryOpKind::Divide => "/",
            BinaryOpKind::Equal => "==",
            BinaryOpKind::NotEqual => "!=",
            BinaryOpKind::Less => "<",
            BinaryOpKind::LessEqual => "<=",
            BinaryOpKind::Greater => ">",
            BinaryOpKind::GreaterEqual => ">=",
            BinaryOpKind::And => "&",
            BinaryOpKind::Or => "|",
            BinaryOpKind::Xor => "^",
            BinaryOpKind::ShiftRight => ">>",
            BinaryOpKind::ShiftLeft => "<<",
            BinaryOpKind::Modulo => "%",
        }
    }

    pub fn as_token(self) -> Token {
        match self {
            BinaryOpKind::Add => Token::Plus,
            BinaryOpKind::Subtract => Token::Minus,
            BinaryOpKind::Multiply => Token::Star,
            BinaryOpKind::Divide => Token::Slash,
            BinaryOpKind::Equal => Token::Equal,
            BinaryOpKind::NotEqual => Token::NotEqual,
            BinaryOpKind::Less => Token::Less,
            BinaryOpKind::LessEqual => Token::LessEqual,
            BinaryOpKind::Greater => Token::Greater,
            BinaryOpKind::GreaterEqual => Token::GreaterEqual,
            BinaryOpKind::And => Token::Ampersand,
            BinaryOpKind::Or => Token::Pipe,
            BinaryOpKind::Xor => Token::Caret,
            BinaryOpKind::ShiftLeft => Token::ShiftLeft,
            BinaryOpKind::ShiftRight => Token::ShiftRight,
            BinaryOpKind::Modulo => Token::Percent,
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
pub enum UnaryOp {
    Minus,
    Not,
    MutableReference,

    /// If implicitly_added is true, this operation was implicitly added by the compiler for a
    /// field dereference. The compiler may undo some of these implicitly added dereferences if
    /// the reference later turns out to be needed (e.g. passing a field by reference to a function
    /// requiring an &mut parameter).
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
    Integer(FieldElement, /*sign*/ bool), // false for positive integer and true for negative
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
pub struct Lambda {
    pub parameters: Vec<(Pattern, UnresolvedType)>,
    pub return_type: UnresolvedType,
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
    pub span: Span,
    pub where_clause: Vec<UnresolvedTraitConstraint>,
    pub return_type: FunctionReturnType,
    pub return_visibility: Visibility,
}

impl FunctionDefinition {
    pub fn is_private(&self) -> bool {
        self.visibility == ItemVisibility::Private
    }

    pub fn is_test(&self) -> bool {
        if let Some(attribute) = self.attributes.function() {
            matches!(attribute, FunctionAttribute::Test(..))
        } else {
            false
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Param {
    pub visibility: Visibility,
    pub pattern: Pattern,
    pub typ: UnresolvedType,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionReturnType {
    /// Returns type is not specified.
    Default(Span),
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

    /// This may be filled out during macro expansion
    /// so that we can skip re-resolving the type name since it
    /// would be lost at that point.
    pub struct_type: Option<StructId>,
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
    pub fn pop(&mut self) -> Option<StatementKind> {
        self.statements.pop().map(|stmt| stmt.kind)
    }

    pub fn len(&self) -> usize {
        self.statements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
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
            Cast(cast) => cast.fmt(f),
            Infix(infix) => infix.fmt(f),
            If(if_expr) => if_expr.fmt(f),
            Variable(path) => path.fmt(f),
            Constructor(constructor) => constructor.fmt(f),
            MemberAccess(access) => access.fmt(f),
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Lambda(lambda) => lambda.fmt(f),
            Parenthesized(sub_expr) => write!(f, "({sub_expr})"),
            Comptime(block, _) => write!(f, "comptime {block}"),
            Unsafe(block, _) => write!(f, "unsafe {block}"),
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
            Literal::Integer(integer, sign) => {
                if *sign {
                    write!(f, "-{}", integer.to_u128())
                } else {
                    write!(f, "{}", integer.to_u128())
                }
            }
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
            UnaryOp::MutableReference => write!(f, "&mut"),
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
        let fields =
            self.fields.iter().map(|(ident, expr)| format!("{ident}: {expr}")).collect::<Vec<_>>();

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

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parameters = vecmap(&self.parameters, |(name, r#type)| format!("{name}: {type}"));

        write!(f, "|{}| -> {} {{ {} }}", parameters.join(", "), self.return_type, self.body)
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
            write!(f, "::{}", turbofish)?;
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
                span: ident.span().merge(unresolved_type.span),
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
            span: name.span(),
            where_clause,
            return_type: return_type.clone(),
            return_visibility: Visibility::Private,
        }
    }

    pub fn signature(&self) -> String {
        let parameters = vecmap(&self.parameters, |Param { visibility, pattern, typ, span: _ }| {
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
            FunctionReturnType::Default(span) => {
                Cow::Owned(UnresolvedType { typ: UnresolvedTypeData::Unit, span: *span })
            }
            FunctionReturnType::Ty(typ) => Cow::Borrowed(typ),
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
