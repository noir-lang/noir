use std::{cell::RefCell, rc::Rc};

use noirc_errors::Span;

use crate::{
    hir_def::{
        expr::{self, HirBinaryOp, HirBinaryOpKind, HirExpression, HirLiteral, HirUnaryOp},
        function::Param,
        types::Type,
    },
    node_interner::{ExprId, FuncId, NodeInterner},
    util::vecmap,
    ArraySize, IsConst, TypeBinding,
};

use super::errors::TypeCheckError;

pub(crate) fn type_check_expression(
    interner: &mut NodeInterner,
    expr_id: &ExprId,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let typ = match interner.expression(expr_id) {
        HirExpression::Ident(ident) => {
            // If an Ident is used in an expression, it cannot be a declaration statement
            interner.id_type(ident.id)
        }
        HirExpression::Literal(literal) => {
            match literal {
                HirLiteral::Array(arr) => {
                    // Type check the contents of the array
                    let elem_types =
                        vecmap(&arr.contents, |arg| type_check_expression(interner, arg, errors));

                    let first_elem_type = elem_types.get(0).cloned().unwrap_or(Type::Error);

                    // Specify the type of the Array
                    // Note: This assumes that the array is homogeneous, which will be checked next
                    let arr_type = Type::Array(
                        ArraySize::Fixed(elem_types.len() as u128),
                        Box::new(first_elem_type.clone()),
                    );

                    // Check if the array is homogeneous
                    for (index, elem_type) in elem_types.iter().enumerate().skip(1) {
                        let location = interner.expr_location(&arr.contents[index]);

                        elem_type.unify(&first_elem_type, location.span, errors, || {
                            TypeCheckError::NonHomogeneousArray {
                                first_span: interner.expr_location(&arr.contents[0]).span,
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
                HirLiteral::Bool(_) => Type::Bool(IsConst::new(interner)),
                HirLiteral::Integer(_) => {
                    let id = interner.next_type_variable_id();
                    Type::PolymorphicInteger(
                        IsConst::new(interner),
                        Rc::new(RefCell::new(TypeBinding::Unbound(id))),
                    )
                }
                HirLiteral::Str(_) => unimplemented!(
                    "[Coming Soon] : Currently string literal types have not been implemented"
                ),
            }
        }
        HirExpression::Infix(infix_expr) => {
            // The type of the infix expression must be looked up from a type table
            let lhs_type = type_check_expression(interner, &infix_expr.lhs, errors);
            let rhs_type = type_check_expression(interner, &infix_expr.rhs, errors);

            match infix_operand_type_rules(&lhs_type, &infix_expr.operator, &rhs_type, errors) {
                Ok(typ) => typ,
                Err(msg) => {
                    let lhs_span = interner.expr_span(&infix_expr.lhs);
                    let rhs_span = interner.expr_span(&infix_expr.rhs);
                    errors
                        .push(TypeCheckError::Unstructured { msg, span: lhs_span.merge(rhs_span) });
                    Type::Error
                }
            }
        }
        HirExpression::Index(index_expr) => {
            type_check_index_expression(interner, index_expr, errors)
        }
        HirExpression::Call(call_expr) => {
            let args =
                vecmap(&call_expr.arguments, |arg| type_check_expression(interner, arg, errors));
            type_check_function_call(interner, expr_id, &call_expr.func_id, args, errors)
        }
        HirExpression::MethodCall(method_call) => {
            let object_type = type_check_expression(interner, &method_call.object, errors);
            let method_name = method_call.method.0.contents.as_str();
            match lookup_method(interner, object_type.clone(), method_name, expr_id, errors) {
                Some(method_id) => {
                    let mut args = vec![object_type];
                    let mut arg_types = vecmap(&method_call.arguments, |arg| {
                        type_check_expression(interner, arg, errors)
                    });
                    args.append(&mut arg_types);
                    let ret = type_check_function_call(interner, expr_id, &method_id, args, errors);

                    // Desugar the method call into a normal, resolved function call
                    // so that the backend doesn't need to worry about methods
                    let function_call = method_call.into_function_call(method_id);
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
            start_range_type.unify(&Type::constant(Some(span)), span, errors, || {
                TypeCheckError::TypeCannotBeUsed {
                    typ: start_range_type.clone(),
                    place: "for loop",
                    span,
                }
                .add_context("The range of a loop must be const (known at compile-time)")
            });

            let span = interner.expr_span(&for_expr.end_range);
            end_range_type.unify(&Type::constant(Some(span)), span, errors, || {
                TypeCheckError::TypeCannotBeUsed {
                    typ: end_range_type.clone(),
                    place: "for loop",
                    span,
                }
                .add_context("The range of a loop must be const (known at compile-time)")
            });

            interner.push_definition_type(for_expr.identifier.id, start_range_type);

            let last_type = type_check_expression(interner, &for_expr.block, errors);
            Type::Array(ArraySize::Variable, Box::new(last_type))
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
        HirExpression::MemberAccess(access) => check_member_access(access, interner, errors),
        HirExpression::Error => Type::Error,
        HirExpression::Tuple(elements) => {
            Type::Tuple(vecmap(&elements, |elem| type_check_expression(interner, elem, errors)))
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

    index_type.unify(&Type::constant(Some(span)), span, errors, || {
        // Specialize the error in the case the user has a Field, just not a const one.
        if matches!(index_type, Type::FieldElement(..)) {
            TypeCheckError::Unstructured {
                msg: format!("Array index must be const (known at compile-time), but here a non-const {} was used instead", index_type),
                span,
            }
        } else {
            TypeCheckError::TypeMismatch {
                expected_typ: "const Field".to_owned(),
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
    let is_const = match from {
        Type::Integer(is_const, ..) => is_const,
        Type::FieldElement(is_const) => is_const,
        Type::PolymorphicInteger(is_const, binding) => match &*binding.borrow() {
            TypeBinding::Bound(from) => return check_cast(from.clone(), to, span, errors),
            TypeBinding::Unbound(_) => is_const,
        },
        Type::Bool(is_const) => is_const,
        Type::Error => return Type::Error,
        from => {
            let msg = format!(
                "Cannot cast type {}, 'as' is only for primitive field or integer types",
                from
            );
            errors.push(TypeCheckError::Unstructured { msg, span });
            return Type::Error;
        }
    };

    match to {
        Type::Integer(dest_is_const, sign, bits) => {
            if dest_is_const.is_const() && is_const.unify(&dest_is_const, span).is_err() {
                let msg = "Cannot cast to a const type, argument to cast is non-const (not known at compile-time)".into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }

            Type::Integer(is_const, sign, bits)
        }
        Type::FieldElement(dest_is_const) => {
            if dest_is_const.is_const() && is_const.unify(&dest_is_const, span).is_err() {
                let msg = "Cannot cast to a const type, argument to cast is non-const (not known at compile-time)".into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }

            Type::FieldElement(is_const)
        }
        Type::Bool(dest_is_const) => {
            if dest_is_const.is_const() && is_const.unify(&dest_is_const, span).is_err() {
                let msg = "Cannot cast to a const type, argument to cast is non-const (not known at compile-time)".into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }
            Type::Bool(dest_is_const)
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
    match object_type {
        Type::Struct(ref typ) => {
            let typ = typ.borrow();
            match typ.methods.get(method_name) {
                Some(method_id) => Some(*method_id),
                None => {
                    errors.push(TypeCheckError::Unstructured {
                        span: interner.expr_span(expr_id),
                        msg: format!(
                            "No method named '{}' found for type '{}'",
                            method_name, object_type
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
                msg: format!("Type '{}' must be a struct type to call methods on it", other),
            });
            None
        }
    }
}

fn type_check_function_call(
    interner: &mut NodeInterner,
    expr_id: &ExprId,
    func_id: &FuncId,
    arguments: Vec<Type>,
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
            let span = interner.expr_span(expr_id);
            errors.push(TypeCheckError::ArityMisMatch {
                expected: param_len as u16,
                found: arg_len as u16,
                span,
            });
        }

        // Check for argument param equality
        // In the case where we previously issued an error for a parameter count mismatch
        // this will only check up until the shorter of the two Vecs.
        // Type check arguments
        for (param, arg_type) in func_meta.parameters.iter().zip(arguments) {
            check_param_argument(interner, *expr_id, param, &arg_type, errors);
        }

        // The type of the call expression is the return type of the function being called
        func_meta.return_type
    }
}

pub fn prefix_operand_type_rules(op: &HirUnaryOp, rhs_type: &Type) -> Result<Type, String> {
    match op {
        HirUnaryOp::Minus => {
            if !matches!(rhs_type, Type::Integer(..) | Type::Error) {
                return Err("Only Integers can be used in a Minus expression".to_string());
            }
        }
        HirUnaryOp::Not => {
            if !matches!(rhs_type, Type::Integer(..) | Type::Bool(_) | Type::Error) {
                return Err("Only Integers or Bool can be used in a Not expression".to_string());
            }
        }
    }
    Ok(rhs_type.clone())
}

// Given a binary operator and another type. This method will produce the output type
// XXX: Review these rules. In particular, the interaction between integers, constants and private/public variables
pub fn infix_operand_type_rules(
    lhs_type: &Type,
    op: &HirBinaryOp,
    rhs_type: &Type,
    errors: &mut Vec<TypeCheckError>,
) -> Result<Type, String> {
    if op.kind.is_comparator() {
        return comparator_operand_type_rules(lhs_type, rhs_type, op, errors);
    }

    use Type::*;
    match (lhs_type, rhs_type)  {
        (Integer(is_const_x, sign_x, bit_width_x), Integer(is_const_y, sign_y, bit_width_y)) => {
            if sign_x != sign_y {
                return Err(format!("Integers must have the same signedness LHS is {:?}, RHS is {:?} ", sign_x, sign_y))
            }
            if bit_width_x != bit_width_y {
                return Err(format!("Integers must have the same bit width LHS is {}, RHS is {} ", bit_width_x, bit_width_y))
            }
            let is_const = is_const_x.and(is_const_y, op.location.span);
            Ok(Integer(is_const, *sign_x, *bit_width_x))
        }
        (Integer(..), FieldElement(..)) | (FieldElement(..), Integer(..)) => {
            Err("Cannot use an integer and a Field in a binary operation, try converting the Field into an integer".to_string())
        }
        (PolymorphicInteger(is_const, int), other)
        | (other, PolymorphicInteger(is_const, int)) => {
            if let TypeBinding::Bound(binding) = &*int.borrow() {
                return infix_operand_type_rules(binding, op, other, errors);
            }
            if other.try_bind_to_polymorphic_int(int, is_const, op.location.span).is_ok() || other == &Type::Error {
                Ok(other.clone())
            } else {
                Err(format!("Types in a binary operation should match, but found {} and {}", lhs_type, rhs_type))
            }
        }
        (Integer(..), typ) | (typ,Integer(..)) => {
            Err(format!("Integer cannot be used with type {}", typ))
        }
        // These types are not supported in binary operations
        (Array(..), _) | (_, Array(..)) => Err("Arrays cannot be used in an infix operation".to_string()),
        (Struct(..), _) | (_, Struct(..)) => Err("Structs cannot be used in an infix operation".to_string()),
        (Tuple(_), _) | (_, Tuple(_)) => Err("Tuples cannot be used in an infix operation".to_string()),

        // An error type on either side will always return an error
        (Error, _) | (_,Error) => Ok(Error),
        (Unspecified, _) | (_,Unspecified) => Ok(Unspecified),
        (Unit, _) | (_,Unit) => Ok(Unit),

        // The result of two Fields is always a witness
        (FieldElement(is_const_x), FieldElement(is_const_y)) => {
            let is_const = is_const_x.and(is_const_y, op.location.span);
            Ok(FieldElement(is_const))
        }

        (Bool(is_const_x), Bool(is_const_y)) => Ok(Bool(is_const_x.and(is_const_y, op.location.span))),

        (lhs, rhs) => Err(format!("Unsupported types for binary operation: {} and {}", lhs, rhs)),
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
    cond_type.unify(&Type::Bool(IsConst::new(interner)), expr_span, errors, || {
        TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool(IsConst::No(None)).to_string(),
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
    assert_eq!(constructor.fields.len(), typ.borrow().fields.len());

    // Sort argument types by name so we can zip with the struct type in the same ordering.
    // Note that we use a Vec to store the original arguments (rather than a BTreeMap) to
    // preserve the evaluation order of the source code.
    let mut args = constructor.fields.clone();
    args.sort_by_key(|arg| arg.0.clone());

    for ((param_name, param_type), (arg_ident, arg)) in typ.borrow().fields.iter().zip(args) {
        // Sanity check to ensure we're matching against the same field
        assert_eq!(param_name, &arg_ident);

        let arg_type = type_check_expression(interner, &arg, errors);

        let span = interner.expr_span(expr_id);
        arg_type.make_subtype_of(param_type, span, errors, || TypeCheckError::TypeMismatch {
            expected_typ: param_type.to_string(),
            expr_typ: arg_type.to_string(),
            expr_span: span,
        });
    }

    Type::Struct(typ.clone())
}

pub fn check_member_access(
    access: expr::HirMemberAccess,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let lhs_type = type_check_expression(interner, &access.lhs, errors);

    if let Type::Struct(s) = &lhs_type {
        let s = s.borrow();
        if let Some(field) = s.get_field(&access.rhs.0.contents) {
            // TODO: Should the struct's visibility be applied to the field?
            return field.clone();
        }
    } else if let Type::Tuple(elements) = &lhs_type {
        if let Ok(index) = access.rhs.0.contents.parse::<usize>() {
            if index < elements.len() {
                return elements[index].clone();
            }
        }
    }

    if lhs_type != Type::Error {
        errors.push(TypeCheckError::Unstructured {
            msg: format!("Type {} has no member named {}", lhs_type, access.rhs),
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
    use HirBinaryOpKind::{Equal, NotEqual};
    use Type::*;
    match (lhs_type, rhs_type)  {
        (Integer(is_const_x, sign_x, bit_width_x), Integer(is_const_y, sign_y, bit_width_y)) => {
            if sign_x != sign_y {
                return Err(format!("Integers must have the same signedness LHS is {:?}, RHS is {:?} ", sign_x, sign_y))
            }
            if bit_width_x != bit_width_y {
                return Err(format!("Integers must have the same bit width LHS is {}, RHS is {} ", bit_width_x, bit_width_y))
            }
            let is_const = is_const_x.and(is_const_y, op.location.span);
            Ok(Bool(is_const))
        }
        (Integer(..), FieldElement(..)) | ( FieldElement(..), Integer(..) ) => {
            Err("Cannot use an integer and a Field in a binary operation, try converting the Field into an integer first".to_string())
        }
        (PolymorphicInteger(is_const, int), other)
        | (other, PolymorphicInteger(is_const, int)) => {
            if let TypeBinding::Bound(binding) = &*int.borrow() {
                return comparator_operand_type_rules(other, binding, op, errors);
            }
            if other.try_bind_to_polymorphic_int(int, is_const, op.location.span).is_ok() || other == &Type::Error {
                Ok(Bool(is_const.clone()))
            } else {
                Err(format!("Types in a binary operation should match, but found {} and {}", lhs_type, rhs_type))
            }
        }
        (Integer(..), typ) | (typ,Integer(..)) => {
            Err(format!("Integer cannot be used with type {}", typ))
        }
        (FieldElement(is_const_x, ..), FieldElement(is_const_y, ..)) => {
            let is_const = is_const_x.and(is_const_y, op.location.span);
            Ok(Bool(is_const))
        }

        // <= and friends are technically valid for booleans, just not very useful
        (Bool(is_const_x), Bool(is_const_y)) => {
            let is_const = is_const_x.and(is_const_y, op.location.span);
            Ok(Bool(is_const))
        }

        // Avoid reporting errors multiple times
        (Error, _) | (_,Error) => Ok(Bool(IsConst::Yes(None))),
        (Unspecified, _) | (_,Unspecified) => Ok(Bool(IsConst::Yes(None))),

        // Special-case == and != for arrays
        (Array(x_size, x_type), Array(y_size, y_type)) if matches!(op.kind, Equal | NotEqual) => {
            x_type.unify(y_type, op.location.span, errors, &mut || {
                TypeCheckError::Unstructured {
                    msg: format!("Cannot compare {} and {}, the array element types differ", lhs_type, rhs_type),
                    span: op.location.span,
                }
            });

            if x_size != y_size {
                return Err(format!("Can only compare arrays of the same length. Here LHS is of length {}, and RHS is {} ", 
                    x_size, y_size));
            }

            // We could check if all elements of all arrays are const but I am lazy
            Ok(Bool(IsConst::No(Some(op.location.span))))
        }
        (lhs, rhs) => Err(format!("Unsupported types for comparison: {} and {}", lhs, rhs)),
    }
}

fn check_param_argument(
    interner: &NodeInterner,
    expr_id: ExprId,
    param: &Param,
    arg_type: &Type,
    errors: &mut Vec<TypeCheckError>,
) {
    let param_type = &param.1;

    if arg_type.is_variable_sized_array() {
        unreachable!("arg type type cannot be a variable sized array. This is not supported.")
    }

    let expr_span = interner.expr_span(&expr_id);
    arg_type.make_subtype_of(param_type, expr_span, errors, || TypeCheckError::TypeMismatch {
        expected_typ: param_type.to_string(),
        expr_typ: arg_type.to_string(),
        expr_span,
    });
}
