use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Located, Location, Span};

use crate::NamedGeneric;
use crate::ast::{
    ArrayLiteral, AssignStatement, BlockExpression, CallExpression, CastExpression, ConstrainKind,
    ConstructorExpression, ExpressionKind, ForLoopStatement, ForRange, GenericTypeArgs, Ident,
    IfExpression, IndexExpression, InfixExpression, LValue, Lambda, Literal, LoopStatement,
    MatchExpression, MemberAccessExpression, Path, PathSegment, Pattern, PrefixExpression,
    UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression, UnsafeExpression, WhileStatement,
};
use crate::ast::{ConstrainExpression, Expression, Statement, StatementKind};
use crate::hir_def::expr::{
    Constructor, HirArrayLiteral, HirBlockExpression, HirExpression, HirIdent, HirLiteral, HirMatch,
};
use crate::hir_def::stmt::{HirLValue, HirPattern, HirStatement};
use crate::hir_def::types::{Type, TypeBinding};
use crate::node_interner::{DefinitionId, ExprId, NodeInterner, StmtId};

// TODO:
// - Full path for idents & types
// - Assert/AssertEq information lost
// - The type name span is lost in constructor patterns & expressions
// - All type spans are lost
// - Type::TypeVariable has no equivalent in the Ast

impl HirStatement {
    pub fn to_display_ast(&self, interner: &NodeInterner, location: Location) -> Statement {
        let kind = match self {
            HirStatement::Let(let_stmt) => {
                let pattern = let_stmt.pattern.to_display_ast(interner);
                let r#type = Some(interner.id_type(let_stmt.expression).to_display_ast());
                let expression = let_stmt.expression.to_display_ast(interner);
                StatementKind::new_let(pattern, r#type, expression, let_stmt.attributes.clone())
            }
            HirStatement::Assign(assign) => StatementKind::Assign(AssignStatement {
                lvalue: assign.lvalue.to_display_ast(interner),
                expression: assign.expression.to_display_ast(interner),
            }),
            HirStatement::For(for_stmt) => StatementKind::For(ForLoopStatement {
                identifier: for_stmt.identifier.to_display_ast(interner),
                range: ForRange::range(
                    for_stmt.start_range.to_display_ast(interner),
                    for_stmt.end_range.to_display_ast(interner),
                ),
                block: for_stmt.block.to_display_ast(interner),
                location,
            }),
            HirStatement::Loop(block) => StatementKind::Loop(LoopStatement {
                body: block.to_display_ast(interner),
                loop_keyword_location: location,
            }),
            HirStatement::While(condition, block) => StatementKind::While(WhileStatement {
                condition: condition.to_display_ast(interner),
                body: block.to_display_ast(interner),
                while_keyword_location: location,
            }),
            HirStatement::Break => StatementKind::Break,
            HirStatement::Continue => StatementKind::Continue,
            HirStatement::Expression(expr) => {
                StatementKind::Expression(expr.to_display_ast(interner))
            }
            HirStatement::Semi(expr) => StatementKind::Semi(expr.to_display_ast(interner)),
            HirStatement::Error => StatementKind::Error,
            HirStatement::Comptime(statement) => {
                StatementKind::Comptime(Box::new(statement.to_display_ast(interner)))
            }
        };

        Statement { kind, location }
    }
}

impl StmtId {
    /// Convert to AST for display (some details lost)
    pub fn to_display_ast(self, interner: &NodeInterner) -> Statement {
        let statement = interner.statement(&self);
        let location = interner.statement_location(self);

        statement.to_display_ast(interner, location)
    }
}

impl HirExpression {
    /// Convert to AST for display (some details lost)
    pub fn to_display_ast(&self, interner: &NodeInterner, location: Location) -> Expression {
        let kind = match self {
            HirExpression::Ident(ident, generics) => {
                ident.to_display_expr(interner, generics, location)
            }
            HirExpression::Literal(HirLiteral::Array(array)) => {
                let array = array.to_display_ast(interner, location);
                ExpressionKind::Literal(Literal::Array(array))
            }
            HirExpression::Literal(HirLiteral::Vector(array)) => {
                let array = array.to_display_ast(interner, location);
                ExpressionKind::Literal(Literal::Vector(array))
            }
            HirExpression::Literal(HirLiteral::Bool(value)) => {
                ExpressionKind::Literal(Literal::Bool(*value))
            }
            HirExpression::Literal(HirLiteral::Integer(value)) => {
                // Losing the integer suffix information here, but this should just be for
                // displaying these values anyway
                ExpressionKind::Literal(Literal::Integer(*value, None))
            }
            HirExpression::Literal(HirLiteral::Str(string)) => {
                ExpressionKind::Literal(Literal::Str(string.clone()))
            }
            HirExpression::Literal(HirLiteral::FmtStr(fragments, _exprs, length)) => {
                // TODO: Is throwing away the exprs here valid?
                ExpressionKind::Literal(Literal::FmtStr(fragments.clone(), *length))
            }
            HirExpression::Literal(HirLiteral::Unit) => ExpressionKind::Literal(Literal::Unit),
            HirExpression::Block(expr) => ExpressionKind::Block(expr.to_display_ast(interner)),
            HirExpression::Prefix(prefix) => ExpressionKind::Prefix(Box::new(PrefixExpression {
                operator: prefix.operator,
                rhs: prefix.rhs.to_display_ast(interner),
            })),
            HirExpression::Infix(infix) => ExpressionKind::Infix(Box::new(InfixExpression {
                lhs: infix.lhs.to_display_ast(interner),
                operator: Located::from(infix.operator.location, infix.operator.kind),
                rhs: infix.rhs.to_display_ast(interner),
            })),
            HirExpression::Index(index) => ExpressionKind::Index(Box::new(IndexExpression {
                collection: index.collection.to_display_ast(interner),
                index: index.index.to_display_ast(interner),
            })),
            HirExpression::Constructor(constructor) => {
                let type_name = constructor.r#type.borrow().name.to_string();
                let type_name = Path::from_single(type_name, location);
                let fields = vecmap(constructor.fields.clone(), |(name, expr): (Ident, ExprId)| {
                    (name, expr.to_display_ast(interner))
                });

                ExpressionKind::Constructor(Box::new(ConstructorExpression {
                    typ: UnresolvedType::from_path(type_name),
                    fields,
                }))
            }
            HirExpression::MemberAccess(access) => {
                ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                    lhs: access.lhs.to_display_ast(interner),
                    rhs: access.rhs.clone(),
                }))
            }
            HirExpression::Call(call) => {
                let func = Box::new(call.func.to_display_ast(interner));
                let arguments = vecmap(call.arguments.clone(), |arg| arg.to_display_ast(interner));
                let is_macro_call = false;
                ExpressionKind::Call(Box::new(CallExpression { func, arguments, is_macro_call }))
            }
            HirExpression::Constrain(constrain) => {
                let expr = constrain.0.to_display_ast(interner);
                let mut arguments = vec![expr];
                if let Some(message) = constrain.2 {
                    arguments.push(message.to_display_ast(interner));
                }

                // TODO: Find difference in usage between Assert & AssertEq
                ExpressionKind::Constrain(ConstrainExpression {
                    kind: ConstrainKind::Assert,
                    arguments,
                    location,
                })
            }
            HirExpression::Cast(cast) => {
                let lhs = cast.lhs.to_display_ast(interner);
                let r#type = cast.r#type.to_display_ast();
                ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type }))
            }
            HirExpression::If(if_expr) => ExpressionKind::If(Box::new(IfExpression {
                condition: if_expr.condition.to_display_ast(interner),
                consequence: if_expr.consequence.to_display_ast(interner),
                alternative: if_expr.alternative.map(|expr| expr.to_display_ast(interner)),
            })),
            HirExpression::Match(match_expr) => match_expr.to_display_ast(interner, location),
            HirExpression::Tuple(fields) => {
                ExpressionKind::Tuple(vecmap(fields, |field| field.to_display_ast(interner)))
            }
            HirExpression::Lambda(lambda) => {
                let parameters = vecmap(lambda.parameters.clone(), |(pattern, typ)| {
                    (pattern.to_display_ast(interner), Some(typ.to_display_ast()))
                });
                let return_type = Some(lambda.return_type.to_display_ast());
                let body = lambda.body.to_display_ast(interner);
                let unconstrained = lambda.unconstrained;
                ExpressionKind::Lambda(Box::new(Lambda {
                    parameters,
                    return_type,
                    body,
                    unconstrained,
                }))
            }
            HirExpression::Error => ExpressionKind::Error,
            HirExpression::Unsafe(block) => ExpressionKind::Unsafe(UnsafeExpression {
                block: block.to_display_ast(interner),
                unsafe_keyword_location: location,
            }),
            HirExpression::Quote(block) => ExpressionKind::Quote(block.clone()),

            // A macro was evaluated here: return the quoted result
            HirExpression::Unquote(block) => ExpressionKind::Quote(block.clone()),

            // Convert this back into a function call `Enum::Foo(args)`
            HirExpression::EnumConstructor(constructor) => {
                let typ = constructor.r#type.borrow();
                let variant = &typ.variant_at(constructor.variant_index);
                let segment1 = PathSegment { ident: typ.name.clone(), location, generics: None };
                let segment2 =
                    PathSegment { ident: variant.name.clone(), location, generics: None };
                let path = Path::plain(vec![segment1, segment2], location);
                let func = Box::new(Expression::new(ExpressionKind::Variable(path), location));
                let arguments = vecmap(&constructor.arguments, |arg| arg.to_display_ast(interner));
                let call = CallExpression { func, arguments, is_macro_call: false };
                ExpressionKind::Call(Box::new(call))
            }
        };

        Expression::new(kind, location)
    }
}

impl HirMatch {
    fn to_display_ast(&self, interner: &NodeInterner, location: Location) -> ExpressionKind {
        match self {
            HirMatch::Success(expr) => expr.to_display_ast(interner).kind,
            HirMatch::Failure { .. } => ExpressionKind::Error,
            HirMatch::Guard { cond, body, otherwise } => {
                let condition = cond.to_display_ast(interner);
                let consequence = body.to_display_ast(interner);
                let alternative =
                    Some(Expression::new(otherwise.to_display_ast(interner, location), location));

                ExpressionKind::If(Box::new(IfExpression { condition, consequence, alternative }))
            }
            HirMatch::Switch(variable, cases, default) => {
                let location = interner.definition(*variable).location;
                let ident = HirIdent::non_trait_method(*variable, location);
                let expression = ident.to_display_expr(interner, &None, location);
                let expression = Expression::new(expression, location);

                let mut rules = vecmap(cases, |case| {
                    let args = vecmap(&case.arguments, |arg| arg.to_display_ast(interner));
                    let constructor = case.constructor.to_display_ast(args);
                    let constructor = Expression::new(constructor, location);
                    let branch = case.body.to_display_ast(interner, location);
                    (constructor, Expression::new(branch, location))
                });

                if let Some(case) = default {
                    let kind =
                        ExpressionKind::Variable(Path::from_single("_".to_string(), location));
                    let pattern = Expression::new(kind, location);
                    let branch = Expression::new(case.to_display_ast(interner, location), location);
                    rules.push((pattern, branch));
                }

                ExpressionKind::Match(Box::new(MatchExpression { expression, rules }))
            }
        }
    }
}

impl DefinitionId {
    fn to_display_ast(self, interner: &NodeInterner) -> Expression {
        let location = interner.definition(self).location;
        let kind =
            HirIdent::non_trait_method(self, location).to_display_expr(interner, &None, location);
        Expression::new(kind, location)
    }
}

impl Constructor {
    fn to_display_ast(&self, arguments: Vec<Expression>) -> ExpressionKind {
        match self {
            Constructor::True => ExpressionKind::Literal(Literal::Bool(true)),
            Constructor::False => ExpressionKind::Literal(Literal::Bool(false)),
            Constructor::Unit => ExpressionKind::Literal(Literal::Unit),
            Constructor::Int(value) => ExpressionKind::Literal(Literal::Integer(*value, None)),
            Constructor::Tuple(_) => ExpressionKind::Tuple(arguments),
            Constructor::Variant(typ, index) => {
                let typ = typ.follow_bindings_shallow();
                let Type::DataType(def, _) = typ.as_ref() else {
                    return ExpressionKind::Error;
                };

                let Some(variants) = def.borrow().get_variants_as_written() else {
                    return ExpressionKind::Error;
                };

                let Some(name) = variants.get(*index).map(|variant| variant.name.clone()) else {
                    return ExpressionKind::Error;
                };

                let location = name.location();
                let name = ExpressionKind::Variable(Path::from_ident(name));
                let func = Box::new(Expression::new(name, location));
                let is_macro_call = false;
                ExpressionKind::Call(Box::new(CallExpression { func, arguments, is_macro_call }))
            }
            Constructor::Range(_start, _end) => {
                unreachable!("Range is unimplemented")
            }
        }
    }
}

impl ExprId {
    /// Convert to AST for display (some details lost)
    pub fn to_display_ast(self, interner: &NodeInterner) -> Expression {
        let expression = interner.expression(&self);
        // TODO: empty 0 span
        let location = interner.try_id_location(self).unwrap_or_else(Location::dummy);
        expression.to_display_ast(interner, location)
    }
}

impl HirPattern {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> Pattern {
        match self {
            HirPattern::Identifier(ident) => Pattern::Identifier(ident.to_display_ast(interner)),
            HirPattern::Mutable(pattern, location) => {
                let pattern = Box::new(pattern.to_display_ast(interner));
                Pattern::Mutable(pattern, *location, false)
            }
            HirPattern::Tuple(patterns, location) => {
                let patterns = vecmap(patterns, |pattern| pattern.to_display_ast(interner));
                Pattern::Tuple(patterns, *location)
            }
            HirPattern::Struct(typ, patterns, location) => {
                let patterns = vecmap(patterns, |(name, pattern)| {
                    (name.clone(), pattern.to_display_ast(interner))
                });
                let name = match typ.follow_bindings() {
                    Type::DataType(struct_def, _) => {
                        let struct_def = struct_def.borrow();
                        struct_def.name.to_string()
                    }
                    // This pass shouldn't error so if the type isn't a struct we just get a string
                    // representation of any other type and use that. We're relying on name
                    // resolution to fail later when this Ast is re-converted to Hir.
                    other => other.to_string(),
                };
                // The name span is lost here
                let path = Path::from_single(name, *location);
                Pattern::Struct(path, patterns, *location)
            }
        }
    }
}

impl HirIdent {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> Ident {
        let name = interner.definition_name(self.id).to_owned();
        Ident::new(name, self.location)
    }

    fn to_display_expr(
        &self,
        interner: &NodeInterner,
        generics: &Option<Vec<Type>>,
        location: Location,
    ) -> ExpressionKind {
        let ident = self.to_display_ast(interner);
        let segment = PathSegment {
            ident,
            generics: generics
                .as_ref()
                .map(|option| option.iter().map(|generic| generic.to_display_ast()).collect()),
            location,
        };

        let path = Path::plain(vec![segment], location);

        ExpressionKind::Variable(path)
    }
}

impl Type {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self) -> UnresolvedType {
        let typ = match self {
            Type::FieldElement => UnresolvedTypeData::field(Location::dummy()),
            Type::Array(length, element) => {
                let length = length.to_type_expression();
                let element = Box::new(element.to_display_ast());
                UnresolvedTypeData::Array(length, element)
            }
            Type::Vector(element) => {
                let element = Box::new(element.to_display_ast());
                UnresolvedTypeData::Vector(element)
            }
            Type::Integer(sign, bit_size) => {
                UnresolvedTypeData::integer(*sign, *bit_size, Location::dummy())
            }
            Type::Bool => UnresolvedTypeData::bool(Location::dummy()),
            Type::String(length) => {
                let length = length.to_type_expression();
                UnresolvedTypeData::str(length, Location::dummy())
            }
            Type::FmtString(length, element) => {
                let length = length.to_type_expression();
                let element = element.to_display_ast();
                UnresolvedTypeData::fmtstr(length, element, Location::dummy())
            }
            Type::Unit => UnresolvedTypeData::Unit,
            Type::Tuple(fields) => {
                let fields = vecmap(fields, |field| field.to_display_ast());
                UnresolvedTypeData::Tuple(fields)
            }
            Type::DataType(def, generics) => {
                let struct_def = def.borrow();
                let ordered_args = vecmap(generics, |generic| generic.to_display_ast());
                let generics =
                    GenericTypeArgs { ordered_args, named_args: Vec::new(), kinds: Vec::new() };
                let name = Path::from_ident(struct_def.name.clone());
                UnresolvedTypeData::Named(name, generics, false)
            }
            Type::Alias(type_def, generics) => {
                // Keep the alias name instead of expanding this in case the
                // alias' definition was changed
                let type_def = type_def.borrow();
                let ordered_args = vecmap(generics, |generic| generic.to_display_ast());
                let generics =
                    GenericTypeArgs { ordered_args, named_args: Vec::new(), kinds: Vec::new() };
                let name = Path::from_ident(type_def.name.clone());
                UnresolvedTypeData::Named(name, generics, false)
            }
            Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => return typ.to_display_ast(),
                TypeBinding::Unbound(id, type_var_kind) => {
                    let name = format!("var_{type_var_kind:?}_{id}");
                    let path =
                        Path::from_single(name, Location::new(Span::empty(0), FileId::dummy()));
                    let expression = UnresolvedTypeExpression::Variable(path);
                    UnresolvedTypeData::Expression(expression)
                }
            },
            Type::TraitAsType(_, name, generics) => {
                let ordered_args = vecmap(&generics.ordered, |generic| generic.to_display_ast());
                let named_args = vecmap(&generics.named, |named_type| {
                    (named_type.name.clone(), named_type.typ.to_display_ast())
                });
                let generics = GenericTypeArgs { ordered_args, named_args, kinds: Vec::new() };
                let name = Path::from_single(name.as_ref().clone(), Location::dummy());
                UnresolvedTypeData::TraitAsType(name, generics)
            }
            Type::NamedGeneric(NamedGeneric { name, .. }) => {
                let name = Path::from_single(name.as_ref().clone(), Location::dummy());
                UnresolvedTypeData::Named(name, GenericTypeArgs::default(), true)
            }
            Type::CheckedCast { to, .. } => to.to_display_ast().typ,
            Type::Function(args, ret, env, unconstrained) => {
                let args = vecmap(args, |arg| arg.to_display_ast());
                let ret = Box::new(ret.to_display_ast());
                let env = Box::new(env.to_display_ast());
                UnresolvedTypeData::Function(args, ret, env, *unconstrained)
            }
            Type::Reference(element, mutable) => {
                let element = Box::new(element.to_display_ast());
                UnresolvedTypeData::Reference(element, *mutable)
            }
            // Type::Forall is only for generic functions which don't store a type
            // in their Ast so they don't need to call to_display_ast for their Forall type.
            // Since there is no UnresolvedTypeData equivalent for Type::Forall, we use
            // this to ignore this case since it shouldn't be needed anyway.
            Type::Forall(_, typ) => return typ.to_display_ast(),
            Type::Constant(value, kind) => {
                UnresolvedTypeData::Expression(UnresolvedTypeExpression::Constant(
                    *value,
                    kind.as_integer_type_suffix(),
                    Location::dummy(),
                ))
            }
            Type::Quoted(quoted_type) => {
                UnresolvedTypeData::quoted(*quoted_type, Location::dummy())
            }
            Type::Error => UnresolvedTypeData::Error,
            Type::InfixExpr(lhs, op, rhs, _) => {
                let lhs = Box::new(lhs.to_type_expression());
                let rhs = Box::new(rhs.to_type_expression());
                let location = Location::dummy();
                let expr = UnresolvedTypeExpression::BinaryOperation(lhs, *op, rhs, location);
                UnresolvedTypeData::Expression(expr)
            }
        };

        UnresolvedType { typ, location: Location::dummy() }
    }

    /// Convert to AST for display (some details lost)
    fn to_type_expression(&self) -> UnresolvedTypeExpression {
        let location = Location::dummy();

        match self.follow_bindings() {
            Type::Constant(length, kind) => {
                UnresolvedTypeExpression::Constant(length, kind.as_integer_type_suffix(), location)
            }
            Type::NamedGeneric(NamedGeneric { name, .. }) => {
                let path = Path::from_single(name.as_ref().clone(), location);
                UnresolvedTypeExpression::Variable(path)
            }
            // TODO: This should be turned into a proper error.
            other => panic!("Cannot represent {other:?} as type expression"),
        }
    }
}

impl HirLValue {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> LValue {
        match self {
            HirLValue::Ident(path, _) => {
                LValue::Path(Path::from_ident(path.to_display_ast(interner)))
            }
            HirLValue::MemberAccess { object, field_name, field_index: _, typ: _, location } => {
                let object = Box::new(object.to_display_ast(interner));
                LValue::MemberAccess { object, field_name: field_name.clone(), location: *location }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array = Box::new(array.to_display_ast(interner));
                let index = index.to_display_ast(interner);
                LValue::Index { array, index, location: *location }
            }
            HirLValue::Dereference { lvalue, element_type: _, location, implicitly_added: _ } => {
                let lvalue = Box::new(lvalue.to_display_ast(interner));
                LValue::Dereference(lvalue, *location)
            }
        }
    }
}

impl HirArrayLiteral {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner, location: Location) -> ArrayLiteral {
        match self {
            HirArrayLiteral::Standard(elements) => {
                ArrayLiteral::Standard(vecmap(elements, |element| element.to_display_ast(interner)))
            }
            HirArrayLiteral::Repeated { repeated_element, length } => {
                let repeated_element = Box::new(repeated_element.to_display_ast(interner));
                let length = match length {
                    Type::Constant(length, kind) => {
                        let suffix = kind.as_integer_type_suffix();
                        let literal = Literal::Integer(*length, suffix);
                        let expr_kind = ExpressionKind::Literal(literal);
                        Box::new(Expression::new(expr_kind, location))
                    }
                    other => panic!(
                        "Cannot convert non-constant type for repeated array literal from Hir -> Ast: {other:?}"
                    ),
                };
                ArrayLiteral::Repeated { repeated_element, length }
            }
        }
    }
}

impl HirBlockExpression {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> BlockExpression {
        let statements =
            vecmap(self.statements.clone(), |statement| statement.to_display_ast(interner));
        BlockExpression { statements }
    }
}
