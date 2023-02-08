use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    hir_def::{
        expr::{self, HirBinaryOp, HirExpression, HirLiteral},
        types::Type,
    },
    node_interner::{ExprId, FuncId, NodeInterner},
    CompTime, Shared, TypeBinding,
};

use super::{bind_pattern, errors::TypeCheckError};

pub(crate) fn type_check_expression(
    interner: &mut NodeInterner,
    expr_id: &ExprId,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let typ = match interner.expression(expr_id) {
        HirExpression::Ident(ident) => {
            // An identifiers type may be forall-quantified in the case of generic functions.
            // E.g. `fn foo<T>(t: T, field: Field) -> T` has type `forall T. fn(T, Field) -> T`.
            // We must instantiate identifiers at every call site to replace this T with a new type
            // variable to handle generic functions.
            let t = interner.id_type(ident.id);
            let (typ, bindings) = t.instantiate(interner);
            interner.store_instantiation_bindings(*expr_id, bindings);
            typ
        }
        HirExpression::Literal(literal) => {
            match literal {
                HirLiteral::Array(arr) => {
                    let elem_types =
                        vecmap(&arr, |arg| type_check_expression(interner, arg, errors));

                    let first_elem_type = elem_types.get(0).cloned().unwrap_or(Type::Error);

                    let arr_type = Type::Array(
                        Box::new(Type::Constant(arr.len() as u64)),
                        Box::new(first_elem_type.clone()),
                    );

                    // Check if the array is homogeneous
                    for (index, elem_type) in elem_types.iter().enumerate().skip(1) {
                        let location = interner.expr_location(&arr[index]);

                        elem_type.unify(&first_elem_type, location.span, errors, || {
                            TypeCheckError::NonHomogeneousArray {
                                first_span: interner.expr_location(&arr[0]).span,
                                first_type: first_elem_type.to_string(),
                                first_index: index,
                                second_span: location.span,
                                second_type: elem_type.to_string(),
                                second_index: index + 1,
                            }
                            .add_context("elements in an array must have the same type")
                        });
                    }

                    arr_type
                }
                HirLiteral::Bool(_) => Type::Bool(CompTime::new(interner)),
                HirLiteral::Integer(_) => {
                    let id = interner.next_type_variable_id();
                    Type::PolymorphicInteger(
                        CompTime::new(interner),
                        Shared::new(TypeBinding::Unbound(id)),
                    )
                }
                HirLiteral::Str(string) => {
                    let len = Type::Constant(string.len() as u64);
                    Type::String(Box::new(len))
                }
            }
        }
        HirExpression::Infix(infix_expr) => {
            // The type of the infix expression must be looked up from a type table
            let lhs_type = type_check_expression(interner, &infix_expr.lhs, errors);
            let rhs_type = type_check_expression(interner, &infix_expr.rhs, errors);

            let lhs_span = interner.expr_span(&infix_expr.lhs);
            let rhs_span = interner.expr_span(&infix_expr.rhs);
            let span = lhs_span.merge(rhs_span);

            infix_operand_type_rules(
                &lhs_type,
                &infix_expr.operator,
                &rhs_type,
                span,
                interner,
                errors,
            )
            .unwrap_or_else(|error| {
                errors.push(error);
                Type::Error
            })
        }
        HirExpression::Index(index_expr) => {
            type_check_index_expression(interner, index_expr, errors)
        }
        HirExpression::Call(call_expr) => {
            let function = type_check_expression(interner, &call_expr.func, errors);
            let args = vecmap(&call_expr.arguments, |arg| {
                let typ = type_check_expression(interner, arg, errors);
                (typ, interner.expr_span(arg))
            });
            let span = interner.expr_span(expr_id);
            bind_function_type(function, args, span, interner, errors)
        }
        HirExpression::MethodCall(method_call) => {
            let object_type = type_check_expression(interner, &method_call.object, errors);
            let method_name = method_call.method.0.contents.as_str();
            match lookup_method(interner, object_type.clone(), method_name, expr_id, errors) {
                Some(method_id) => {
                    let mut args = vec![(object_type, interner.expr_span(&method_call.object))];
                    let mut arg_types = vecmap(&method_call.arguments, |arg| {
                        let typ = type_check_expression(interner, arg, errors);
                        (typ, interner.expr_span(arg))
                    });
                    args.append(&mut arg_types);

                    // Desugar the method call into a normal, resolved function call
                    // so that the backend doesn't need to worry about methods
                    let location = method_call.location;
                    let (function_id, function_call) =
                        method_call.into_function_call(method_id, location, interner);

                    let span = interner.expr_span(expr_id);
                    let ret = type_check_method_call(
                        interner,
                        &function_id,
                        &method_id,
                        args,
                        span,
                        errors,
                    );

                    interner.replace_expr(expr_id, function_call);
                    ret
                }
                None => Type::Error,
            }
        }
        HirExpression::Cast(cast_expr) => {
            // Evaluate the LHS
            let lhs_type = type_check_expression(interner, &cast_expr.lhs, errors);
            let span = interner.expr_span(expr_id);
            check_cast(lhs_type, cast_expr.r#type, span, errors)
        }
        HirExpression::For(for_expr) => {
            let start_range_type = type_check_expression(interner, &for_expr.start_range, errors);
            let end_range_type = type_check_expression(interner, &for_expr.end_range, errors);

            let span = interner.expr_span(&for_expr.start_range);
            start_range_type.unify(&Type::comp_time(Some(span)), span, errors, || {
                TypeCheckError::TypeCannotBeUsed {
                    typ: start_range_type.clone(),
                    place: "for loop",
                    span,
                }
                .add_context("The range of a loop must be known at compile-time")
            });

            let span = interner.expr_span(&for_expr.end_range);
            end_range_type.unify(&Type::comp_time(Some(span)), span, errors, || {
                TypeCheckError::TypeCannotBeUsed {
                    typ: end_range_type.clone(),
                    place: "for loop",
                    span,
                }
                .add_context("The range of a loop must be known at compile-time")
            });

            interner.push_definition_type(for_expr.identifier.id, start_range_type);

            type_check_expression(interner, &for_expr.block, errors);
            Type::Unit
        }
        HirExpression::Block(block_expr) => {
            let mut block_type = Type::Unit;

            let statements = block_expr.statements();
            for (i, stmt) in statements.iter().enumerate() {
                let expr_type = super::stmt::type_check(interner, stmt, errors);

                if i + 1 < statements.len() {
                    let id = match interner.statement(stmt) {
                        crate::hir_def::stmt::HirStatement::Expression(expr) => expr,
                        _ => *expr_id,
                    };

                    let span = interner.expr_span(&id);
                    expr_type.unify(&Type::Unit, span, errors, || TypeCheckError::TypeMismatch {
                        expected_typ: Type::Unit.to_string(),
                        expr_typ: expr_type.to_string(),
                        expr_span: span,
                    });
                } else {
                    block_type = expr_type;
                }
            }

            block_type
        }
        HirExpression::Prefix(prefix_expr) => {
            let rhs_type = type_check_expression(interner, &prefix_expr.rhs, errors);
            match prefix_operand_type_rules(&prefix_expr.operator, &rhs_type) {
                Ok(typ) => typ,
                Err(msg) => {
                    let rhs_span = interner.expr_span(&prefix_expr.rhs);
                    errors.push(TypeCheckError::Unstructured { msg, span: rhs_span });
                    Type::Error
                }
            }
        }
        HirExpression::If(if_expr) => check_if_expr(&if_expr, expr_id, interner, errors),
        HirExpression::Constructor(constructor) => {
            check_constructor(&constructor, expr_id, interner, errors)
        }
        HirExpression::MemberAccess(access) => {
            check_member_access(access, interner, *expr_id, errors)
        }
        HirExpression::Error => Type::Error,
        HirExpression::Tuple(elements) => {
            Type::Tuple(vecmap(&elements, |elem| type_check_expression(interner, elem, errors)))
        }
        HirExpression::Lambda(lambda) => {
            let params = vecmap(lambda.parameters, |(pattern, typ)| {
                bind_pattern(interner, &pattern, typ.clone(), errors);
                typ
            });

            let actual_return = type_check_expression(interner, &lambda.body, errors);

            let span = interner.expr_span(&lambda.body);
            actual_return.make_subtype_of(&lambda.return_type, span, errors, || {
                TypeCheckError::TypeMismatch {
                    expected_typ: lambda.return_type.to_string(),
                    expr_typ: actual_return.to_string(),
                    expr_span: span,
                }
            });
            Type::Function(params, Box::new(lambda.return_type))
        }
    };

    interner.push_expr_type(expr_id, typ.clone());
    typ
}

fn type_check_index_expression(
    interner: &mut NodeInterner,
    index_expr: expr::HirIndexExpression,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let index_type = type_check_expression(interner, &index_expr.index, errors);
    let span = interner.expr_span(&index_expr.index);

    index_type.unify(&Type::comp_time(Some(span)), span, errors, || {
        // Specialize the error in the case the user has a Field, just not a `comptime` one.
        if matches!(index_type, Type::FieldElement(..)) {
            TypeCheckError::Unstructured {
                msg: format!("Array index must be known at compile-time, but here a non-comptime {index_type} was used instead"),
                span,
            }
        } else {
            TypeCheckError::TypeMismatch {
                expected_typ: "comptime Field".to_owned(),
                expr_typ: index_type.to_string(),
                expr_span: span,
            }
        }
    });

    let lhs_type = type_check_expression(interner, &index_expr.collection, errors);
    match lhs_type {
        // XXX: We can check the array bounds here also, but it may be better to constant fold first
        // and have ConstId instead of ExprId for constants
        Type::Array(_, base_type) => *base_type,
        Type::Error => Type::Error,
        typ => {
            let span = interner.expr_span(&index_expr.collection);
            errors.push(TypeCheckError::TypeMismatch {
                expected_typ: "Array".to_owned(),
                expr_typ: typ.to_string(),
                expr_span: span,
            });
            Type::Error
        }
    }
}

fn check_cast(from: Type, to: Type, span: Span, errors: &mut Vec<TypeCheckError>) -> Type {
    let is_comp_time = match from {
        Type::Integer(is_comp_time, ..) => is_comp_time,
        Type::FieldElement(is_comp_time) => is_comp_time,
        Type::PolymorphicInteger(is_comp_time, binding) => match &*binding.borrow() {
            TypeBinding::Bound(from) => return check_cast(from.clone(), to, span, errors),
            TypeBinding::Unbound(_) => is_comp_time,
        },
        Type::Bool(is_comp_time) => is_comp_time,
        Type::Error => return Type::Error,
        from => {
            let msg = format!(
                "Cannot cast type {from}, 'as' is only for primitive field or integer types",
            );
            errors.push(TypeCheckError::Unstructured { msg, span });
            return Type::Error;
        }
    };

    let error_message =
        "Cannot cast to a comptime type, argument to cast is not known at compile-time";
    match to {
        Type::Integer(dest_comp_time, sign, bits) => {
            if dest_comp_time.is_comp_time() && is_comp_time.unify(&dest_comp_time, span).is_err() {
                let msg = error_message.into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }

            Type::Integer(is_comp_time, sign, bits)
        }
        Type::FieldElement(dest_comp_time) => {
            if dest_comp_time.is_comp_time() && is_comp_time.unify(&dest_comp_time, span).is_err() {
                let msg = error_message.into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }

            Type::FieldElement(is_comp_time)
        }
        Type::Bool(dest_comp_time) => {
            if dest_comp_time.is_comp_time() && is_comp_time.unify(&dest_comp_time, span).is_err() {
                let msg = error_message.into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }
            Type::Bool(dest_comp_time)
        }
        Type::Error => Type::Error,
        _ => {
            let msg = "Only integer and Field types may be casted to".into();
            errors.push(TypeCheckError::Unstructured { msg, span });
            Type::Error
        }
    }
}

fn lookup_method(
    interner: &mut NodeInterner,
    object_type: Type,
    method_name: &str,
    expr_id: &ExprId,
    errors: &mut Vec<TypeCheckError>,
) -> Option<FuncId> {
    match &object_type {
        Type::Struct(typ, _args) => {
            let typ = typ.borrow();
            match typ.methods.get(method_name) {
                Some(method_id) => Some(*method_id),
                None => {
                    errors.push(TypeCheckError::Unstructured {
                        span: interner.expr_span(expr_id),
                        msg: format!(
                            "No method named '{method_name}' found for type '{object_type}'",
                        ),
                    });
                    None
                }
            }
        }
        // If we fail to resolve the object to a struct type, we have no way of type
        // checking its arguments as we can't even resolve the name of the function
        Type::Error => None,

        // In the future we could support methods for non-struct types if we have a context
        // (in the interner?) essentially resembling HashMap<Type, Methods>
        other => {
            errors.push(TypeCheckError::Unstructured {
                span: interner.expr_span(expr_id),
                msg: format!("Type '{other}' must be a struct type to call methods on it"),
            });
            None
        }
    }
}

// We need a special function to type check method calls since the method
// is not a Expression::Ident it must be manually instantiated here
fn type_check_method_call(
    interner: &mut NodeInterner,
    function_ident_id: &ExprId,
    func_id: &FuncId,
    arguments: Vec<(Type, Span)>,
    span: Span,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    if func_id == &FuncId::dummy_id() {
        Type::Error
    } else {
        let func_meta = interner.function_meta(func_id);

        // Check function call arity is correct
        let param_len = func_meta.parameters.len();
        let arg_len = arguments.len();

        if param_len != arg_len {
            errors.push(TypeCheckError::ArityMisMatch {
                expected: param_len as u16,
                found: arg_len as u16,
                span,
            });
        }

        let (function_type, instantiation_bindings) = func_meta.typ.instantiate(interner);
        interner.store_instantiation_bindings(*function_ident_id, instantiation_bindings);
        interner.push_expr_type(function_ident_id, function_type.clone());
        bind_function_type(function_type, arguments, span, interner, errors)
    }
}

fn bind_function_type(
    function: Type,
    args: Vec<(Type, Span)>,
    span: Span,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    // Could do a single unification for the entire function type, but matching beforehand
    // lets us issue a more precise error on the individual argument that fails to type check.
    match function {
        Type::TypeVariable(binding) => {
            if let TypeBinding::Bound(typ) = &*binding.borrow() {
                return bind_function_type(typ.clone(), args, span, interner, errors);
            }

            let ret = interner.next_type_variable();
            let args = vecmap(args, |(arg, _)| arg);
            let expected = Type::Function(args, Box::new(ret.clone()));
            *binding.borrow_mut() = TypeBinding::Bound(expected);

            ret
        }
        Type::Function(parameters, ret) => {
            if parameters.len() != args.len() {
                let empty_or_s = if parameters.len() == 1 { "" } else { "s" };
                let was_or_were = if args.len() == 1 { "was" } else { "were" };

                errors.push(TypeCheckError::Unstructured {
                    msg: format!(
                        "Function expects {} parameter{} but {} {} given",
                        parameters.len(),
                        empty_or_s,
                        args.len(),
                        was_or_were
                    ),
                    span,
                });
                return Type::Error;
            }

            for (param, (arg, arg_span)) in parameters.iter().zip(args) {
                arg.make_subtype_of(param, arg_span, errors, || TypeCheckError::TypeMismatch {
                    expected_typ: param.to_string(),
                    expr_typ: arg.to_string(),
                    expr_span: arg_span,
                });
            }

            *ret
        }
        Type::Error => Type::Error,
        other => {
            errors.push(TypeCheckError::Unstructured {
                msg: format!("Expected a function, but found a(n) {other}"),
                span,
            });
            Type::Error
        }
    }
}

pub fn prefix_operand_type_rules(op: &crate::UnaryOp, rhs_type: &Type) -> Result<Type, String> {
    match op {
        crate::UnaryOp::Minus => {
            if !matches!(rhs_type, Type::Integer(..) | Type::Error) {
                return Err("Only Integers can be used in a Minus expression".to_string());
            }
        }
        crate::UnaryOp::Not => {
            if !matches!(rhs_type, Type::Integer(..) | Type::Bool(_) | Type::Error) {
                return Err("Only Integers or Bool can be used in a Not expression".to_string());
            }
        }
    }
    Ok(rhs_type.clone())
}

// Given a binary operator and another type. This method will produce the output type
// XXX: Review these rules. In particular, the interaction between integers, comptime and private/public variables
pub fn infix_operand_type_rules(
    lhs_type: &Type,
    op: &HirBinaryOp,
    rhs_type: &Type,
    span: Span,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Result<Type, TypeCheckError> {
    let make_error = move |msg| TypeCheckError::Unstructured { msg, span };

    if op.kind.is_comparator() {
        return comparator_operand_type_rules(lhs_type, rhs_type, op, errors).map_err(make_error);
    }

    use Type::*;
    match (lhs_type, rhs_type)  {
        (Integer(comptime_x, sign_x, bit_width_x), Integer(comptime_y, sign_y, bit_width_y)) => {
            if sign_x != sign_y {
                return Err(make_error(format!("Integers must have the same signedness LHS is {sign_x:?}, RHS is {sign_y:?} ")))
            }
            if bit_width_x != bit_width_y {
                return Err(make_error(format!("Integers must have the same bit width LHS is {bit_width_x}, RHS is {bit_width_y} ")))
            }
            let comptime = comptime_x.and(comptime_y, op.location.span);
            Ok(Integer(comptime, *sign_x, *bit_width_x))
        }
        (Integer(..), FieldElement(..)) | (FieldElement(..), Integer(..)) => {
            Err(make_error("Cannot use an integer and a Field in a binary operation, try converting the Field into an integer".to_string()))
        }
        (PolymorphicInteger(comptime, int), other)
        | (other, PolymorphicInteger(comptime, int)) => {
            if let TypeBinding::Bound(binding) = &*int.borrow() {
                return infix_operand_type_rules(binding, op, other, span, interner, errors);
            }

            if op.is_bitwise() && (other.is_bindable() || other.is_field()) {
                let other = other.follow_bindings();

                // This will be an error if these types later resolve to a Field, or stay
                // polymorphic as the bit size will be unknown. Delay this error until the function
                // finishes resolving so we can still allow cases like `let x: u8 = 1 << 2;`.
                interner.push_delayed_type_check(Box::new(move || {
                    if other.is_field() {
                        Err(make_error("Bitwise operations are invalid on Field types. Try casting the operands to a sized integer type first".into()))
                    } else if other.is_bindable() {
                        Err(make_error("The number of bits to use for this bitwise operation is ambiguous. Either the operand's type or return type should be specified".into()))
                    } else {
                        Ok(())
                    }
                }));
            }

            if other.try_bind_to_polymorphic_int(int, comptime, true, op.location.span).is_ok() || other == &Type::Error {
                Ok(other.clone())
            } else {
                Err(make_error(format!("Types in a binary operation should match, but found {lhs_type} and {rhs_type}")))
            }
        }
        (Integer(..), typ) | (typ,Integer(..)) => {
            Err(make_error(format!("Integer cannot be used with type {typ}")))
        }
        // These types are not supported in binary operations
        (Array(..), _) | (_, Array(..)) => Err(make_error("Arrays cannot be used in an infix operation".to_string())),
        (Struct(..), _) | (_, Struct(..)) => Err(make_error("Structs cannot be used in an infix operation".to_string())),
        (Tuple(_), _) | (_, Tuple(_)) => Err(make_error("Tuples cannot be used in an infix operation".to_string())),

        // An error type on either side will always return an error
        (Error, _) | (_,Error) => Ok(Error),
        (Unit, _) | (_,Unit) => Ok(Unit),

        // The result of two Fields is always a witness
        (FieldElement(comptime_x), FieldElement(comptime_y)) => {
            if op.is_bitwise() {
                return Err(make_error("Bitwise operations are invalid on Field types. Try casting the operands to a sized integer type first.".into()));
            }
            let comptime = comptime_x.and(comptime_y, op.location.span);
            Ok(FieldElement(comptime))
        }

        (Bool(comptime_x), Bool(comptime_y)) => Ok(Bool(comptime_x.and(comptime_y, op.location.span))),

        (TypeVariable(var), other)
        | (other, TypeVariable(var)) => {
            if let TypeBinding::Bound(binding) = &*var.borrow() {
                return infix_operand_type_rules(binding, op, other, span, interner, errors);
            }

            let comptime = CompTime::No(None);
            if other.try_bind_to_polymorphic_int(var, &comptime, true, op.location.span).is_ok() || other == &Type::Error {
                Ok(other.clone())
            } else {
                Err(make_error(format!("Types in a binary operation should match, but found {lhs_type} and {rhs_type}")))
            }
        }

        (lhs, rhs) => Err(make_error(format!("Unsupported types for binary operation: {lhs} and {rhs}"))),
    }
}

fn check_if_expr(
    if_expr: &expr::HirIfExpression,
    expr_id: &ExprId,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let cond_type = type_check_expression(interner, &if_expr.condition, errors);
    let then_type = type_check_expression(interner, &if_expr.consequence, errors);

    let expr_span = interner.expr_span(&if_expr.condition);
    cond_type.unify(&Type::Bool(CompTime::new(interner)), expr_span, errors, || {
        TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool(CompTime::No(None)).to_string(),
            expr_typ: cond_type.to_string(),
            expr_span,
        }
    });

    match if_expr.alternative {
        None => Type::Unit,
        Some(alternative) => {
            let else_type = type_check_expression(interner, &alternative, errors);

            let expr_span = interner.expr_span(expr_id);
            then_type.unify(&else_type, expr_span, errors, || {
                let err = TypeCheckError::TypeMismatch {
                    expected_typ: then_type.to_string(),
                    expr_typ: else_type.to_string(),
                    expr_span,
                };

                let context = if then_type == Type::Unit {
                    "Are you missing a semicolon at the end of your 'else' branch?"
                } else if else_type == Type::Unit {
                    "Are you missing a semicolon at the end of the first block of this 'if'?"
                } else {
                    "Expected the types of both if branches to be equal"
                };

                err.add_context(context)
            });

            then_type
        }
    }
}

fn check_constructor(
    constructor: &expr::HirConstructorExpression,
    expr_id: &ExprId,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let typ = &constructor.r#type;

    // Sanity check, this should be caught during name resolution anyway
    assert_eq!(constructor.fields.len(), typ.borrow().num_fields());

    // Sort argument types by name so we can zip with the struct type in the same ordering.
    // Note that we use a Vec to store the original arguments (rather than a BTreeMap) to
    // preserve the evaluation order of the source code.
    let mut args = constructor.fields.clone();
    args.sort_by_key(|arg| arg.0.clone());

    let typ_ref = typ.borrow();
    let (generics, fields) = typ_ref.instantiate(interner);

    for ((param_name, param_type), (arg_ident, arg)) in fields.into_iter().zip(args) {
        // Sanity check to ensure we're matching against the same field
        assert_eq!(param_name, &arg_ident.0.contents);

        let arg_type = type_check_expression(interner, &arg, errors);

        let span = interner.expr_span(expr_id);
        arg_type.make_subtype_of(&param_type, span, errors, || TypeCheckError::TypeMismatch {
            expected_typ: param_type.to_string(),
            expr_typ: arg_type.to_string(),
            expr_span: span,
        });
    }

    Type::Struct(typ.clone(), generics)
}

pub fn check_member_access(
    access: expr::HirMemberAccess,
    interner: &mut NodeInterner,
    expr_id: ExprId,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let lhs_type = type_check_expression(interner, &access.lhs, errors);

    if let Type::Struct(s, args) = &lhs_type {
        let s = s.borrow();
        if let Some((field, index)) = s.get_field(&access.rhs.0.contents, args) {
            interner.set_field_index(expr_id, index);
            return field;
        }
    } else if let Type::Tuple(elements) = &lhs_type {
        if let Ok(index) = access.rhs.0.contents.parse::<usize>() {
            if index < elements.len() {
                interner.set_field_index(expr_id, index);
                return elements[index].clone();
            }
        }
    }

    if lhs_type != Type::Error {
        errors.push(TypeCheckError::Unstructured {
            msg: format!("Type {lhs_type} has no member named {}", access.rhs),
            span: interner.expr_span(&access.lhs),
        });
    }

    Type::Error
}

pub fn comparator_operand_type_rules(
    lhs_type: &Type,
    rhs_type: &Type,
    op: &HirBinaryOp,
    errors: &mut Vec<TypeCheckError>,
) -> Result<Type, String> {
    use crate::BinaryOpKind::{Equal, NotEqual};
    use Type::*;
    match (lhs_type, rhs_type)  {
        (Integer(comptime_x, sign_x, bit_width_x), Integer(comptime_y, sign_y, bit_width_y)) => {
            if sign_x != sign_y {
                return Err(format!("Integers must have the same signedness LHS is {sign_x:?}, RHS is {sign_y:?} "))
            }
            if bit_width_x != bit_width_y {
                return Err(format!("Integers must have the same bit width LHS is {bit_width_x}, RHS is {bit_width_y} "))
            }
            let comptime = comptime_x.and(comptime_y, op.location.span);
            Ok(Bool(comptime))
        }
        (Integer(..), FieldElement(..)) | ( FieldElement(..), Integer(..) ) => {
            Err("Cannot use an integer and a Field in a binary operation, try converting the Field into an integer first".to_string())
        }
        (PolymorphicInteger(comptime, int), other)
        | (other, PolymorphicInteger(comptime, int)) => {
            if let TypeBinding::Bound(binding) = &*int.borrow() {
                return comparator_operand_type_rules(other, binding, op, errors);
            }
            if other.try_bind_to_polymorphic_int(int, comptime, true, op.location.span).is_ok() || other == &Type::Error {
                Ok(Bool(comptime.clone()))
            } else {
                Err(format!("Types in a binary operation should match, but found {lhs_type} and {rhs_type}"))
            }
        }
        (Integer(..), typ) | (typ,Integer(..)) => {
            Err(format!("Integer cannot be used with type {typ}"))
        }
        (FieldElement(comptime_x), FieldElement(comptime_y)) => {
            match op.kind {
                Equal | NotEqual => {
                    let comptime = comptime_x.and(comptime_y, op.location.span);
                    Ok(Bool(comptime))
                },
                _ => {
                    Err("Fields cannot be compared, try casting to an integer first".into())
                }
            }
        }

        // <= and friends are technically valid for booleans, just not very useful
        (Bool(comptime_x), Bool(comptime_y)) => {
            let comptime = comptime_x.and(comptime_y, op.location.span);
            Ok(Bool(comptime))
        }

        // Avoid reporting errors multiple times
        (Error, _) | (_,Error) => Ok(Bool(CompTime::Yes(None))),

        // Special-case == and != for arrays
        (Array(x_size, x_type), Array(y_size, y_type)) if matches!(op.kind, Equal | NotEqual) => {
            x_type.unify(y_type, op.location.span, errors, || {
                TypeCheckError::Unstructured {
                    msg: format!("Cannot compare {lhs_type} and {rhs_type}, the array element types differ"),
                    span: op.location.span,
                }
            });

            x_size.unify(y_size, op.location.span, errors, || {
                TypeCheckError::Unstructured {
                    msg: format!("Can only compare arrays of the same length. Here LHS is of length {x_size}, and RHS is {y_size}"),
                    span: op.location.span,
                }
            });

            // We could check if all elements of all arrays are comptime but I am lazy
            Ok(Bool(CompTime::No(Some(op.location.span))))
        }
        (NamedGeneric(binding_a, name_a), NamedGeneric(binding_b, name_b)) => {
            if binding_a == binding_b {
                return Ok(Bool(CompTime::No(Some(op.location.span))));
            }
            Err(format!("Unsupported types for comparison: {name_a} and {name_b}"))
        }
        (TypeVariable(var), other)
        | (other, TypeVariable(var)) => {
            if let TypeBinding::Bound(binding) = &*var.borrow() {
                return comparator_operand_type_rules(binding, other, op, errors);
            }

            let comptime = CompTime::No(None);
            if other.try_bind_to_polymorphic_int(var, &comptime, true, op.location.span).is_ok() || other == &Type::Error {
                Ok(other.clone())
            } else {
                Err(format!("Types in a binary operation should match, but found {lhs_type} and {rhs_type}"))
            }
        }
        (String(x_size), String(y_size)) => {
            x_size.unify(y_size, op.location.span, errors, || {
                TypeCheckError::Unstructured {
                    msg: format!("Can only compare strings of the same length. Here LHS is of length {x_size}, and RHS is {y_size} "),
                    span: op.location.span,
                }
            });

            Ok(Bool(CompTime::No(Some(op.location.span))))
        }
        (lhs, rhs) => Err(format!("Unsupported types for comparison: {lhs} and {rhs}")),
    }
}
