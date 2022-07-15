use std::fmt::Display;

use crate::token::{Attribute, Token};
use crate::util::vecmap;
use crate::{Ident, Path, Pattern, Recoverable, Statement, UnresolvedType};
use acvm::FieldElement;
use noirc_errors::{Span, Spanned};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpressionKind {
    Ident(String),
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
    For(Box<ForExpression>),
    If(Box<IfExpression>),
    Path(Path),
    Tuple(Vec<Expression>),
    Error,
}

impl ExpressionKind {
    pub fn into_path(self) -> Option<Path> {
        match self {
            ExpressionKind::Path(path) => Some(path),
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
        ExpressionKind::Prefix(Box::new(PrefixExpression { operator, rhs }))
    }

    pub fn array(contents: Vec<Expression>) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Array(ArrayLiteral {
            length: contents.len() as u128,
            contents,
        }))
    }

    pub fn integer(contents: FieldElement) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Integer(contents))
    }

    pub fn boolean(contents: bool) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Bool(contents))
    }

    pub fn string(contents: String) -> ExpressionKind {
        ExpressionKind::Literal(Literal::Str(contents))
    }

    pub fn function_call((func_name, arguments): (Path, Vec<Expression>)) -> ExpressionKind {
        ExpressionKind::Call(Box::new(CallExpression { func_name, arguments }))
    }

    pub fn constructor((type_name, fields): (Path, Vec<(Ident, Expression)>)) -> ExpressionKind {
        ExpressionKind::Constructor(Box::new(ConstructorExpression { type_name, fields }))
    }

    /// Returns true if the expression is a literal integer
    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    fn as_integer(&self) -> Option<FieldElement> {
        let literal = match self {
            ExpressionKind::Literal(literal) => literal,
            _ => return None,
        };

        match literal {
            Literal::Integer(integer) => Some(*integer),
            _ => None,
        }
    }

    /// Returns true if the expression can be used in a range expression
    /// Currently we only support Identifiers and constants literals
    pub fn can_be_used_range(&self) -> bool {
        self.is_identifier() || self.is_integer()
    }

    /// Returns true if the expression is an identifier
    fn is_identifier(&self) -> bool {
        self.as_identifier().is_some()
    }

    fn as_identifier(&self) -> Option<String> {
        match self {
            ExpressionKind::Ident(x) => Some(x.clone()),
            _ => None,
        }
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

    pub fn into_ident(self) -> Option<Ident> {
        let identifier = match self.kind {
            ExpressionKind::Ident(x) => x,
            _ => return None,
        };

        let ident = Ident(Spanned::from(self.span, identifier));
        Some(ident)
    }

    pub fn member_access_or_method_call(
        lhs: Expression,
        (rhs, args): (Ident, Option<Vec<Expression>>),
        span: Span,
    ) -> Expression {
        let kind = match args {
            None => ExpressionKind::MemberAccess(Box::new(MemberAccessExpression { lhs, rhs })),
            Some(arguments) => ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                object: lhs,
                method_name: rhs,
                arguments,
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ForExpression {
    pub identifier: Ident,
    pub start_range: Expression,
    pub end_range: Expression,
    pub block: Expression,
}

pub type BinaryOp = Spanned<BinaryOpKind>;

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
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
    // Assign is the only binary operator which cannot be used in a constrain statement
    Assign,
}

impl BinaryOpKind {
    /// Comparator operators return a 0 or 1
    /// When seen in the middle of an infix operator,
    /// they transform the infix expression into a predicate expression
    pub fn is_comparator(&self) -> bool {
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
            BinaryOpKind::Assign => "=",
            BinaryOpKind::ShiftRight => ">>",
            BinaryOpKind::ShiftLeft => "<<",
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
            BinaryOpKind::Assign => Token::Assign,
        }
    }
}

impl From<&Token> for Option<BinaryOpKind> {
    fn from(token: &Token) -> Option<BinaryOpKind> {
        let op = match token {
            Token::Plus => BinaryOpKind::Add,
            Token::Ampersand => BinaryOpKind::And,
            Token::Caret => BinaryOpKind::Xor,
            Token::ShiftLeft => BinaryOpKind::ShiftLeft,
            Token::ShiftRight => BinaryOpKind::ShiftRight,
            Token::Pipe => BinaryOpKind::Or,
            Token::Minus => BinaryOpKind::Subtract,
            Token::Star => BinaryOpKind::Multiply,
            Token::Slash => BinaryOpKind::Divide,
            Token::Equal => BinaryOpKind::Equal,
            Token::NotEqual => BinaryOpKind::NotEqual,
            Token::Less => BinaryOpKind::Less,
            Token::LessEqual => BinaryOpKind::LessEqual,
            Token::Greater => BinaryOpKind::Greater,
            Token::GreaterEqual => BinaryOpKind::GreaterEqual,
            Token::Assign => BinaryOpKind::Assign,
            _ => return None,
        };
        Some(op)
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
pub enum UnaryOp {
    Minus,
    Not,
}

impl UnaryOp {
    /// Converts a token to a unary operator
    /// If you want the parser to recognise another Token as being a prefix operator, it is defined here
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
    Bool(bool),
    Integer(FieldElement),
    Str(String),
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
pub struct FunctionDefinition {
    pub name: Ident,
    pub attribute: Option<Attribute>, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
    pub parameters: Vec<(Pattern, UnresolvedType, noirc_abi::AbiFEType)>,
    pub body: BlockExpression,
    pub span: Span,
    pub return_type: UnresolvedType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayLiteral {
    pub length: u128, // XXX: Maybe allow field element, so that the user can define the length using a constant
    pub contents: Vec<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallExpression {
    pub func_name: Path,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MethodCallExpression {
    pub object: Expression,
    pub method_name: Ident,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstructorExpression {
    pub type_name: Path,
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
pub struct BlockExpression(pub Vec<Statement>);

impl BlockExpression {
    pub fn pop(&mut self) -> Option<Statement> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
            Ident(name) => name.fmt(f),
            Literal(literal) => literal.fmt(f),
            Block(block) => block.fmt(f),
            Prefix(prefix) => prefix.fmt(f),
            Index(index) => index.fmt(f),
            Call(call) => call.fmt(f),
            MethodCall(call) => call.fmt(f),
            Cast(cast) => cast.fmt(f),
            Infix(infix) => infix.fmt(f),
            For(for_loop) => for_loop.fmt(f),
            If(if_expr) => if_expr.fmt(f),
            Path(path) => path.fmt(f),
            Constructor(constructor) => constructor.fmt(f),
            MemberAccess(access) => access.fmt(f),
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Error => write!(f, "Error"),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Array(array) => {
                let contents = vecmap(&array.contents, ToString::to_string);
                write!(f, "[{}]", contents.join(", "))
            }
            Literal::Bool(boolean) => write!(f, "{}", if *boolean { "true" } else { "false" }),
            Literal::Integer(integer) => write!(f, "{}", integer.to_u128()),
            Literal::Str(string) => write!(f, "\"{}\"", string),
        }
    }
}

impl Display for BlockExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in &self.0 {
            let statement = statement.to_string();
            for line in statement.lines() {
                writeln!(f, "    {}", line)?;
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
        write!(f, "{}({})", self.func_name, args.join(", "))
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
        let fields = self
            .fields
            .iter()
            .map(|(ident, expr)| format!("{}: {}", ident, expr))
            .collect::<Vec<_>>();

        write!(f, "({} {{ {} }})", self.type_name, fields.join(", "))
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
            BinaryOpKind::Assign => write!(f, "="),
        }
    }
}

impl Display for ForExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "for {} in {} .. {} {}",
            self.identifier, self.start_range, self.end_range, self.block
        )
    }
}

impl Display for IfExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} {}", self.condition, self.consequence)?;
        if let Some(alternative) = &self.alternative {
            write!(f, " else {}", alternative)?;
        }
        Ok(())
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(attribute) = &self.attribute {
            writeln!(f, "{}", attribute)?;
        }

        let parameters = vecmap(&self.parameters, |(name, r#type, visibility)| {
            format!("{}: {} {}", name, visibility, r#type)
        });

        write!(
            f,
            "fn {}({}) -> {} {}",
            self.name,
            parameters.join(", "),
            self.return_type,
            self.body
        )
    }
}
