use std::{fmt::Display, rc::Rc};

use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    ast::{
        ArrayLiteral, AsTraitPath, AssignStatement, BlockExpression, CallExpression,
        CastExpression, ConstrainStatement, ConstructorExpression, Expression, ExpressionKind,
        ForBounds, ForLoopStatement, ForRange, GenericTypeArgs, IfExpression, IndexExpression,
        InfixExpression, LValue, Lambda, LetStatement, Literal, MemberAccessExpression,
        MethodCallExpression, Pattern, PrefixExpression, Statement, StatementKind, UnresolvedType,
        UnresolvedTypeData,
    },
    hir_def::traits::TraitConstraint,
    node_interner::{InternedStatementKind, NodeInterner},
    token::{Keyword, Token},
    Type,
};

use super::{
    value::{ExprValue, TypedExpr},
    Value,
};

pub(super) fn display_quoted(
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
        TokensPrettyPrinter { tokens, interner, indent }.fmt(f)?;
        writeln!(f)?;
        let indent = indent - 1;
        write!(f, "{}", " ".repeat(indent * 4))?;
        write!(f, "}}")
    }
}

struct TokensPrettyPrinter<'tokens, 'interner> {
    tokens: &'tokens [Token],
    interner: &'interner NodeInterner,
    indent: usize,
}

impl<'tokens, 'interner> Display for TokensPrettyPrinter<'tokens, 'interner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut token_printer = TokenPrettyPrinter::new(self.interner, self.indent);
        for token in self.tokens {
            token_printer.print(token, f)?;
        }

        // If the printer refrained from printing a token right away, this will make it do it
        token_printer.print(&Token::EOF, f)?;

        Ok(())
    }
}

pub(super) fn tokens_to_string(tokens: Rc<Vec<Token>>, interner: &NodeInterner) -> String {
    let tokens: Vec<Token> = tokens.iter().cloned().collect();
    TokensPrettyPrinter { tokens: &tokens, interner, indent: 0 }.to_string()
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
    last_was_alphanumeric: bool,
    last_was_right_brace: bool,
    last_was_semicolon: bool,
    last_was_op: bool,
}

impl<'interner> TokenPrettyPrinter<'interner> {
    fn new(interner: &'interner NodeInterner, indent: usize) -> Self {
        Self {
            interner,
            indent,
            last_was_alphanumeric: false,
            last_was_right_brace: false,
            last_was_semicolon: false,
            last_was_op: false,
        }
    }

    fn print(&mut self, token: &Token, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let last_was_alphanumeric = self.last_was_alphanumeric;
        self.last_was_alphanumeric = false;

        let last_was_op = self.last_was_op;
        self.last_was_op = false;

        // After `}` we usually want a newline... but not always!
        if self.last_was_right_brace {
            self.last_was_right_brace = false;

            match token {
                Token::Keyword(Keyword::Else) => {
                    // If we have `} else` we don't want a newline
                    write!(f, " else")?;
                    self.last_was_alphanumeric = true;
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

        // If the last token was one of `+`, `-`, etc. and the current token is not `=`, we want a space
        // (we avoid outputting a space if the token is `=` a bit below)
        if last_was_op && !matches!(token, Token::Assign) {
            write!(f, " ")?;
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
            Token::Keyword(..)
            | Token::Ident(..)
            | Token::IntType(..)
            | Token::Int(..)
            | Token::Bool(..) => {
                if last_was_alphanumeric {
                    write!(f, " ")?;
                }
                self.last_was_alphanumeric = true;
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
            Token::Quote(tokens) => {
                if last_was_alphanumeric {
                    write!(f, " ")?;
                }
                let tokens = vecmap(&tokens.0, |spanned_token| spanned_token.clone().into_token());
                display_quoted(&tokens, self.indent, self.interner, f)
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
            | Token::Arrow => write!(f, " {token} "),
            Token::Assign => {
                if last_was_op {
                    write!(f, "{token} ")
                } else {
                    write!(f, " {token} ")
                }
            }
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Ampersand
            | Token::ShiftLeft
            | Token::ShiftRight => {
                self.last_was_op = true;
                write!(f, " {token}")
            }
            Token::LeftParen
            | Token::RightParen
            | Token::LeftBracket
            | Token::RightBracket
            | Token::Dot
            | Token::DoubleColon
            | Token::DoubleDot
            | Token::DoubleDotEqual
            | Token::Caret
            | Token::Pound
            | Token::Pipe
            | Token::Bang
            | Token::DollarSign => {
                write!(f, "{token}")
            }
            Token::Str(..)
            | Token::RawStr(..)
            | Token::FmtStr(..)
            | Token::Whitespace(_)
            | Token::LineComment(..)
            | Token::BlockComment(..)
            | Token::AttributeStart { .. }
            | Token::Invalid(_) => {
                if last_was_alphanumeric {
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

        self.last_was_alphanumeric = string.bytes().all(|byte| byte.is_ascii_alphanumeric());
        self.last_was_right_brace = string.ends_with('}');
        self.last_was_semicolon = string.ends_with(';');

        Ok(())
    }

    fn write_indent(&mut self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", " ".repeat(self.indent * 4))
    }
}

impl Value {
    pub fn display<'value, 'interner>(
        &'value self,
        interner: &'interner NodeInterner,
    ) -> ValuePrinter<'value, 'interner> {
        ValuePrinter { value: self, interner }
    }
}

pub struct ValuePrinter<'value, 'interner> {
    value: &'value Value,
    interner: &'interner NodeInterner,
}

impl<'value, 'interner> Display for ValuePrinter<'value, 'interner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Unit => write!(f, "()"),
            Value::Bool(value) => {
                let msg = if *value { "true" } else { "false" };
                write!(f, "{msg}")
            }
            Value::Field(value) => write!(f, "{value}"),
            Value::I8(value) => write!(f, "{value}"),
            Value::I16(value) => write!(f, "{value}"),
            Value::I32(value) => write!(f, "{value}"),
            Value::I64(value) => write!(f, "{value}"),
            Value::U1(value) => write!(f, "{value}"),
            Value::U8(value) => write!(f, "{value}"),
            Value::U16(value) => write!(f, "{value}"),
            Value::U32(value) => write!(f, "{value}"),
            Value::U64(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, "{value}"),
            Value::CtString(value) => write!(f, "{value}"),
            Value::FormatString(value, _) => write!(f, "{value}"),
            Value::Function(..) => write!(f, "(function)"),
            Value::Closure(..) => write!(f, "(closure)"),
            Value::Tuple(fields) => {
                let fields = vecmap(fields, |field| field.display(self.interner).to_string());
                write!(f, "({})", fields.join(", "))
            }
            Value::Struct(fields, typ) => {
                let typename = match typ.follow_bindings() {
                    Type::Struct(def, _) => def.borrow().name.to_string(),
                    other => other.to_string(),
                };
                let fields = vecmap(fields, |(name, value)| {
                    format!("{}: {}", name, value.display(self.interner))
                });
                write!(f, "{typename} {{ {} }}", fields.join(", "))
            }
            Value::Pointer(value, _) => write!(f, "&mut {}", value.borrow().display(self.interner)),
            Value::Array(values, _) => {
                let values = vecmap(values, |value| value.display(self.interner).to_string());
                write!(f, "[{}]", values.join(", "))
            }
            Value::Slice(values, _) => {
                let values = vecmap(values, |value| value.display(self.interner).to_string());
                write!(f, "&[{}]", values.join(", "))
            }
            Value::Quoted(tokens) => display_quoted(tokens, 0, self.interner, f),
            Value::StructDefinition(id) => {
                let def = self.interner.get_struct(*id);
                let def = def.borrow();
                write!(f, "{}", def.name)
            }
            Value::TraitConstraint(trait_id, generics) => {
                let trait_ = self.interner.get_trait(*trait_id);
                write!(f, "{}{generics}", trait_.name)
            }
            Value::TraitDefinition(trait_id) => {
                let trait_ = self.interner.get_trait(*trait_id);
                write!(f, "{}", trait_.name)
            }
            Value::TraitImpl(trait_impl_id) => {
                let trait_impl = self.interner.get_trait_implementation(*trait_impl_id);
                let trait_impl = trait_impl.borrow();

                let generic_string =
                    vecmap(&trait_impl.trait_generics, ToString::to_string).join(", ");
                let generic_string = if generic_string.is_empty() {
                    generic_string
                } else {
                    format!("<{}>", generic_string)
                };

                let where_clause = vecmap(&trait_impl.where_clause, |trait_constraint| {
                    display_trait_constraint(self.interner, trait_constraint)
                });
                let where_clause = where_clause.join(", ");
                let where_clause = if where_clause.is_empty() {
                    where_clause
                } else {
                    format!(" where {}", where_clause)
                };

                write!(
                    f,
                    "impl {}{} for {}{}",
                    trait_impl.ident, generic_string, trait_impl.typ, where_clause
                )
            }
            Value::FunctionDefinition(function_id) => {
                write!(f, "{}", self.interner.function_name(function_id))
            }
            Value::ModuleDefinition(module_id) => {
                if let Some(attributes) = self.interner.try_module_attributes(module_id) {
                    write!(f, "{}", &attributes.name)
                } else {
                    write!(f, "(crate root)")
                }
            }
            Value::Zeroed(typ) => write!(f, "(zeroed {typ})"),
            Value::Type(typ) => write!(f, "{}", typ),
            Value::Expr(ExprValue::Expression(expr)) => {
                let expr = remove_interned_in_expression_kind(self.interner, expr.clone());
                write!(f, "{}", expr)
            }
            Value::Expr(ExprValue::Statement(statement)) => {
                write!(f, "{}", remove_interned_in_statement_kind(self.interner, statement.clone()))
            }
            Value::Expr(ExprValue::LValue(lvalue)) => {
                write!(f, "{}", remove_interned_in_lvalue(self.interner, lvalue.clone()))
            }
            Value::Expr(ExprValue::Pattern(pattern)) => {
                write!(f, "{}", remove_interned_in_pattern(self.interner, pattern.clone()))
            }
            Value::TypedExpr(TypedExpr::ExprId(id)) => {
                let hir_expr = self.interner.expression(id);
                let expr = hir_expr.to_display_ast(self.interner, Span::default());
                write!(f, "{}", expr.kind)
            }
            Value::TypedExpr(TypedExpr::StmtId(id)) => {
                let hir_statement = self.interner.statement(id);
                let stmt = hir_statement.to_display_ast(self.interner, Span::default());
                write!(f, "{}", stmt.kind)
            }
            Value::UnresolvedType(typ) => {
                write!(f, "{}", remove_interned_in_unresolved_type_data(self.interner, typ.clone()))
            }
        }
    }
}

impl Token {
    pub fn display<'token, 'interner>(
        &'token self,
        interner: &'interner NodeInterner,
    ) -> TokenPrinter<'token, 'interner> {
        TokenPrinter { token: self, interner }
    }
}

pub struct TokenPrinter<'token, 'interner> {
    token: &'token Token,
    interner: &'interner NodeInterner,
}

impl<'token, 'interner> Display for TokenPrinter<'token, 'interner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.token {
            Token::QuotedType(id) => {
                write!(f, "{}", self.interner.get_quoted_type(*id))
            }
            Token::InternedExpr(id) => {
                let value = Value::expression(ExpressionKind::Interned(*id));
                value.display(self.interner).fmt(f)
            }
            Token::InternedStatement(id) => {
                let value = Value::statement(StatementKind::Interned(*id));
                value.display(self.interner).fmt(f)
            }
            Token::InternedLValue(id) => {
                let value = Value::lvalue(LValue::Interned(*id, Span::default()));
                value.display(self.interner).fmt(f)
            }
            Token::InternedUnresolvedTypeData(id) => {
                let value = Value::UnresolvedType(UnresolvedTypeData::Interned(*id));
                value.display(self.interner).fmt(f)
            }
            Token::InternedPattern(id) => {
                let value = Value::pattern(Pattern::Interned(*id, Span::default()));
                value.display(self.interner).fmt(f)
            }
            Token::UnquoteMarker(id) => {
                let value = Value::TypedExpr(TypedExpr::ExprId(*id));
                value.display(self.interner).fmt(f)
            }
            other => write!(f, "{other}"),
        }
    }
}

fn display_trait_constraint(interner: &NodeInterner, trait_constraint: &TraitConstraint) -> String {
    let trait_ = interner.get_trait(trait_constraint.trait_bound.trait_id);
    format!(
        "{}: {}{}",
        trait_constraint.typ, trait_.name, trait_constraint.trait_bound.trait_generics
    )
}

// Returns a new Expression where all Interned and Resolved expressions have been turned into non-interned ExpressionKind.
fn remove_interned_in_expression(interner: &NodeInterner, expr: Expression) -> Expression {
    Expression { kind: remove_interned_in_expression_kind(interner, expr.kind), span: expr.span }
}

// Returns a new ExpressionKind where all Interned and Resolved expressions have been turned into non-interned ExpressionKind.
fn remove_interned_in_expression_kind(
    interner: &NodeInterner,
    expr: ExpressionKind,
) -> ExpressionKind {
    match expr {
        ExpressionKind::Literal(literal) => {
            ExpressionKind::Literal(remove_interned_in_literal(interner, literal))
        }
        ExpressionKind::Block(block) => {
            let statements =
                vecmap(block.statements, |stmt| remove_interned_in_statement(interner, stmt));
            ExpressionKind::Block(BlockExpression { statements })
        }
        ExpressionKind::Prefix(prefix) => ExpressionKind::Prefix(Box::new(PrefixExpression {
            rhs: remove_interned_in_expression(interner, prefix.rhs),
            ..*prefix
        })),
        ExpressionKind::Index(index) => ExpressionKind::Index(Box::new(IndexExpression {
            collection: remove_interned_in_expression(interner, index.collection),
            index: remove_interned_in_expression(interner, index.index),
        })),
        ExpressionKind::Call(call) => ExpressionKind::Call(Box::new(CallExpression {
            func: Box::new(remove_interned_in_expression(interner, *call.func)),
            arguments: vecmap(call.arguments, |arg| remove_interned_in_expression(interner, arg)),
            ..*call
        })),
        ExpressionKind::MethodCall(call) => {
            ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                object: remove_interned_in_expression(interner, call.object),
                arguments: vecmap(call.arguments, |arg| {
                    remove_interned_in_expression(interner, arg)
                }),
                ..*call
            }))
        }
        ExpressionKind::Constructor(constructor) => {
            ExpressionKind::Constructor(Box::new(ConstructorExpression {
                fields: vecmap(constructor.fields, |(name, expr)| {
                    (name, remove_interned_in_expression(interner, expr))
                }),
                ..*constructor
            }))
        }
        ExpressionKind::MemberAccess(member_access) => {
            ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                lhs: remove_interned_in_expression(interner, member_access.lhs),
                ..*member_access
            }))
        }
        ExpressionKind::Cast(cast) => ExpressionKind::Cast(Box::new(CastExpression {
            lhs: remove_interned_in_expression(interner, cast.lhs),
            ..*cast
        })),
        ExpressionKind::Infix(infix) => ExpressionKind::Infix(Box::new(InfixExpression {
            lhs: remove_interned_in_expression(interner, infix.lhs),
            rhs: remove_interned_in_expression(interner, infix.rhs),
            ..*infix
        })),
        ExpressionKind::If(if_expr) => ExpressionKind::If(Box::new(IfExpression {
            condition: remove_interned_in_expression(interner, if_expr.condition),
            consequence: remove_interned_in_expression(interner, if_expr.consequence),
            alternative: if_expr
                .alternative
                .map(|alternative| remove_interned_in_expression(interner, alternative)),
        })),
        ExpressionKind::Variable(_) => expr,
        ExpressionKind::Tuple(expressions) => ExpressionKind::Tuple(vecmap(expressions, |expr| {
            remove_interned_in_expression(interner, expr)
        })),
        ExpressionKind::Lambda(lambda) => ExpressionKind::Lambda(Box::new(Lambda {
            body: remove_interned_in_expression(interner, lambda.body),
            ..*lambda
        })),
        ExpressionKind::Parenthesized(expr) => {
            ExpressionKind::Parenthesized(Box::new(remove_interned_in_expression(interner, *expr)))
        }
        ExpressionKind::Quote(_) => expr,
        ExpressionKind::Unquote(expr) => {
            ExpressionKind::Unquote(Box::new(remove_interned_in_expression(interner, *expr)))
        }
        ExpressionKind::Comptime(block, span) => {
            let statements =
                vecmap(block.statements, |stmt| remove_interned_in_statement(interner, stmt));
            ExpressionKind::Comptime(BlockExpression { statements }, span)
        }
        ExpressionKind::Unsafe(block, span) => {
            let statements =
                vecmap(block.statements, |stmt| remove_interned_in_statement(interner, stmt));
            ExpressionKind::Unsafe(BlockExpression { statements }, span)
        }
        ExpressionKind::AsTraitPath(mut path) => {
            path.typ = remove_interned_in_unresolved_type(interner, path.typ);
            path.trait_generics =
                remove_interned_in_generic_type_args(interner, path.trait_generics);
            ExpressionKind::AsTraitPath(path)
        }
        ExpressionKind::TypePath(mut path) => {
            path.typ = remove_interned_in_unresolved_type(interner, path.typ);
            path.turbofish = path
                .turbofish
                .map(|turbofish| remove_interned_in_generic_type_args(interner, turbofish));
            ExpressionKind::TypePath(path)
        }
        ExpressionKind::Resolved(id) => {
            let expr = interner.expression(&id);
            expr.to_display_ast(interner, Span::default()).kind
        }
        ExpressionKind::Interned(id) => {
            let expr = interner.get_expression_kind(id).clone();
            remove_interned_in_expression_kind(interner, expr)
        }
        ExpressionKind::Error => expr,
        ExpressionKind::InternedStatement(id) => remove_interned_in_statement_expr(interner, id),
    }
}

fn remove_interned_in_statement_expr(
    interner: &NodeInterner,
    id: InternedStatementKind,
) -> ExpressionKind {
    let expr = match interner.get_statement_kind(id).clone() {
        StatementKind::Expression(expr) | StatementKind::Semi(expr) => expr.kind,
        StatementKind::Interned(id) => remove_interned_in_statement_expr(interner, id),
        _ => ExpressionKind::Error,
    };
    remove_interned_in_expression_kind(interner, expr)
}

fn remove_interned_in_literal(interner: &NodeInterner, literal: Literal) -> Literal {
    match literal {
        Literal::Array(array_literal) => {
            Literal::Array(remove_interned_in_array_literal(interner, array_literal))
        }
        Literal::Slice(array_literal) => {
            Literal::Array(remove_interned_in_array_literal(interner, array_literal))
        }
        Literal::Bool(_)
        | Literal::Integer(_, _)
        | Literal::Str(_)
        | Literal::RawStr(_, _)
        | Literal::FmtStr(_, _)
        | Literal::Unit => literal,
    }
}

fn remove_interned_in_array_literal(
    interner: &NodeInterner,
    literal: ArrayLiteral,
) -> ArrayLiteral {
    match literal {
        ArrayLiteral::Standard(expressions) => {
            ArrayLiteral::Standard(vecmap(expressions, |expr| {
                remove_interned_in_expression(interner, expr)
            }))
        }
        ArrayLiteral::Repeated { repeated_element, length } => ArrayLiteral::Repeated {
            repeated_element: Box::new(remove_interned_in_expression(interner, *repeated_element)),
            length: Box::new(remove_interned_in_expression(interner, *length)),
        },
    }
}

// Returns a new Statement where all Interned statements have been turned into non-interned StatementKind.
fn remove_interned_in_statement(interner: &NodeInterner, statement: Statement) -> Statement {
    Statement {
        kind: remove_interned_in_statement_kind(interner, statement.kind),
        span: statement.span,
    }
}

// Returns a new StatementKind where all Interned statements have been turned into non-interned StatementKind.
fn remove_interned_in_statement_kind(
    interner: &NodeInterner,
    statement: StatementKind,
) -> StatementKind {
    match statement {
        StatementKind::Let(let_statement) => StatementKind::Let(LetStatement {
            pattern: remove_interned_in_pattern(interner, let_statement.pattern),
            expression: remove_interned_in_expression(interner, let_statement.expression),
            r#type: remove_interned_in_unresolved_type(interner, let_statement.r#type),
            ..let_statement
        }),
        StatementKind::Constrain(constrain) => StatementKind::Constrain(ConstrainStatement {
            arguments: vecmap(constrain.arguments, |expr| {
                remove_interned_in_expression(interner, expr)
            }),
            ..constrain
        }),
        StatementKind::Expression(expr) => {
            StatementKind::Expression(remove_interned_in_expression(interner, expr))
        }
        StatementKind::Assign(assign) => StatementKind::Assign(AssignStatement {
            lvalue: assign.lvalue,
            expression: remove_interned_in_expression(interner, assign.expression),
        }),
        StatementKind::For(for_loop) => StatementKind::For(ForLoopStatement {
            range: match for_loop.range {
                ForRange::Range(ForBounds { start, end, inclusive }) => {
                    ForRange::Range(ForBounds {
                        start: remove_interned_in_expression(interner, start),
                        end: remove_interned_in_expression(interner, end),
                        inclusive,
                    })
                }
                ForRange::Array(expr) => {
                    ForRange::Array(remove_interned_in_expression(interner, expr))
                }
            },
            block: remove_interned_in_expression(interner, for_loop.block),
            ..for_loop
        }),
        StatementKind::Loop(block) => {
            StatementKind::Loop(remove_interned_in_expression(interner, block))
        }
        StatementKind::Comptime(statement) => {
            StatementKind::Comptime(Box::new(remove_interned_in_statement(interner, *statement)))
        }
        StatementKind::Semi(expr) => {
            StatementKind::Semi(remove_interned_in_expression(interner, expr))
        }
        StatementKind::Interned(id) => {
            let statement = interner.get_statement_kind(id).clone();
            remove_interned_in_statement_kind(interner, statement)
        }
        StatementKind::Break | StatementKind::Continue | StatementKind::Error => statement,
    }
}

// Returns a new LValue where all Interned LValues have been turned into LValue.
fn remove_interned_in_lvalue(interner: &NodeInterner, lvalue: LValue) -> LValue {
    match lvalue {
        LValue::Ident(_) => lvalue,
        LValue::MemberAccess { object, field_name, span } => LValue::MemberAccess {
            object: Box::new(remove_interned_in_lvalue(interner, *object)),
            field_name,
            span,
        },
        LValue::Index { array, index, span } => LValue::Index {
            array: Box::new(remove_interned_in_lvalue(interner, *array)),
            index: remove_interned_in_expression(interner, index),
            span,
        },
        LValue::Dereference(lvalue, span) => {
            LValue::Dereference(Box::new(remove_interned_in_lvalue(interner, *lvalue)), span)
        }
        LValue::Interned(id, span) => {
            let lvalue = interner.get_lvalue(id, span);
            remove_interned_in_lvalue(interner, lvalue)
        }
    }
}

fn remove_interned_in_unresolved_type(
    interner: &NodeInterner,
    typ: UnresolvedType,
) -> UnresolvedType {
    UnresolvedType {
        typ: remove_interned_in_unresolved_type_data(interner, typ.typ),
        span: typ.span,
    }
}

fn remove_interned_in_unresolved_type_data(
    interner: &NodeInterner,
    typ: UnresolvedTypeData,
) -> UnresolvedTypeData {
    match typ {
        UnresolvedTypeData::Array(expr, typ) => UnresolvedTypeData::Array(
            expr,
            Box::new(remove_interned_in_unresolved_type(interner, *typ)),
        ),
        UnresolvedTypeData::Slice(typ) => {
            UnresolvedTypeData::Slice(Box::new(remove_interned_in_unresolved_type(interner, *typ)))
        }
        UnresolvedTypeData::FormatString(expr, typ) => UnresolvedTypeData::FormatString(
            expr,
            Box::new(remove_interned_in_unresolved_type(interner, *typ)),
        ),
        UnresolvedTypeData::Parenthesized(typ) => UnresolvedTypeData::Parenthesized(Box::new(
            remove_interned_in_unresolved_type(interner, *typ),
        )),
        UnresolvedTypeData::Named(path, generic_type_args, is_synthesized) => {
            UnresolvedTypeData::Named(
                path,
                remove_interned_in_generic_type_args(interner, generic_type_args),
                is_synthesized,
            )
        }
        UnresolvedTypeData::TraitAsType(path, generic_type_args) => {
            UnresolvedTypeData::TraitAsType(
                path,
                remove_interned_in_generic_type_args(interner, generic_type_args),
            )
        }
        UnresolvedTypeData::MutableReference(typ) => UnresolvedTypeData::MutableReference(
            Box::new(remove_interned_in_unresolved_type(interner, *typ)),
        ),
        UnresolvedTypeData::Tuple(types) => UnresolvedTypeData::Tuple(vecmap(types, |typ| {
            remove_interned_in_unresolved_type(interner, typ)
        })),
        UnresolvedTypeData::Function(arg_types, ret_type, env_type, unconstrained) => {
            UnresolvedTypeData::Function(
                vecmap(arg_types, |typ| remove_interned_in_unresolved_type(interner, typ)),
                Box::new(remove_interned_in_unresolved_type(interner, *ret_type)),
                Box::new(remove_interned_in_unresolved_type(interner, *env_type)),
                unconstrained,
            )
        }
        UnresolvedTypeData::AsTraitPath(as_trait_path) => {
            UnresolvedTypeData::AsTraitPath(Box::new(AsTraitPath {
                typ: remove_interned_in_unresolved_type(interner, as_trait_path.typ),
                trait_generics: remove_interned_in_generic_type_args(
                    interner,
                    as_trait_path.trait_generics,
                ),
                ..*as_trait_path
            }))
        }
        UnresolvedTypeData::Interned(id) => interner.get_unresolved_type_data(id).clone(),
        UnresolvedTypeData::FieldElement
        | UnresolvedTypeData::Integer(_, _)
        | UnresolvedTypeData::Bool
        | UnresolvedTypeData::Unit
        | UnresolvedTypeData::String(_)
        | UnresolvedTypeData::Resolved(_)
        | UnresolvedTypeData::Quoted(_)
        | UnresolvedTypeData::Expression(_)
        | UnresolvedTypeData::Unspecified
        | UnresolvedTypeData::Error => typ,
    }
}

fn remove_interned_in_generic_type_args(
    interner: &NodeInterner,
    args: GenericTypeArgs,
) -> GenericTypeArgs {
    GenericTypeArgs {
        ordered_args: vecmap(args.ordered_args, |typ| {
            remove_interned_in_unresolved_type(interner, typ)
        }),
        named_args: vecmap(args.named_args, |(name, typ)| {
            (name, remove_interned_in_unresolved_type(interner, typ))
        }),
        kinds: args.kinds,
    }
}

// Returns a new Pattern where all Interned Patterns have been turned into Pattern.
fn remove_interned_in_pattern(interner: &NodeInterner, pattern: Pattern) -> Pattern {
    match pattern {
        Pattern::Identifier(_) => pattern,
        Pattern::Mutable(pattern, span, is_synthesized) => Pattern::Mutable(
            Box::new(remove_interned_in_pattern(interner, *pattern)),
            span,
            is_synthesized,
        ),
        Pattern::Tuple(patterns, span) => Pattern::Tuple(
            vecmap(patterns, |pattern| remove_interned_in_pattern(interner, pattern)),
            span,
        ),
        Pattern::Struct(path, patterns, span) => {
            let patterns = vecmap(patterns, |(name, pattern)| {
                (name, remove_interned_in_pattern(interner, pattern))
            });
            Pattern::Struct(path, patterns, span)
        }
        Pattern::Interned(id, _) => interner.get_pattern(id).clone(),
    }
}
