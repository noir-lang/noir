use std::fmt::Display;

use fm::FileMap;
use iter_extended::vecmap;
use noirc_errors::{Location, reporter::line_and_column_from_span};

use crate::{
    Type,
    ast::{
        ArrayLiteral, AsTraitPath, AssignOpStatement, AssignStatement, BlockExpression,
        CallExpression, CastExpression, ConstrainExpression, ConstructorExpression, Expression,
        ExpressionKind, ForBounds, ForLoopStatement, ForRange, GenericTypeArgs, IfExpression,
        IndexExpression, InfixExpression, LValue, Lambda, LetStatement, Literal, LoopStatement,
        MatchExpression, MemberAccessExpression, MethodCallExpression, Pattern, PrefixExpression,
        Statement, StatementKind, UnresolvedType, UnresolvedTypeData, UnsafeExpression,
        WhileStatement,
    },
    hir::comptime::interpreter::builtin_helpers::fragments_to_bytes,
    hir_def::traits::TraitConstraint,
    node_interner::{InternedStatementKind, NodeInterner},
    token::{Keyword, LocatedToken, Token},
};

use super::{
    Value,
    value::{ExprValue, TypedExpr},
};

/// Format a [`Location`] as `Location("filename:line:column")` (pointing at the start of the
/// span), matching the format used by compiler error messages. Returns
/// `Location("unknown")` when the location is dummy or its file cannot be resolved.
fn display_location(location: &Location, files: &FileMap) -> String {
    const UNKNOWN: &str = r#"Location("unknown")"#;

    if location.is_dummy() {
        return UNKNOWN.to_string();
    }

    let Ok(path) = files.get_name(location.file) else {
        return UNKNOWN.to_string();
    };
    let Some(source) = files.get_file(location.file).map(|file| file.source()) else {
        return UNKNOWN.to_string();
    };

    let (line, column) = line_and_column_from_span(source, &location.span);
    format!(r#"Location("{path}:{line}:{column}")"#)
}

pub(super) fn display_quoted(
    tokens: &[LocatedToken],
    indent: usize,
    interner: &NodeInterner,
    files: &FileMap,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    if tokens.is_empty() {
        write!(f, "quote {{ }}")
    } else {
        writeln!(f, "quote {{")?;
        let indent = indent + 1;
        write!(f, "{}", " ".repeat(indent * 4))?;
        TokensPrettyPrinter { tokens, interner, files, indent, preserve_unquote_markers: false }
            .fmt(f)?;
        writeln!(f)?;
        let indent = indent - 1;
        write!(f, "{}", " ".repeat(indent * 4))?;
        write!(f, "}}")
    }
}

struct TokensPrettyPrinter<'tokens, 'interner> {
    tokens: &'tokens [LocatedToken],
    interner: &'interner NodeInterner,
    files: &'interner FileMap,
    indent: usize,
    preserve_unquote_markers: bool,
}

impl Display for TokensPrettyPrinter<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut token_printer = TokenPrettyPrinter::new(
            self.interner,
            self.files,
            self.indent,
            self.preserve_unquote_markers,
        );
        for token in self.tokens {
            token_printer.print(token.token(), f)?;
        }

        // If the printer refrained from printing a token right away, this will make it do it
        token_printer.print(&Token::EOF, f)?;

        Ok(())
    }
}

pub fn tokens_to_string(
    tokens: &[LocatedToken],
    interner: &NodeInterner,
    files: &FileMap,
) -> String {
    tokens_to_string_with_indent(tokens, 0, false, interner, files)
}

pub fn tokens_to_string_with_indent(
    tokens: &[LocatedToken],
    indent: usize,
    preserve_unquote_markers: bool,
    interner: &NodeInterner,
    files: &FileMap,
) -> String {
    TokensPrettyPrinter { tokens, interner, files, indent, preserve_unquote_markers }.to_string()
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
    files: &'interner FileMap,
    indent: usize,
    preserve_unquote_markers: bool,
    /// Determines whether the last outputted byte was alphanumeric.
    /// This is used to add a space after the last token and before another token
    /// that starts with an alphanumeric byte.
    last_was_alphanumeric: bool,
    last_was_right_brace: bool,
    last_was_semicolon: bool,
    last_was_op: bool,
}

impl<'interner> TokenPrettyPrinter<'interner> {
    fn new(
        interner: &'interner NodeInterner,
        files: &'interner FileMap,
        indent: usize,
        preserve_unquote_markers: bool,
    ) -> Self {
        Self {
            interner,
            files,
            indent,
            preserve_unquote_markers,
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
            Token::QuotedType(id) => {
                let value = Value::Type(self.interner.get_quoted_type(*id).clone());
                self.print_value(&value, last_was_alphanumeric, f)
            }
            Token::InternedExpr(id) => {
                let value = Value::expression(ExpressionKind::Interned(*id));
                self.print_value(&value, last_was_alphanumeric, f)
            }
            Token::InternedStatement(id) => {
                let value = Value::statement(StatementKind::Interned(*id));
                self.print_value(&value, last_was_alphanumeric, f)
            }
            Token::InternedLValue(id) => {
                let value = Value::lvalue(LValue::Interned(*id, Location::dummy()));
                self.print_value(&value, last_was_alphanumeric, f)
            }
            Token::InternedUnresolvedTypeData(id) => {
                let value = Value::UnresolvedType(UnresolvedTypeData::Interned(*id));
                self.print_value(&value, last_was_alphanumeric, f)
            }
            Token::InternedPattern(id) => {
                let value = Value::pattern(Pattern::Interned(*id, Location::dummy()));
                self.print_value(&value, last_was_alphanumeric, f)
            }
            Token::InternedCrate(_) => write!(f, "$crate"),
            Token::UnquoteMarker(id) => {
                let value = Value::TypedExpr(TypedExpr::ExprId(*id));
                let last_was_alphanumeric = if self.preserve_unquote_markers {
                    if last_was_alphanumeric {
                        write!(f, " ")?;
                    }
                    write!(f, "$")?;
                    false
                } else {
                    last_was_alphanumeric
                };
                self.print_value(&value, last_was_alphanumeric, f)
            }
            Token::Keyword(..) | Token::Ident(..) | Token::Int(..) | Token::Bool(..) => {
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
                // Saturate the decrement so an unmatched right brace (e.g. a malformed quoted
                // token stream being formatted for a recoverable parse-error diagnostic) does not
                // underflow the `usize` indent counter and panic the compiler.
                self.indent = self.indent.saturating_sub(1);
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
                display_quoted(&tokens.0, self.indent, self.interner, self.files, f)
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
            | Token::FatArrow
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
            | Token::ShiftRight
            | Token::LogicalAnd => {
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
            | Token::At
            | Token::Backslash
            | Token::DollarSign => {
                write!(f, "{token}")
            }
            Token::Str(..)
            | Token::RawStr(..)
            | Token::FmtStr(..)
            | Token::Whitespace(_)
            | Token::BlockComment(..)
            | Token::AttributeStart { .. }
            | Token::Invalid(_) => {
                if last_was_alphanumeric {
                    write!(f, " ")?;
                }
                write!(f, "{token}")
            }
            Token::LineComment(..) => {
                writeln!(f, "{token}")?;
                self.write_indent(f)
            }
            Token::EOF => Ok(()),
        }
    }

    fn print_value(
        &mut self,
        value: &Value,
        last_was_alphanumeric: bool,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let string = value.display(self.interner, self.files).to_string();
        if string.is_empty() {
            return Ok(());
        }

        if last_was_alphanumeric && string.bytes().next().unwrap().is_ascii_alphanumeric() {
            write!(f, " ")?;
        }

        for (index, line) in string.lines().enumerate() {
            if index > 0 {
                writeln!(f)?;
                self.write_indent(f)?;
            }
            line.fmt(f)?;
        }

        self.last_was_alphanumeric = string.bytes().last().unwrap().is_ascii_alphanumeric();
        self.last_was_right_brace = string.ends_with('}');
        self.last_was_semicolon = string.ends_with(';');

        Ok(())
    }

    fn write_indent(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", " ".repeat(self.indent * 4))
    }
}

impl Value {
    pub fn display<'value, 'interner>(
        &'value self,
        interner: &'interner NodeInterner,
        files: &'interner FileMap,
    ) -> ValuePrinter<'value, 'interner> {
        ValuePrinter { value: self, interner, files }
    }
}

pub struct ValuePrinter<'value, 'interner> {
    value: &'value Value,
    interner: &'interner NodeInterner,
    files: &'interner FileMap,
}

impl Display for ValuePrinter<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = value_to_bytes(self.value, self.interner, self.files);
        write!(f, "{}", String::from_utf8_lossy(&bytes))
    }
}

/// Renders a comptime value to bytes.
pub(crate) fn value_to_bytes(value: &Value, interner: &NodeInterner, files: &FileMap) -> Vec<u8> {
    let mut result = Vec::new();
    write_value_bytes(&mut result, value, interner, files);
    result
}

fn write_value_bytes(out: &mut Vec<u8>, value: &Value, interner: &NodeInterner, files: &FileMap) {
    // Append a value that cannot contain non-UTF-8 bytes, formatted through its `Display` impl.
    macro_rules! push_display {
        ($($arg:tt)*) => {
            out.extend_from_slice(format!($($arg)*).as_bytes())
        };
    }

    match value {
        Value::Unit => push_display!("()"),
        Value::Bool(value) => out.extend_from_slice(if *value { b"true" } else { b"false" }),
        Value::Integer(int) => push_display!("{int}"),
        Value::String(bytes) | Value::CtString(bytes) => {
            out.extend_from_slice(bytes);
        }
        Value::FormatString(fragments, _, _) => {
            out.extend_from_slice(&fragments_to_bytes(fragments, interner, files));
        }
        Value::Function(..) => push_display!("(function)"),
        Value::Closure(..) => push_display!("(closure)"),
        Value::Tuple(fields) => {
            out.push(b'(');
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    out.extend_from_slice(b", ");
                }
                write_value_bytes(out, &field.borrow(), interner, files);
            }
            if fields.len() == 1 {
                out.push(b',');
            }
            out.push(b')');
        }
        Value::Struct(fields, typ) => {
            let data_type = match typ.follow_bindings() {
                Type::DataType(def, _) => def,
                other => unreachable!("Expected data type, found {other}"),
            };
            let data_type = data_type.borrow();
            out.extend_from_slice(data_type.name.to_string().as_bytes());
            out.extend_from_slice(b" { ");

            // Display fields in the order they are defined in the struct.
            // Some fields might not be there if they were missing in the constructor.
            let mut first = true;
            for field in data_type.fields_raw().unwrap() {
                let name = field.name.as_string();
                if let Some(value) = fields.get(name) {
                    if !first {
                        out.extend_from_slice(b", ");
                    }
                    first = false;
                    out.extend_from_slice(name.as_bytes());
                    out.extend_from_slice(b": ");
                    write_value_bytes(out, &value.borrow(), interner, files);
                }
            }
            out.extend_from_slice(b" }");
        }
        Value::Enum(tag, args, typ) => match typ.follow_bindings_shallow().as_ref() {
            Type::DataType(def, _) => {
                let def = def.borrow();
                let variant = def.variant_at(*tag);
                if variant.is_function {
                    push_display!("{}::{}(", def.name, variant.name);
                    for (index, arg) in args.iter().enumerate() {
                        if index > 0 {
                            out.extend_from_slice(b", ");
                        }
                        write_value_bytes(out, arg, interner, files);
                    }
                    out.push(b')');
                } else {
                    push_display!("{}::{}", def.name, variant.name);
                }
            }
            other => push_display!("{other}(args)"),
        },
        Value::Pointer(value, _, mutable) => {
            out.extend_from_slice(if *mutable { b"&mut " } else { b"&" });
            write_value_bytes(out, &value.borrow(), interner, files);
        }
        Value::Array(values, _) => {
            out.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.extend_from_slice(b", ");
                }
                write_value_bytes(out, value, interner, files);
            }
            out.push(b']');
        }
        Value::Vector(values, _) => {
            out.extend_from_slice(b"@[");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.extend_from_slice(b", ");
                }
                write_value_bytes(out, value, interner, files);
            }
            out.push(b']');
        }
        Value::Quoted(tokens) => {
            out.extend_from_slice(QuotedPrinter { tokens, interner, files }.to_string().as_bytes());
        }
        Value::TypeDefinition(id) => {
            let def = interner.get_type(*id);
            let def = def.borrow();
            push_display!("{}", def.name);
        }
        Value::TraitConstraint(trait_id, generics) => {
            let trait_ = interner.get_trait(*trait_id);
            push_display!("{}{generics}", trait_.name);
        }
        Value::TraitDefinition(trait_id) => {
            let trait_ = interner.get_trait(*trait_id);
            push_display!("{}", trait_.name);
        }
        Value::TraitImpl(trait_impl_id) => {
            let trait_impl = interner.get_trait_implementation(*trait_impl_id);
            let trait_impl = trait_impl.borrow();
            let ordered_generics = interner.get_ordered_generics_for_impl(*trait_impl_id);

            let generic_string = vecmap(ordered_generics, ToString::to_string).join(", ");
            let generic_string = if generic_string.is_empty() {
                generic_string
            } else {
                format!("<{generic_string}>")
            };

            let where_clause = vecmap(&trait_impl.where_clause, |trait_constraint| {
                display_trait_constraint(interner, trait_constraint)
            });
            let where_clause = where_clause.join(", ");
            let where_clause = if where_clause.is_empty() {
                where_clause
            } else {
                format!(" where {where_clause}")
            };

            push_display!(
                "impl {}{} for {}{}",
                trait_impl.ident,
                generic_string,
                trait_impl.typ,
                where_clause
            );
        }
        Value::FunctionDefinition(function_id) => {
            push_display!("{}", interner.function_name(function_id));
        }
        Value::ModuleDefinition(module_id) => {
            if let Some(attributes) = interner.try_module_attributes(*module_id) {
                push_display!("{}", attributes.name);
            } else {
                push_display!("(crate root)");
            }
        }
        Value::Zeroed(typ) => push_display!("(zeroed {typ})"),
        Value::Type(typ) => push_display!("{typ}"),
        Value::Expr(expr) => match expr.as_ref() {
            ExprValue::Expression(expr) => {
                let expr = remove_interned_in_expression_kind(interner, expr.clone());
                push_display!("{expr}");
            }
            ExprValue::Statement(statement) => {
                push_display!("{}", remove_interned_in_statement_kind(interner, statement.clone()));
            }
            ExprValue::LValue(lvalue) => {
                push_display!("{}", remove_interned_in_lvalue(interner, lvalue.clone()));
            }
            ExprValue::Pattern(pattern) => {
                push_display!("{}", remove_interned_in_pattern(interner, pattern.clone()));
            }
        },
        Value::TypedExpr(TypedExpr::ExprId(id)) => {
            let hir_expr = interner.expression(id);
            let expr = hir_expr.to_display_ast(interner, Location::dummy());
            push_display!("{}", expr.kind);
        }
        Value::TypedExpr(TypedExpr::StmtId(id)) => {
            let hir_statement = interner.statement(id);
            let stmt = hir_statement.to_display_ast(interner, Location::dummy());
            push_display!("{}", stmt.kind);
        }
        Value::UnresolvedType(typ) => {
            push_display!("{}", remove_interned_in_unresolved_type_data(interner, typ.clone()));
        }
        Value::Location(location) => {
            push_display!("{}", display_location(location, files));
        }
    }
}

/// Renders a quoted value's tokens (with the surrounding `quote { ... }`) so it can be appended
/// to a byte buffer. Token output is always valid UTF-8.
struct QuotedPrinter<'a> {
    tokens: &'a [LocatedToken],
    interner: &'a NodeInterner,
    files: &'a FileMap,
}

impl Display for QuotedPrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_quoted(self.tokens, 0, self.interner, self.files, f)
    }
}

impl Token {
    pub fn display<'token, 'interner>(
        &'token self,
        interner: &'interner NodeInterner,
        files: &'interner FileMap,
    ) -> TokenPrinter<'token, 'interner> {
        TokenPrinter { token: self, interner, files }
    }
}

pub struct TokenPrinter<'token, 'interner> {
    token: &'token Token,
    interner: &'interner NodeInterner,
    files: &'interner FileMap,
}

impl Display for TokenPrinter<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.token {
            Token::QuotedType(id) => {
                write!(f, "{}", self.interner.get_quoted_type(*id))
            }
            Token::InternedExpr(id) => {
                let value = Value::expression(ExpressionKind::Interned(*id));
                value.display(self.interner, self.files).fmt(f)
            }
            Token::InternedStatement(id) => {
                let value = Value::statement(StatementKind::Interned(*id));
                value.display(self.interner, self.files).fmt(f)
            }
            Token::InternedLValue(id) => {
                let value = Value::lvalue(LValue::Interned(*id, Location::dummy()));
                value.display(self.interner, self.files).fmt(f)
            }
            Token::InternedUnresolvedTypeData(id) => {
                let value = Value::UnresolvedType(UnresolvedTypeData::Interned(*id));
                value.display(self.interner, self.files).fmt(f)
            }
            Token::InternedPattern(id) => {
                let value = Value::pattern(Pattern::Interned(*id, Location::dummy()));
                value.display(self.interner, self.files).fmt(f)
            }
            Token::UnquoteMarker(id) => {
                let value = Value::TypedExpr(TypedExpr::ExprId(*id));
                value.display(self.interner, self.files).fmt(f)
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
    Expression {
        kind: remove_interned_in_expression_kind(interner, expr.kind),
        location: expr.location,
    }
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
        ExpressionKind::Constrain(constrain) => ExpressionKind::Constrain(ConstrainExpression {
            arguments: vecmap(constrain.arguments, |expr| {
                remove_interned_in_expression(interner, expr)
            }),
            ..constrain
        }),
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
        ExpressionKind::Match(match_expr) => ExpressionKind::Match(Box::new(MatchExpression {
            expression: remove_interned_in_expression(interner, match_expr.expression),
            rules: vecmap(match_expr.rules, |(pattern, branch)| {
                let pattern = remove_interned_in_expression(interner, pattern);
                let branch = remove_interned_in_expression(interner, branch);
                (pattern, branch)
            }),
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
        ExpressionKind::Unsafe(UnsafeExpression { block, unsafe_keyword_location }) => {
            let statements =
                vecmap(block.statements, |stmt| remove_interned_in_statement(interner, stmt));
            ExpressionKind::Unsafe(UnsafeExpression {
                block: BlockExpression { statements },
                unsafe_keyword_location,
            })
        }
        ExpressionKind::AsTraitPath(mut path) => {
            path.typ = remove_interned_in_unresolved_type(interner, path.typ);
            path.trait_generics =
                remove_interned_in_generic_type_args(interner, path.trait_generics);
            path.turbofish = path
                .turbofish
                .map(|turbofish| remove_interned_in_generic_type_args(interner, turbofish));
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
            expr.to_display_ast(interner, Location::dummy()).kind
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
        Literal::Vector(array_literal) => {
            Literal::Array(remove_interned_in_array_literal(interner, array_literal))
        }
        Literal::Bool(_)
        | Literal::Integer(..)
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
        location: statement.location,
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
            r#type: remove_interned_in_option_unresolved_type(interner, let_statement.r#type),
            ..let_statement
        }),
        StatementKind::Expression(expr) => {
            StatementKind::Expression(remove_interned_in_expression(interner, expr))
        }
        StatementKind::Assign(assign) => StatementKind::Assign(AssignStatement {
            lvalue: assign.lvalue,
            expression: remove_interned_in_expression(interner, assign.expression),
        }),
        StatementKind::AssignOp(assign_op) => StatementKind::AssignOp(AssignOpStatement {
            lvalue: assign_op.lvalue,
            op: assign_op.op,
            expression: remove_interned_in_expression(interner, assign_op.expression),
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
        StatementKind::Loop(loop_) => StatementKind::Loop(LoopStatement {
            body: remove_interned_in_expression(interner, loop_.body),
            loop_keyword_location: loop_.loop_keyword_location,
        }),
        StatementKind::While(while_) => StatementKind::While(WhileStatement {
            condition: remove_interned_in_expression(interner, while_.condition),
            body: remove_interned_in_expression(interner, while_.body),
            while_keyword_location: while_.while_keyword_location,
        }),
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
        LValue::Path(_) => lvalue,
        LValue::MemberAccess { object, field_name, location: span } => LValue::MemberAccess {
            object: Box::new(remove_interned_in_lvalue(interner, *object)),
            field_name,
            location: span,
        },
        LValue::Index { array, index, location: span } => LValue::Index {
            array: Box::new(remove_interned_in_lvalue(interner, *array)),
            index: remove_interned_in_expression(interner, index),
            location: span,
        },
        LValue::Dereference(expr, span) => {
            LValue::Dereference(Box::new(remove_interned_in_expression(interner, *expr)), span)
        }
        LValue::Interned(id, span) => {
            let lvalue = interner.get_lvalue(id, span);
            remove_interned_in_lvalue(interner, lvalue)
        }
    }
}

fn remove_interned_in_option_unresolved_type(
    interner: &NodeInterner,
    typ: Option<UnresolvedType>,
) -> Option<UnresolvedType> {
    typ.map(|typ| remove_interned_in_unresolved_type(interner, typ))
}

fn remove_interned_in_unresolved_type(
    interner: &NodeInterner,
    typ: UnresolvedType,
) -> UnresolvedType {
    UnresolvedType {
        typ: remove_interned_in_unresolved_type_data(interner, typ.typ),
        location: typ.location,
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
        UnresolvedTypeData::Vector(typ) => {
            UnresolvedTypeData::Vector(Box::new(remove_interned_in_unresolved_type(interner, *typ)))
        }
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
        UnresolvedTypeData::Reference(typ, mutable) => UnresolvedTypeData::Reference(
            Box::new(remove_interned_in_unresolved_type(interner, *typ)),
            mutable,
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
                turbofish: as_trait_path
                    .turbofish
                    .map(|turbofish| remove_interned_in_generic_type_args(interner, turbofish)),
                ..*as_trait_path
            }))
        }
        UnresolvedTypeData::Interned(id) => interner.get_unresolved_type_data(id).clone(),
        UnresolvedTypeData::Unit
        | UnresolvedTypeData::Resolved(_)
        | UnresolvedTypeData::Expression(_)
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
        Pattern::Mutable(pattern, location, is_synthesized) => Pattern::Mutable(
            Box::new(remove_interned_in_pattern(interner, *pattern)),
            location,
            is_synthesized,
        ),
        Pattern::Tuple(patterns, location) => Pattern::Tuple(
            vecmap(patterns, |pattern| remove_interned_in_pattern(interner, pattern)),
            location,
        ),
        Pattern::Struct(path, patterns, location) => {
            let patterns = vecmap(patterns, |(name, pattern)| {
                (name, remove_interned_in_pattern(interner, pattern))
            });
            Pattern::Struct(path, patterns, location)
        }
        Pattern::Parenthesized(pattern, location) => Pattern::Parenthesized(
            Box::new(remove_interned_in_pattern(interner, *pattern)),
            location,
        ),
        Pattern::Interned(id, _) => interner.get_pattern(id).clone(),
    }
}

#[cfg(test)]
mod tests {
    use fm::FileMap;
    use noirc_errors::Location;

    use crate::node_interner::NodeInterner;
    use crate::token::{LocatedToken, Token};

    use super::tokens_to_string;

    #[test]
    fn tokens_to_string_handles_unmatched_right_brace() {
        let interner = NodeInterner::default();
        let files = FileMap::default();
        let tokens = [LocatedToken::new(Token::RightBrace, Location::dummy())];

        // An unmatched right brace must not underflow the indent counter and panic; it should
        // simply be rendered as a closing brace.
        let result = tokens_to_string(&tokens, &interner, &files);
        assert_eq!(result.trim(), "}");
    }
}
